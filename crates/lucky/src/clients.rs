use std::{
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

use config::Config;

#[derive(Debug, PartialEq, Clone)]
pub struct Client {
    pub frame: xcb::x::Window,
    pub window: xcb::x::Window,
    pub workspace: u8,
    pub visible: bool,
}

#[derive(Default, Debug, Clone)]
pub enum WorkspaceLayout {
    #[default]
    Master,
}

#[derive(Debug)]
pub struct Workspace {
    pub id: u8,
    pub layout: WorkspaceLayout,
}

impl Workspace {
    pub fn new(id: u8) -> Self {
        Workspace {
            id,
            layout: Default::default(),
        }
    }
}

pub struct Clients {
    pub active_workspace: u8,
    pub open_clients: VecDeque<Client>,
    pub frames: HashSet<xcb::x::Window>,
    pub active_clients: HashMap<u8, Option<usize>>,
    pub workspaces: Vec<Workspace>,
}

impl Clients {
    pub fn new(config: &Rc<Config>) -> Self {
        Clients {
            active_workspace: 1,
            open_clients: VecDeque::new(),
            active_clients: HashMap::default(),
            frames: HashSet::default(),
            workspaces: (0..config.workspaces()).map(Workspace::new).collect(),
        }
    }

    pub fn create(&mut self, frame: xcb::x::Window, window: xcb::x::Window) -> anyhow::Result<()> {
        let index = self.open_clients.len();
        self.open_clients.push_back(Client {
            frame,
            window,
            visible: true,
            workspace: self.active_workspace,
        });
        self.frames.insert(frame);

        self.active_clients
            .insert(self.active_workspace, Some(index));

        Ok(())
    }

    fn get_active_client(&mut self) -> Option<&Client> {
        let active_window_index = self
            .active_clients
            .entry(self.active_workspace)
            .or_insert(None);

        if let Some(index) = active_window_index {
            Some(&self.open_clients[*index])
        } else {
            None
        }
    }

    pub fn set_active_client(&mut self) -> anyhow::Result<()> {
        let client_index = self
            .open_clients
            .iter()
            .position(|client| client.workspace.eq(&self.active_workspace) && client.visible);

        if let Some(index) = client_index {
            self.active_clients
                .insert(self.active_workspace, Some(index));
        }

        Ok(())
    }

    pub fn close_active_client(&mut self) -> anyhow::Result<Option<Client>> {
        if let Some(client) = self.get_active_client() {
            let client = client.clone();
            self.open_clients
                .retain(|client| client.window.ne(&client.window));
            self.set_active_client()?;
            return Ok(Some(client.clone()));
        }

        Ok(None)
    }

    pub fn get_active_workspace(&self) -> &Workspace {
        &self.workspaces[self.active_workspace as usize]
    }

    pub fn get_client_from_window(&self, window: &xcb::x::Window) -> Option<&Client> {
        self.open_clients
            .iter()
            .find(|client| client.window.eq(window))
    }
}
