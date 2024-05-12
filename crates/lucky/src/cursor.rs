use std::sync::Arc;

pub struct Cursor {
    pub cursor: xcb::x::Cursor,
}

impl Cursor {
    pub fn new(conn: Arc<xcb::Connection>) -> Self {
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

        Self { cursor }
    }
}
