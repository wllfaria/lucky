use std::sync::{mpsc::channel, Arc};
use xcb::x::{self, ChangeWindowAttributes};

use crate::{cursor::Cursor, keyboard::Keyboard, keys::Keysym};

pub struct Lucky {
    conn: Arc<xcb::Connection>,
    keyboard: Keyboard,
}

impl Lucky {
    pub fn new() -> Self {
        let (conn, _) = xcb::Connection::connect(None).expect("failed to initialize self.conn to the X server. Check the DISPLAY environment variable");

        let conn = Arc::new(conn);
        let cursor = Cursor::new(conn.clone());

        let screen = conn
            .get_setup()
            .roots()
            .next()
            .expect("should have at least a single window");
        let root = screen.root();

        let keyboard = Keyboard::new(conn.clone(), root);

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

        Self { conn, keyboard }
    }

    pub fn run(self) {
        let (sender, receiver) = channel::<XEvent>();

        std::thread::spawn(move || loop {
            if let Ok(event) = self.conn.wait_for_event() {
                match event {
                    xcb::Event::X(xcb::x::Event::KeyPress(e)) => {
                        if sender.send(XEvent::KeyPress(e)).is_err() {
                            tracing::debug!("failed to send event through channel");
                            std::process::abort();
                        }
                    }
                    xcb::Event::X(xcb::x::Event::ConfigureRequest(_)) => todo!(),
                    xcb::Event::X(xcb::x::Event::MapRequest(_)) => todo!(),
                    xcb::Event::X(xcb::x::Event::PropertyNotify(_)) => todo!(),
                    xcb::Event::X(xcb::x::Event::EnterNotify(_)) => todo!(),
                    xcb::Event::X(xcb::x::Event::UnmapNotify(_)) => todo!(),
                    xcb::Event::X(xcb::x::Event::DestroyNotify(_)) => todo!(),
                    _ => (),
                };
            };
            self.conn.flush().unwrap();
        });

        loop {
            if let Ok(event) = receiver.recv() {
                match event {
                    XEvent::KeyPress(e) => {
                        if let Ok(keysym) =
                            Keysym::try_from(self.keyboard.state.key_get_one_sym(e.detail().into()))
                        {
                            tracing::debug!("{keysym}");
                        }
                    }
                    XEvent::ConfigureRequest(_) => todo!(),
                    XEvent::MapRequest(_) => todo!(),
                    XEvent::PropertyNotify(_) => todo!(),
                    XEvent::EnterNotify(_) => todo!(),
                    XEvent::UnmapNotify(_) => todo!(),
                    XEvent::DestroyNotify(_) => todo!(),
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

// let root_win = conn
//     .get_setup()
//     .roots()
//     .next()
//     .expect("should have at least a single window");
//
// let window = e.window();
//
// let cookies = conn.send_request_checked(&ConfigureWindow {
//     window: e.window(),
//     value_list: &[
//         x::ConfigWindow::X(0),
//         x::ConfigWindow::Y(0),
//         x::ConfigWindow::Width(root_win.width_in_pixels().into()),
//         x::ConfigWindow::Height(root_win.height_in_pixels().into()),
//     ],
// });
// conn.check_request(cookies)?;
// let cookies = conn.send_request_checked(&MapWindow { window });
// conn.check_request(cookies)?;
