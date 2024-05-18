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

    pub fn clients(&self) -> &HashMap<xcb::x::Window, Client> {
        &self.clients
    }

    pub fn clients_mut(&mut self) -> &mut HashMap<xcb::x::Window, Client> {
        &mut self.clients
    }

    pub fn screens(&self) -> &[Screen] {
        &self.screens
    }

    pub fn screens_mut(&mut self) -> &mut [Screen] {
        &mut self.screens
    }

    pub fn screen(&self, index: usize) -> &Screen {
        assert!(
            self.screens.len().gt(&index),
            "attempted to access an out of bounds screen"
        );
        &self.screens[index]
    }

    pub fn screen_mut(&mut self, index: usize) -> &mut Screen {
        assert!(
            self.screens.len().gt(&index),
            "attempted to access an out of bounds screen"
        );
        &mut self.screens[index]
    }

    pub fn active_screen_idx(&self) -> usize {
        self.active_screen
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
                workspace: self.screens[self.active_screen].active_workspace().id(),
            },
        );
        tracing::debug!("inserting client {frame:?} on clients");
        tracing::debug!("{:#?}", self.screens);

        let screen = &mut self.screens[self.active_screen];
        let workspace = screen.active_workspace_mut();
        workspace.new_client(frame);
        tracing::debug!("{:#?}", workspace);

        if self.config.borrow().focus_new_clients() || workspace.clients().len().eq(&1) {
            workspace.set_focused_client(Some(frame));
        }
    }

    /// Directly focus a client on any of the screens;
    ///
    /// This is mainly used together with `focus_follow_mouse` configuration
    /// which allows for windows to be focused in a non-linear maner
    ///
    /// Window here is the window that triggered the "EnterNotify" event, which
    /// can be a frame or a client window, so we have to check both
    pub fn focus_client(&mut self, window: xcb::x::Window) {
        match self
            .clients
            .values()
            .find(|&&client| client.window.eq(&window) || client.frame.eq(&window))
        {
            Some(client) => {
                tracing::debug!("focusing client: {client:?}");
                self.screens.iter().for_each(|screen| {
                    let workspace = screen.active_workspace_mut();
                    workspace
                        .clients()
                        .contains(&client.frame)
                        .then(|| workspace.set_focused_client(Some(client.frame)));
                });
            }
            None => tracing::error!("tried to select a client that was not on our list"),
        }
    }

    pub fn get_focused_client(&self) -> Option<&Client> {
        if let Some(index) = self.screens[self.active_screen].focused_client() {
            return self.clients.get(&index);
        }
        None
    }

    pub fn close_focused_client(&mut self) -> anyhow::Result<Option<Client>> {
        let active_screen = &mut self.screens[self.active_screen];
        if let Some(frame) = active_screen.focused_client() {
            let workspace = active_screen.active_workspace_mut();
            workspace.remove_client(frame);
            workspace.set_focused_client(workspace.clients().first().copied());
            return Ok(self.clients.remove(&frame));
        }
        Ok(None)
    }

    pub fn get_visible_screen_clients(&self, screen: &Screen) -> Vec<&Client> {
        screen
            .active_workspace()
            .clients()
            .iter()
            .map(|frame| {
                self.clients
                    .get(frame)
                    .expect("we tried to index into an non-existing frame.")
            })
            .collect::<Vec<&Client>>()
    }
}
