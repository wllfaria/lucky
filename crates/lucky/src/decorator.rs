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
        self.reparent_client(frame, client)?;
        Ok(frame)
    }

    fn reparent_client(&self, frame: xcb::x::Window, client: xcb::x::Window) -> anyhow::Result<()> {
        self.conn
            .check_request(self.conn.send_request_checked(&xcb::x::ReparentWindow {
                window: client,
                parent: frame,
                x: 0,
                y: 0,
            }))?;

        Ok(())
    }

    fn create_frame(&self) -> anyhow::Result<xcb::x::Window> {
        let frame = self.conn.generate_id();

        let root = self
            .conn
            .get_setup()
            .roots()
            .next()
            .expect("should have at least one screen to manage");

        self.conn
            .check_request(self.conn.send_request_checked(&xcb::x::CreateWindow {
                depth: xcb::x::COPY_FROM_PARENT as u8,
                wid: frame,
                parent: root.root(),
                x: 0,
                y: 0,
                width: 1,
                height: 1,
                border_width: self.config.borrow().border_width(),
                class: xcb::x::WindowClass::InputOutput,
                visual: root.root_visual(),
                value_list: &[
                    xcb::x::Cw::BackPixel(root.black_pixel()),
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
        self.conn.send_request(&xcb::x::ChangeWindowAttributes {
            window: client.frame,
            value_list: &[xcb::x::Cw::BorderPixel(self.config.borrow().border_color())],
        });
        Ok(())
    }

    pub fn focus_client(&self, client: &Client) -> anyhow::Result<()> {
        self.conn.send_request(&xcb::x::ChangeWindowAttributes {
            window: client.frame,
            value_list: &[xcb::x::Cw::BorderPixel(
                self.config.borrow().active_border_color(),
            )],
        });
        self.conn.send_request(&xcb::x::SetInputFocus {
            time: xcb::x::CURRENT_TIME,
            focus: client.window,
            revert_to: xcb::x::InputFocus::Parent,
        });

        Ok(())
    }
}
