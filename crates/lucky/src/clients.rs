use config::Config;
use std::rc::Rc;

use crate::screen_manager::Position;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Client {
    pub frame: xcb::x::Window,
    pub window: xcb::x::Window,
    pub workspace: u8,
    pub visible: bool,
}

#[derive(Default, Debug, Clone)]
pub enum WorkspaceLayout {
    #[default]
    Tall,
}

#[derive(Debug)]
pub struct Workspace {
    pub id: u8,
    pub layout: WorkspaceLayout,
    pub clients: Vec<xcb::x::Window>,
    pub focused_client: Option<xcb::x::Window>,
}

impl Workspace {
    pub fn new(id: u8) -> Self {
        Workspace {
            id,
            layout: Default::default(),
            clients: vec![],
            focused_client: None,
        }
    }
}

#[derive(Debug)]
pub struct Screen {
    pub position: Position,
    pub active_workspace: u8,
    pub workspaces: Vec<Workspace>,
}

impl Screen {
    pub fn new(config: &Rc<Config>, position: Position) -> Self {
        Screen {
            position,
            active_workspace: 1,
            workspaces: (0..config.workspaces()).map(Workspace::new).collect(),
        }
    }

    pub fn get_active_client_index(&self) -> Option<xcb::x::Window> {
        self.workspaces[self.active_workspace as usize].focused_client
    }

    pub fn set_active_client(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn get_active_workspace(&self) -> &Workspace {
        &self.workspaces[self.active_workspace as usize]
    }
}
