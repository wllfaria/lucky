use config::Config;
use std::sync::{mpsc::channel, Arc, Mutex};
use xcb::x::{self, ChangeWindowAttributes, ConfigureWindow, MapWindow};

use crate::{cursor::Cursor, keyboard::Keyboard, keys::Keysym};

pub struct Lucky {
    conn: Arc<xcb::Connection>,
    keyboard: Keyboard,
    config: Arc<Mutex<Config>>,
}

impl Lucky {
    pub fn new(config: Config) -> Self {
        let (conn, _) = xcb::Connection::connect(None).expect("failed to initialize self.conn to the X server. Check the DISPLAY environment variable");

        let conn = Arc::new(conn);
        let cursor = Cursor::new(conn.clone());

        let screen = conn
            .get_setup()
            .roots()
            .next()
            .expect("should have at least a single window");
        let root = screen.root();

        let keyboard = Keyboard::new(conn.clone(), root, &config);

        conn.check_request(conn.send_request_checked(&ChangeWindowAttributes {
            window: root,
            value_list: &[
                x::Cw::EventMask(
                    x::EventMask::SUBSTRUCTURE_REDIRECT | x::EventMask::SUBSTRUCTURE_NOTIFY,
                ),
                x::Cw::Cursor(cursor.cursor),
            ],
        }))
        .expect("failed to subscribe for substructure redirection");

        Self {
            conn,
            keyboard,
            config,
        }
    }

    pub fn run(self) {
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
                    xcb::Event::X(xcb::x::Event::ConfigureRequest(_)) => todo!(),
                    xcb::Event::X(xcb::x::Event::MapRequest(e)) => {
                        if sender.send(XEvent::MapRequest(e)).is_err() {
                            tracing::debug!("failed to send event through channel");
                            std::process::abort();
                        }
                    }
                    xcb::Event::X(xcb::x::Event::PropertyNotify(_)) => todo!(),
                    xcb::Event::X(xcb::x::Event::EnterNotify(_)) => todo!(),
                    xcb::Event::X(xcb::x::Event::UnmapNotify(_)) => {}
                    xcb::Event::X(xcb::x::Event::DestroyNotify(_)) => todo!(),
                    _ => (),
                };
            };
            conn.flush().unwrap();
        });

        loop {
            if let Ok(event) = receiver.recv() {
                match event {
                    XEvent::KeyPress(e) => {
                        if let Ok(keysym) =
                            Keysym::try_from(self.keyboard.state.key_get_one_sym(e.detail().into()))
                        {
                            tracing::debug!("{keysym} {}", e.detail());
                            tracing::debug!("{:?}", e.state());
                        }
                    }
                    XEvent::ConfigureRequest(_) => todo!(),
                    XEvent::MapRequest(e) => {
                        let root_win = self
                            .conn
                            .get_setup()
                            .roots()
                            .next()
                            .expect("should have at least a single window");

                        let window = e.window();

                        let cookies = self.conn.send_request_checked(&ConfigureWindow {
                            window: e.window(),
                            value_list: &[
                                x::ConfigWindow::X(0),
                                x::ConfigWindow::Y(0),
                                x::ConfigWindow::Width(root_win.width_in_pixels().into()),
                                x::ConfigWindow::Height(root_win.height_in_pixels().into()),
                            ],
                        });
                        self.conn.check_request(cookies).unwrap();
                        let cookies = self.conn.send_request_checked(&MapWindow { window });
                        self.conn.check_request(cookies).unwrap();
                    }
                    XEvent::PropertyNotify(_) => {}
                    XEvent::EnterNotify(_) => {}
                    XEvent::UnmapNotify(_) => {}
                    XEvent::DestroyNotify(_) => {}
                }
            }
        }
    }
}

pub enum XEvent {
    KeyPress(xcb::x::KeyPressEvent),
    ConfigureRequest(xcb::x::ConfigureRequestEvent),
    MapRequest(xcb::x::MapRequestEvent),
    PropertyNotify(xcb::x::PropertyNotifyEvent),
    EnterNotify(xcb::x::EnterNotifyEvent),
    UnmapNotify(xcb::x::UnmapNotifyEvent),
    DestroyNotify(xcb::x::DestroyNotifyEvent),
}
