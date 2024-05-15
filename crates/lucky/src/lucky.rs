use crate::{
    atoms::Atoms, clients::Clients, decorator::Decorator, event::EventContext, handlers::Handlers,
    keyboard::Keyboard, layout_manager::LayoutManager,
};
use config::Config;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{mpsc::channel, Arc},
};
use xcb::x::{self, ChangeProperty, ChangeWindowAttributes};

pub struct Lucky {
    conn: Arc<xcb::Connection>,
    keyboard: Keyboard,
    config: Rc<Config>,
    handlers: Handlers,
    clients: Rc<RefCell<Clients>>,
    atoms: Atoms,
    layout_manager: LayoutManager,
    decorator: Decorator,
}

impl Lucky {
    pub fn new(config: Config) -> Self {
        let (conn, _) = xcb::Connection::connect(None).expect("failed to initialize self.conn to the X server. Check the DISPLAY environment variable");
        let conn = Arc::new(conn);
        let config = Rc::new(config);
        let root = Self::setup(&conn);

        Lucky {
            clients: Rc::new(RefCell::new(Clients::new(&config))),
            keyboard: Keyboard::new(&conn, root, config.clone()),
            layout_manager: LayoutManager::new(conn.clone(), config.clone()),
            decorator: Decorator::new(conn.clone(), config.clone()),
            atoms: Atoms::new(&conn),
            handlers: Handlers::default(),

            conn,
            config,
        }
    }

    pub fn run(mut self) {
        let (sender, receiver) = channel::<XEvent>();

        let conn = self.conn.clone();
        std::thread::spawn(move || loop {
            if let Ok(event) = conn.wait_for_event() {
                match event {
                    xcb::Event::X(xcb::x::Event::KeyPress(e)) => {
                        if sender.send(XEvent::KeyPress(e)).is_err() {
                            tracing::debug!("failed to send event through channel");
                            std::process::abort();
                        }
                    }
                    xcb::Event::X(xcb::x::Event::MapRequest(e)) => {
                        if sender.send(XEvent::MapRequest(e)).is_err() {
                            tracing::debug!("failed to send event through channel");
                            std::process::abort();
                        }
                    }
                    xcb::Event::X(xcb::x::Event::Expose(e)) => {
                        tracing::debug!("here we should show the client {e:?}");
                        conn.flush().unwrap();
                    }
                    xcb::Event::X(xcb::x::Event::MapNotify(e)) => {
                        tracing::debug!("here we should show the map map client {e:?}");
                        if sender.send(XEvent::MapNotify(e)).is_err() {
                            tracing::debug!("failed to send event through channel");
                            std::process::abort();
                        }
                    }
                    xcb::Event::X(xcb::x::Event::DestroyNotify(e)) => {
                        if sender.send(XEvent::DestroyNotify(e)).is_err() {
                            tracing::debug!("failed to send event through channel");
                            std::process::abort();
                        }
                    }
                    xcb::Event::X(xcb::x::Event::EnterNotify(e)) => {
                        // TODO: when entering the window we should focus it if `focus_on_hover` is
                        // enabled
                        if sender.send(XEvent::EnterNotify(e)).is_err() {
                            tracing::debug!("failed to send event through channel");
                            std::process::abort();
                        }
                    }
                    xcb::Event::X(xcb::x::Event::ConfigureRequest(_)) => todo!(),
                    xcb::Event::X(xcb::x::Event::PropertyNotify(_)) => {}
                    xcb::Event::X(xcb::x::Event::UnmapNotify(_)) => {}
                    _ => (),
                };
            };
            conn.flush().unwrap();
        });

        loop {
            if let Ok(event) = receiver.recv() {
                match event {
                    XEvent::KeyPress(event) => self.handlers.on_key_press(EventContext {
                        event,
                        conn: self.conn.clone(),
                        keyboard: &self.keyboard,
                        config: self.config.clone(),
                        clients: self.clients.clone(),
                        atoms: &self.atoms,
                        decorator: &self.decorator,
                        layout_manager: &self.layout_manager,
                    }),
                    XEvent::MapRequest(event) => self.handlers.on_map_request(EventContext {
                        event,
                        conn: self.conn.clone(),
                        keyboard: &self.keyboard,
                        config: self.config.clone(),
                        clients: self.clients.clone(),
                        atoms: &self.atoms,
                        decorator: &self.decorator,
                        layout_manager: &self.layout_manager,
                    }),
                    XEvent::MapNotify(event) => self.handlers.on_map_notify(EventContext {
                        event,
                        conn: self.conn.clone(),
                        keyboard: &self.keyboard,
                        config: self.config.clone(),
                        clients: self.clients.clone(),
                        atoms: &self.atoms,
                        decorator: &self.decorator,
                        layout_manager: &self.layout_manager,
                    }),
                    XEvent::DestroyNotify(event) => self.handlers.on_destroy_notify(EventContext {
                        event,
                        conn: self.conn.clone(),
                        keyboard: &self.keyboard,
                        config: self.config.clone(),
                        clients: self.clients.clone(),
                        atoms: &self.atoms,
                        decorator: &self.decorator,
                        layout_manager: &self.layout_manager,
                    }),
                    XEvent::EnterNotify(event) => self.handlers.on_enter_notify(EventContext {
                        event,
                        conn: self.conn.clone(),
                        keyboard: &self.keyboard,
                        config: self.config.clone(),
                        clients: self.clients.clone(),
                        atoms: &self.atoms,
                        decorator: &self.decorator,
                        layout_manager: &self.layout_manager,
                    }),
                    XEvent::UnmapNotify(_) => {}
                    XEvent::PropertyNotify(_) => {}
                    XEvent::ConfigureRequest(_) => todo!(),
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

        conn.check_request(conn.send_request_checked(&ChangeProperty {
            window: root,
            mode: x::PropMode::Replace,
            r#type: xcb::x::ATOM_STRING,
            data: b"LuckyWM",
            property: xcb::x::ATOM_WM_NAME,
        }))
        .expect("failed to set window manager name");

        root
    }
}

pub enum XEvent {
    KeyPress(xcb::x::KeyPressEvent),
    MapRequest(xcb::x::MapRequestEvent),
    MapNotify(xcb::x::MapNotifyEvent),
    DestroyNotify(xcb::x::DestroyNotifyEvent),
    EnterNotify(xcb::x::EnterNotifyEvent),
    PropertyNotify(xcb::x::PropertyNotifyEvent),
    ConfigureRequest(xcb::x::ConfigureRequestEvent),
    UnmapNotify(xcb::x::UnmapNotifyEvent),
}
