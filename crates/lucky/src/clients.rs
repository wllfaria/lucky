use crate::event::EventContext;
use config::AvailableActions;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use xcb::Xid;

#[derive(Debug, PartialEq)]
pub struct Client {
    pub window: xcb::x::Window,
    pub workspace: u8,
    pub visible: bool,
}

pub struct Clients {
    conn: Arc<xcb::Connection>,
    pub active_workspace: u8,
    pub open_clients: VecDeque<Client>,
    pub active_windows: HashMap<u8, Option<xcb::x::Window>>,
}

impl Clients {
    pub fn new(conn: Arc<xcb::Connection>) -> Self {
        Clients {
            conn,
            active_workspace: 1,
            open_clients: VecDeque::new(),
            active_windows: HashMap::default(),
        }
    }

    pub fn create(&mut self, window: xcb::x::Window) -> anyhow::Result<()> {
        self.open_clients.push_front(Client {
            window,
            visible: true,
            workspace: self.active_workspace,
        });

        self.active_windows
            .insert(self.active_workspace, Some(window));

        Ok(())
    }

    fn get_active_client(&mut self) -> Option<xcb::x::Window> {
        self.active_windows
            .entry(self.active_workspace)
            .or_insert(None)
            .to_owned()
    }

    pub fn set_active_client(&mut self, window: Option<xcb::x::Window>) -> anyhow::Result<()> {
        if let Some(window) = window {
            self.active_windows
                .insert(self.active_workspace, Some(window));
        }

        Ok(())
    }

    pub fn close_active_client(
        &mut self,
        context: &EventContext<xcb::x::KeyPressEvent>,
    ) -> anyhow::Result<Option<xcb::x::Window>> {
        if let Some(window) = self.get_active_client() {
            self.open_clients.retain(|client| client.window.ne(&window));
            let active_client = self.open_clients.iter().next().map(|c| c.window);
            self.set_active_client(active_client)?;
            return Ok(Some(window));
        }

        Ok(None)
    }
}
