use crate::screen::{Client, Screen};
use config::Config;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Position {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Position {
            x,
            y,
            width,
            height,
        }
    }
}

pub struct ScreenManager {
    screens: Vec<Screen>,
    clients: HashMap<xcb::x::Window, Client>,
    active_screen: usize,
    config: Rc<RefCell<Config>>,
}

impl ScreenManager {
    pub fn new(screen_positions: Vec<Position>, config: Rc<RefCell<Config>>) -> Self {
        ScreenManager {
            active_screen: 0,
            clients: HashMap::new(),
            screens: screen_positions
                .into_iter()
                .map(|pos| Screen::new(&config, pos))
                .collect(),
            config,
        }
    }

    pub fn screens(&self) -> &[Screen] {
        &self.screens
    }

    /// Creates a new client on the active screen and active workspace on given screen
    ///
    /// When `focus_new_clients` is true on configuration, we also set the focus to the newly
    /// created client
    ///
    /// even when `focus_new_clients` is false, if the client is the only client on the workspace
    /// we focus it
    pub fn create_client(&mut self, frame: xcb::x::Window, window: xcb::x::Window) {
        self.clients.insert(
            frame,
            Client {
                frame,
                window,
                visible: true,
                workspace: self.screens[self.active_screen].active_workspace,
            },
        );

        let screen = &mut self.screens[self.active_screen];
        let workspace = &mut screen.workspaces[screen.active_workspace as usize];
        workspace.clients.push(frame);

        if self.config.borrow().focus_new_clients() || workspace.clients.len().eq(&1) {
            workspace.focused_client = Some(frame);
        }
    }

    pub fn get_focused_client(&self) -> Option<&Client> {
        if let Some(index) = self.screens[self.active_screen].get_active_client_index() {
            return self.clients.get(&index);
        }
        None
    }

    pub fn close_focused_client(&mut self) -> anyhow::Result<Option<Client>> {
        let active_screen = &mut self.screens[self.active_screen];
        if let Some(frame) = active_screen.get_active_client_index() {
            let workspace = &mut active_screen.workspaces[active_screen.active_workspace as usize];
            workspace.clients.retain(|i| !i.eq(&frame));
            workspace.focused_client = workspace.clients.first().copied();
            return Ok(self.clients.remove(&frame));
        }
        Ok(None)
    }

    pub fn get_visible_screen_clients(&self, screen: &Screen) -> Vec<&Client> {
        screen.workspaces[screen.active_workspace as usize]
            .clients
            .iter()
            .map(|frame| {
                self.clients
                    .get(frame)
                    .expect("we tried to index into an non-existing frame.")
            })
            .collect::<Vec<&Client>>()
    }
}
