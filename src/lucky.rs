use std::sync::Arc;

use xcb::x::{self, ChangeWindowAttributes};

pub struct Lucky {
    conn: Arc<xcb::Connection>,
    cursor: xcb::x::Cursor,
}

impl Lucky {
    pub fn new() -> Self {
        let (conn, _) = xcb::Connection::connect_with_extensions(None, &[xcb::Extension::Xkb], &[]).expect("failed to initialize connection to the X server. Check the DISPLAY environment variable");
        let font = conn.generate_id();

        let cookie = conn.send_request_checked(&xcb::x::OpenFont {
            fid: font,
            name: b"cursor",
        });

        conn.check_request(cookie)
            .expect("failed to open cursor font");

        let cursor = conn.generate_id();

        let cookie = conn.send_request_checked(&xcb::x::CreateGlyphCursor {
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
        });

        conn.check_request(cookie)
            .expect("failed to create a cursor");

        Self {
            conn: Arc::new(conn),
            cursor,
        }
    }

    pub fn start(self) {
        tracing::info!("Lucky started successfully");
        let screen = self
            .conn
            .get_setup()
            .roots()
            .next()
            .expect("should have at least a single window");
        let root = screen.root();

        let cookies = self.conn.send_request_checked(&ChangeWindowAttributes {
            window: root,
            value_list: &[x::Cw::Cursor(self.cursor)],
        });

        self.conn
            .check_request(cookies)
            .expect("failed to attach cursor to the  window");

        let cookies = self.conn.send_request_checked(&ChangeWindowAttributes {
            window: root,
            value_list: &[x::Cw::EventMask(
                x::EventMask::SUBSTRUCTURE_REDIRECT | x::EventMask::SUBSTRUCTURE_NOTIFY,
            )],
        });

        self.conn
            .check_request(cookies)
            .expect("failed to subscribe for substructure redirection");

        loop {
            if let Ok(event) = self.conn.wait_for_event() {
                tokio::spawn(Self::handle_event(self.conn.clone(), event));
            }
            self.conn.flush().unwrap();
        }
    }

    #[tracing::instrument(skip_all, name = "handle_event")]
    async fn handle_event(_conn: Arc<xcb::Connection>, event: xcb::Event) {
        tracing::debug!("event received");
        match event {
            xcb::Event::X(xcb::x::Event::MapNotify(e)) => {
                tracing::trace!("handling keypress {:?}", e);
            }
            xcb::Event::X(xcb::x::Event::MapRequest(e)) => {
                tracing::trace!("handling keypress {:?}", e);
            }
            _ => (),
        }
    }
}
