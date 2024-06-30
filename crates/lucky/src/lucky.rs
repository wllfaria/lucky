use crate::{
    atoms::Atoms, decorator::Decorator, event::EventContext, ewmh::ewmh_set_wm_hints,
    handlers::Handlers, keyboard::Keyboard, layout_manager::LayoutManager, position::Position,
    screen::Screen, screen_manager::ScreenManager,
};
use anyhow::Context;
use config::{AutoCommand, AvailableActions, Config};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        mpsc::{channel, Sender},
        Arc,
    },
};
use xcb::{
    randr,
    x::{self, ChangeWindowAttributes},
};

pub struct Lucky {
    conn: Arc<xcb::Connection>,
    keyboard: Keyboard,
    config: Rc<RefCell<Config>>,
    handlers: Handlers,
    screen_manager: Rc<RefCell<ScreenManager>>,
    atoms: Atoms,
    layout_manager: LayoutManager,
    decorator: Decorator,
    last_pointer_position: (i16, i16),
}

impl Lucky {
    #[tracing::instrument(skip_all, err)]
    pub fn new() -> anyhow::Result<Self> {
        let (conn, _) = xcb::Connection::connect(None).context("failed to initialize self.conn to the X server. Check the DISPLAY environment variable")?;
        let conn = Arc::new(conn);
        let config = Rc::new(RefCell::new(config::load_config()));
        execute_auto_commands(config.borrow().startup_commands())
            .context("failed to run startup commands")?;

        let root = Self::setup(&conn)?;
        let atoms = Atoms::new(&conn);
        let screens = Self::get_monitors(&conn, root, &config)?;
        let screen_manager = ScreenManager::new(screens, config.clone(), root);

        screen_manager.update_atoms(&atoms, &conn);
        ewmh_set_wm_hints(&conn, root, &atoms).context("failed to setup window manager hints")?;

        conn.flush().expect("failed to flush the connection");

        Ok(Lucky {
            keyboard: Keyboard::new(&conn, config.clone(), root)?,
            layout_manager: LayoutManager::new(conn.clone(), config.clone()),
            decorator: Decorator::new(conn.clone(), config.clone()),
            atoms,
            handlers: Handlers::default(),
            screen_manager: Rc::new(RefCell::new(screen_manager)),

            conn,
            config,
            last_pointer_position: (0, 0),
        })
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let (event_tx, event_rx) = channel::<XEvent>();
        let (action_tx, action_rx) = channel::<AvailableActions>();

        let conn = self.conn.clone();
        let event_tx_c = event_tx.clone();
        std::thread::spawn(move || {
            if poll_events(conn.clone(), event_tx_c).is_err() {
                std::process::abort();
            }
        });

        loop {
            if let Ok(AvailableActions::Reload) = action_rx.try_recv() {
                self.config.borrow_mut().update(config::load_config());
                self.layout_manager
                    .display_screens(&self.screen_manager, &self.decorator)
                    .expect("failed to redraw the screen");
            }

            let pointer_cookie = self.conn.send_request(&xcb::x::QueryPointer {
                window: self
                    .conn
                    .get_setup()
                    .roots()
                    .next()
                    .expect("should have at least one screen")
                    .root(),
            });

            if let Ok(pointer_reply) = self.conn.wait_for_reply(pointer_cookie) {
                let pointer_position = (pointer_reply.root_x(), pointer_reply.root_y());
                if pointer_position.ne(&self.last_pointer_position) {
                    self.last_pointer_position = pointer_position;
                    self.screen_manager.borrow_mut().maybe_switch_screen(
                        pointer_reply,
                        &self.conn,
                        &self.atoms,
                    );
                }
            }

            if let Ok(event) = event_rx.try_recv() {
                match event {
                    XEvent::KeyPress(event) => self.handlers.on_key_press(EventContext {
                        event,
                        conn: self.conn.clone(),
                        keyboard: &self.keyboard,
                        config: self.config.clone(),
                        screen_manager: self.screen_manager.clone(),
                        atoms: &self.atoms,
                        decorator: &self.decorator,
                        layout_manager: &self.layout_manager,
                        action_tx: action_tx.clone(),
                    })?,
                    XEvent::MapRequest(event) => self.handlers.on_map_request(EventContext {
                        event,
                        conn: self.conn.clone(),
                        keyboard: &self.keyboard,
                        config: self.config.clone(),
                        screen_manager: self.screen_manager.clone(),
                        atoms: &self.atoms,
                        decorator: &self.decorator,
                        layout_manager: &self.layout_manager,
                        action_tx: action_tx.clone(),
                    })?,
                    XEvent::DestroyNotify(event) => {
                        self.handlers.on_destroy_notify(EventContext {
                            event,
                            conn: self.conn.clone(),
                            keyboard: &self.keyboard,
                            config: self.config.clone(),
                            screen_manager: self.screen_manager.clone(),
                            atoms: &self.atoms,
                            decorator: &self.decorator,
                            layout_manager: &self.layout_manager,
                            action_tx: action_tx.clone(),
                        })?
                    }
                    XEvent::EnterNotify(event) => self.handlers.on_enter_notify(EventContext {
                        event,
                        conn: self.conn.clone(),
                        keyboard: &self.keyboard,
                        config: self.config.clone(),
                        screen_manager: self.screen_manager.clone(),
                        atoms: &self.atoms,
                        decorator: &self.decorator,
                        layout_manager: &self.layout_manager,
                        action_tx: action_tx.clone(),
                    })?,
                    XEvent::UnmapNotify(event) => self.handlers.on_unmap_notify(EventContext {
                        event,
                        conn: self.conn.clone(),
                        keyboard: &self.keyboard,
                        config: self.config.clone(),
                        screen_manager: self.screen_manager.clone(),
                        atoms: &self.atoms,
                        decorator: &self.decorator,
                        layout_manager: &self.layout_manager,
                        action_tx: action_tx.clone(),
                    })?,
                    XEvent::PropertyNotify(event) => {
                        self.handlers.on_property_notify(EventContext {
                            event,
                            conn: self.conn.clone(),
                            keyboard: &self.keyboard,
                            config: self.config.clone(),
                            screen_manager: self.screen_manager.clone(),
                            atoms: &self.atoms,
                            decorator: &self.decorator,
                            layout_manager: &self.layout_manager,
                            action_tx: action_tx.clone(),
                        })?
                    }
                    XEvent::ConfigureRequest(_) => todo!(),
                };

                self.conn.flush().expect("failed to flush the connection");
            }
        }
    }

