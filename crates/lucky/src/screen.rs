use config::Config;
use std::{cell::RefCell, rc::Rc};

use crate::screen_manager::Position;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Client {
    pub frame: xcb::x::Window,
    pub window: xcb::x::Window,
    pub workspace: u8,
    pub visible: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ReservedClient {
    pub window: xcb::x::Window,
    pub show_on_all_workspaces: bool,
    pub workspace: u8,
    pub position: Position,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum WorkspaceLayout {
    #[default]
    Tall,
}

#[derive(Debug, PartialEq)]
pub struct Workspace {
    id: u8,
    layout: WorkspaceLayout,
    clients: Vec<xcb::x::Window>,
    focused_client: Option<xcb::x::Window>,
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

    pub fn layout(&self) -> &WorkspaceLayout {
        &self.layout
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn new_client(&mut self, client: xcb::x::Window) {
        self.clients.push(client)
    }

    pub fn clients(&self) -> &[xcb::x::Window] {
        &self.clients
    }

    pub fn clients_mut(&mut self) -> &mut Vec<xcb::x::Window> {
        &mut self.clients
    }

    pub fn set_focused_client(&mut self, client: Option<xcb::x::Window>) {
        self.focused_client = client
    }

    pub fn remove_client(&mut self, client: xcb::x::Window) {
        self.clients.retain(|i| i.ne(&client));
        self.focused_client
            .is_some_and(|other| client.eq(&other))
            .then(|| self.focused_client = None);
    }
}

#[derive(Debug)]
pub struct Screen {
    position: Position,
    active_workspace: u8,
    workspaces: Vec<Workspace>,
    reserved_clients: Vec<ReservedClient>,
    reserved_left_area: u32,
    reserved_bottom_area: u32,
    reserved_top_area: u32,
    reserved_right_area: u32,
}

impl Screen {
    pub fn new(config: &Rc<RefCell<Config>>, position: Position) -> Self {
        Screen {
            position,
            active_workspace: 0,
            reserved_left_area: 0,
            reserved_bottom_area: 0,
            reserved_top_area: 0,
            reserved_right_area: 0,
            reserved_clients: Vec::default(),
            workspaces: (0..config.borrow().workspaces())
                .map(Workspace::new)
                .collect(),
        }
    }

    pub fn reserved_left_area(&self) -> u32 {
        self.reserved_left_area
    }

    pub fn reserved_bottom_area(&self) -> u32 {
        self.reserved_bottom_area
    }

    pub fn reserved_top_area(&self) -> u32 {
        self.reserved_top_area
    }

    pub fn reserved_right_area(&self) -> u32 {
        self.reserved_right_area
    }

    pub fn reserved_clients(&self) -> &[ReservedClient] {
        &self.reserved_clients
    }

    pub fn focused_client(&self) -> Option<xcb::x::Window> {
        self.workspaces[self.active_workspace as usize].focused_client
    }

    pub fn workspaces(&mut self) -> &[Workspace] {
        &self.workspaces
    }

    pub fn workspaces_mut(&mut self) -> &mut [Workspace] {
        &mut self.workspaces
    }

    pub fn active_workspace(&self) -> &Workspace {
        &self.workspaces[self.active_workspace as usize]
    }

    pub fn active_workspace_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.active_workspace as usize]
    }

    pub fn active_workspace_id(&self) -> usize {
        self.active_workspace as usize
    }

    pub fn set_active_workspace(&mut self, workspace: u8) {
        self.active_workspace = workspace;
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn add_left_reserved_area(&mut self, amount: u32) {
        self.reserved_left_area += amount;
    }

    pub fn add_bottom_reserved_area(&mut self, amount: u32) {
        self.reserved_bottom_area += amount;
    }

    pub fn add_top_reserved_area(&mut self, amount: u32) {
        self.reserved_top_area += amount;
    }

    pub fn add_right_reserved_area(&mut self, amount: u32) {
        self.reserved_right_area += amount;
    }

    pub fn add_reserved_client(&mut self, reserved_client: ReservedClient) {
        self.reserved_clients.push(reserved_client);
    }

    pub fn get_available_area(&self) -> Position {
        let x = self.position.x + self.reserved_left_area as i32;
        let y = self.position.y + self.reserved_top_area as i32;
        let width = self.position.width - self.reserved_left_area - self.reserved_right_area;
        let height = self.position.height - self.reserved_top_area - self.reserved_bottom_area;
        Position::new(x, y, width, height)
    }
}
