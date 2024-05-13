use config::Config;
use std::{collections::HashMap, rc::Rc, sync::Arc};
use xcb::x::{ConfigureWindow, MapWindow};

#[derive(Debug, PartialEq)]
pub struct Client {
    pub window: xcb::x::Window,
    pub workspace: Option<u8>,
    pub visible: bool,
}

pub struct Clients {
    conn: Arc<xcb::Connection>,
    config: Rc<Config>,
    pub active_workspace: u8,
    pub clients: Vec<Client>,
    pub active_windows: HashMap<u8, Option<xcb::x::Window>>,
}

impl Clients {
    pub fn new(conn: Arc<xcb::Connection>, config: Rc<Config>) -> Self {
        Self {
            conn,
            config,
            active_workspace: 1,
            clients: vec![],
            active_windows: HashMap::default(),
        }
    }

    pub fn create(&mut self, window: xcb::x::Window) {
        self.active_windows
            .insert(self.active_workspace, Some(window));
        self.clients.push(Client {
            window,
            visible: true,
            workspace: Some(self.active_workspace),
        });
    }

    pub fn display(&mut self, window: xcb::x::Window) -> anyhow::Result<()> {
        for client in self.clients.iter() {
            if client.window.eq(&window) && client.visible {
                let root_win = self
                    .conn
                    .get_setup()
                    .roots()
                    .next()
                    .expect("should have at least a single window");
                self.conn
                    .check_request(self.conn.send_request_checked(&ConfigureWindow {
                        window: client.window,
                        value_list: &[
                            xcb::x::ConfigWindow::X(0),
                            xcb::x::ConfigWindow::Y(0),
                            xcb::x::ConfigWindow::Width(root_win.width_in_pixels().into()),
                            xcb::x::ConfigWindow::Height(root_win.height_in_pixels().into()),
                        ],
                    }))?;
                self.conn
                    .check_request(self.conn.send_request_checked(&MapWindow { window }))?;
            }
        }

        Ok(())
    }

    pub fn active_client(&mut self) -> Option<xcb::x::Window> {
        self.active_windows
            .entry(self.active_workspace)
            .or_insert(None)
            .to_owned()
    }
}