    #[tracing::instrument(skip_all, err)]
    fn setup(conn: &Arc<xcb::Connection>) -> anyhow::Result<xcb::x::Window> {
        let screen = conn
            .get_setup()
            .roots()
            .next()
            .context("we must have at least one window to manage")?;
        let root = screen.root();

        let font = conn.generate_id();
        conn.check_request(conn.send_request_checked(&xcb::x::OpenFont {
            fid: font,
            name: b"cursor",
        }))
        .context("failed to open cursor font")?;

        let cursor = conn.generate_id();
        conn.check_request(conn.send_request_checked(&xcb::x::CreateGlyphCursor {
            cid: cursor,
            source_font: font,
            mask_font: font,
            source_char: 68,
            mask_char: 69,
            fore_red: 0,
            fore_green: 0,
            fore_blue: 0,
            back_red: 0xffff,
            back_green: 0xffff,
            back_blue: 0xffff,
        }))
        .context("failed to create a cursor")?;

        conn.check_request(conn.send_request_checked(&ChangeWindowAttributes {
            window: root,
            value_list: &[
                x::Cw::EventMask(
                    x::EventMask::SUBSTRUCTURE_REDIRECT | x::EventMask::SUBSTRUCTURE_NOTIFY,
                ),
                x::Cw::Cursor(cursor),
            ],
        }))
        .context("failed to subscribe for substructure redirection")?;

        Ok(root)
    }

    fn get_monitors(
        conn: &Arc<xcb::Connection>,
        root: xcb::x::Window,
        config: &Rc<RefCell<Config>>,
    ) -> anyhow::Result<Vec<Screen>> {
        let total_screens = conn
            .wait_for_reply(conn.send_request(&randr::GetMonitors {
                window: root,
                get_active: true,
            }))
            .context("failed to get monitors")?;

        let screens = total_screens
            .monitors()
            .map(Into::into)
            .collect::<Vec<Position>>()
            .into_iter()
            .map(|position| Screen::new(config, position))
            .collect::<Vec<_>>();

        Ok(screens)
    }
}

#[tracing::instrument(skip_all, err)]
fn poll_events(conn: Arc<xcb::Connection>, event_tx: Sender<XEvent>) -> anyhow::Result<()> {
    loop {
        if let Ok(event) = conn.wait_for_event() {
            match event {
                xcb::Event::X(xcb::x::Event::KeyPress(e)) => event_tx
                    .send(XEvent::KeyPress(e))
                    .context("failed to send event through channel")?,
                xcb::Event::X(xcb::x::Event::MapRequest(e)) => event_tx
                    .send(XEvent::MapRequest(e))
                    .context("failed to send event through channel")?,
                xcb::Event::X(xcb::x::Event::DestroyNotify(e)) => event_tx
                    .send(XEvent::DestroyNotify(e))
                    .context("failed to send event through channel")?,
                xcb::Event::X(xcb::x::Event::EnterNotify(e)) => event_tx
                    .send(XEvent::EnterNotify(e))
                    .context("failed to send event through channel")?,
                xcb::Event::X(xcb::x::Event::UnmapNotify(e)) => event_tx
                    .send(XEvent::UnmapNotify(e))
                    .context("failed to send event through channel")?,
                xcb::Event::X(xcb::x::Event::PropertyNotify(e)) => event_tx
                    .send(XEvent::PropertyNotify(e))
                    .context("failed to send event through channel")?,
                xcb::Event::X(xcb::x::Event::ConfigureRequest(_)) => {}
                xcb::Event::RandR(xcb::randr::Event::Notify(e)) => {
                    tracing::trace!("from notify randr {e:?}")
                }
                xcb::Event::RandR(xcb::randr::Event::ScreenChangeNotify(e)) => {
                    tracing::trace!("from change screen {e:?}")
                }
                _ => {}
            };
        };
        conn.flush().context("failed to flush the connection")?;
    }
}

#[tracing::instrument(skip_all, err)]
pub fn execute_auto_commands(auto_commands: &[AutoCommand]) -> anyhow::Result<()> {
    for command in auto_commands {
        // TODO: we should store what failed to maybe display a notification
        std::process::Command::new(command.command())
            .args(command.args())
            .spawn()
            .context(format!("failed to spawn command {:?}", command))?;
    }

    Ok(())
}

#[derive(Debug)]
pub enum XEvent {
    KeyPress(xcb::x::KeyPressEvent),
    MapRequest(xcb::x::MapRequestEvent),
    DestroyNotify(xcb::x::DestroyNotifyEvent),
    EnterNotify(xcb::x::EnterNotifyEvent),
    UnmapNotify(xcb::x::UnmapNotifyEvent),
    PropertyNotify(xcb::x::PropertyNotifyEvent),
    ConfigureRequest(xcb::x::ConfigureRequestEvent),
}
