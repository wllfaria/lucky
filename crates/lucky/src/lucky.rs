use crate::{
    atoms::Atoms,
    decorator::Decorator,
    event::EventContext,
    handlers::Handlers,
    keyboard::Keyboard,
    layout_manager::LayoutManager,
    screen_manager::{Position, ScreenManager},
};
use config::{AvailableActions, Config};
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
    x::{self, ChangeProperty, ChangeWindowAttributes},
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
    pub fn new() -> anyhow::Result<Self> {
        let (conn, _) = xcb::Connection::connect(None).expect("failed to initialize self.conn to the X server. Check the DISPLAY environment variable");
        let conn = Arc::new(conn);
        let config = Rc::new(RefCell::new(config::load_config()));
        let root = Self::setup(&conn);
        let screen_positions = Self::get_monitors(&conn, root);

        Ok(Lucky {
            keyboard: Keyboard::new(&conn, config.clone(), root)?,
            layout_manager: LayoutManager::new(conn.clone(), config.clone()),
            decorator: Decorator::new(conn.clone(), config.clone()),
            atoms: Atoms::new(&conn),
            handlers: Handlers::default(),
            screen_manager: Rc::new(RefCell::new(ScreenManager::new(
                screen_positions,
                config.clone(),
            ))),

            conn,
            config,
            last_pointer_position: (0, 0),
        })
    }

    pub fn run(mut self) {
        let (event_tx, event_rx) = channel::<XEvent>();
        let (action_tx, action_rx) = channel::<AvailableActions>();

        let conn = self.conn.clone();
        let event_tx_c = event_tx.clone();
        std::thread::spawn(move || poll_events(conn.clone(), event_tx_c));

        loop {
            if let Ok(AvailableActions::Reload) = action_rx.try_recv() {
                self.config.borrow_mut().update(config::load_config());
                self.layout_manager
                    .display_screens(&self.screen_manager, &self.decorator)
                    .expect("failed to redraw the screen");
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
                    }),
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
                    }),
                    XEvent::DestroyNotify(event) => self.handlers.on_destroy_notify(EventContext {
                        event,
                        conn: self.conn.clone(),
                        keyboard: &self.keyboard,
                        config: self.config.clone(),
                        screen_manager: self.screen_manager.clone(),
                        atoms: &self.atoms,
                        decorator: &self.decorator,
                        layout_manager: &self.layout_manager,
                        action_tx: action_tx.clone(),
                    }),
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
                    }),
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
                    }),
                    XEvent::PropertyNotify(_) => {}
                    XEvent::ConfigureRequest(_) => todo!(),
                }

                self.conn.flush().expect("failed to flush the connection");
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
                    self.screen_manager
                        .borrow_mut()
                        .maybe_switch_screen(pointer_reply);
                }
            }
        }
    }

    fn setup(conn: &Arc<xcb::Connection>) -> xcb::x::Window {
        let screen = conn
            .get_setup()
            .roots()
            .next()
            .expect("we must have at least one window to manage");
        let root = screen.root();

        let font = conn.generate_id();
        conn.check_request(conn.send_request_checked(&xcb::x::OpenFont {
            fid: font,
            name: b"cursor",
        }))
        .expect("failed to open cursor font");

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
        .expect("failed to create a cursor");

        conn.check_request(conn.send_request_checked(&ChangeWindowAttributes {
            window: root,
            value_list: &[
                x::Cw::EventMask(
                    x::EventMask::SUBSTRUCTURE_REDIRECT | x::EventMask::SUBSTRUCTURE_NOTIFY,
                ),
                x::Cw::Cursor(cursor),
            ],
        }))
        .expect("failed to subscribe for substructure redirection");

        conn.send_and_check_request(&ChangeProperty {
            window: root,
            mode: x::PropMode::Replace,
            r#type: xcb::x::ATOM_STRING,
            data: b"LuckyWM",
            property: xcb::x::ATOM_WM_NAME,
        })
        .expect("failed to set window manager name");

        root
    }

    fn get_monitors(conn: &Arc<xcb::Connection>, root: xcb::x::Window) -> Vec<Position> {
        let total_screens = conn
            .wait_for_reply(conn.send_request(&randr::GetMonitors {
                window: root,
                get_active: true,
            }))
            .expect("failed to get monitors");

        total_screens
            .monitors()
            .map(Into::into)
            .collect::<Vec<Position>>()
    }
}

impl From<&xcb::randr::MonitorInfo> for Position {
    fn from(value: &xcb::randr::MonitorInfo) -> Self {
        Position {
            x: value.x().into(),
            y: value.y().into(),
            width: value.width().into(),
            height: value.height().into(),
        }
    }
}

fn poll_events(conn: Arc<xcb::Connection>, event_tx: Sender<XEvent>) {
    loop {
        if let Ok(event) = conn.wait_for_event() {
            match event {
                xcb::Event::X(xcb::x::Event::KeyPress(e)) => {
                    if event_tx.send(XEvent::KeyPress(e)).is_err() {
                        tracing::debug!("failed to send event through channel");
                        std::process::abort();
                    }
                }
                xcb::Event::X(xcb::x::Event::MapRequest(e)) => {
                    if event_tx.send(XEvent::MapRequest(e)).is_err() {
                        tracing::debug!("failed to send event through channel");
                        std::process::abort();
                    }
                }
                xcb::Event::X(xcb::x::Event::DestroyNotify(e)) => {
                    if event_tx.send(XEvent::DestroyNotify(e)).is_err() {
                        tracing::debug!("failed to send event through channel");
                        std::process::abort();
                    }
                }
                xcb::Event::X(xcb::x::Event::EnterNotify(e)) => {
                    if event_tx.send(XEvent::EnterNotify(e)).is_err() {
                        tracing::debug!("failed to send event through channel");
                        std::process::abort();
                    }
                }
                xcb::Event::X(xcb::x::Event::UnmapNotify(e)) => {
                    if event_tx.send(XEvent::UnmapNotify(e)).is_err() {
                        tracing::debug!("failed to send event through channel");
                        std::process::abort();
                    }
                }
                xcb::Event::X(xcb::x::Event::ConfigureRequest(_)) => {}
                xcb::Event::X(xcb::x::Event::PropertyNotify(_)) => {}
                e => tracing::error!("{e:?}"),
            };
        };
        conn.flush().expect("failed to flush the connection");
    }
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
