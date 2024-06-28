use crate::macros::*;
use config::Config;
use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::screen::Client;

pub struct Decorator {
    config: Rc<RefCell<Config>>,
    conn: Arc<xcb::Connection>,
}

impl Decorator {
    pub fn new(conn: Arc<xcb::Connection>, config: Rc<RefCell<Config>>) -> Self {
        Decorator { conn, config }
    }

    pub fn decorate_client(&self, client: xcb::x::Window) -> anyhow::Result<xcb::x::Window> {
        let frame = self.create_frame()?;
        xcb_reparent_win!(self.conn, client, frame)?;
        Ok(frame)
    }

    fn create_frame(&self) -> anyhow::Result<xcb::x::Window> {
        let frame = self.conn.generate_id();
        let root = self
            .conn
            .get_setup()
            .roots()
            .next()
            .expect("should have at least one screen to manage")
            .root();

        self.conn
            .check_request(self.conn.send_request_checked(&xcb::x::CreateWindow {
                depth: xcb::x::COPY_FROM_PARENT as u8,
                wid: frame,
                parent: root,
                x: 0,
                y: 0,
                width: 1,
                height: 1,
                border_width: self.config.borrow().border_width(),
                class: xcb::x::WindowClass::InputOutput,
                visual: xcb::x::COPY_FROM_PARENT,
                value_list: &[
                    xcb::x::Cw::BackPixel(0),
                    xcb::x::Cw::BorderPixel(self.config.borrow().border_color()),
                    xcb::x::Cw::EventMask(
                        xcb::x::EventMask::EXPOSURE
                            | xcb::x::EventMask::BUTTON_PRESS
                            | xcb::x::EventMask::BUTTON_RELEASE
                            | xcb::x::EventMask::POINTER_MOTION
                            | xcb::x::EventMask::ENTER_WINDOW
                            | xcb::x::EventMask::LEAVE_WINDOW,
                    ),
                ],
            }))?;

        Ok(frame)
    }

    pub fn unfocus_client(&self, client: &Client) -> anyhow::Result<()> {
        xcb_change_attr!(
            self.conn,
            client.frame,
            &[xcb::x::Cw::BorderPixel(self.config.borrow().border_color())]
        );
        Ok(())
    }

    pub fn focus_client(&self, client: &Client) -> anyhow::Result<()> {
        xcb_change_attr!(
            self.conn,
            client.frame,
            &[xcb::x::Cw::BorderPixel(
                self.config.borrow().active_border_color()
            )]
        );
        xcb_input_focus!(self.conn, client.window);

        Ok(())
    }
}
