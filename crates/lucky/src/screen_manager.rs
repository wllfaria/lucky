use crate::ewmh::*;
use crate::position::Position;
use crate::screen::{Client, Screen};
use config::Config;
use std::{cell::RefCell, collections::HashMap, ops::Add, rc::Rc};

use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Down,
    Up,
    Right,
}

pub struct ScreenManager {
    screens: Vec<Screen>,
    clients: HashMap<xcb::x::Window, Client>,
    root: xcb::x::Window,
    active_screen: usize,
    config: Rc<RefCell<Config>>,
    showing_desktop_status: EwmhShowingDesktop,
}

impl ScreenManager {
    pub fn new(screens: Vec<Screen>, config: Rc<RefCell<Config>>, root: xcb::x::Window) -> Self {
        ScreenManager {
            active_screen: 0,
            root,
            clients: HashMap::new(),
            showing_desktop_status: EwmhShowingDesktop::Hide,
            screens,
            config,
        }
    }

    pub fn root(&self) -> xcb::x::Window {
        self.root
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

    /// finds which screen should be selected when moving/focusing a client
    /// in a specific direction.
    ///
    /// since every screen has a position, we find possible candidates that
    /// are further to the selected direction, and find based on the distance
    /// of each candidates x,y starting point from the x,y starting point of
    /// the current screen which is the closest monitor and therefore which
    /// should be selected.
    pub fn get_relative_screen_idx(&self, direction: Direction) -> Option<usize> {
        let active_screen = &self.screens[self.active_screen];
        let curr_position = active_screen.position();

        let next_screen = self
            .screens
            .iter()
            .enumerate()
            .map(|(idx, screen)| (idx, screen.position()))
            .filter(|(_, position)| match direction {
                Direction::Left => position.right() <= curr_position.left(),
                Direction::Down => position.top() >= curr_position.bottom(),
                Direction::Up => position.bottom() <= curr_position.top(),
                Direction::Right => position.left() >= curr_position.right(),
            })
            .min_by_key(|(_, position)| {
                (euclidean_distance(
                    position.left(),
                    position.top(),
                    curr_position.left(),
                    curr_position.top(),
                ) * 1000.0) as i32
            });

        next_screen.map(|(idx, _)| idx)
    }

    pub fn set_active_screen(&mut self, active_screen_idx: usize) {
        self.active_screen = active_screen_idx
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

        let screen = &mut self.screens[self.active_screen];
        let workspace = screen.active_workspace_mut();
        workspace.new_client(frame);

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
            .find(|client| client.window.eq(&window) || client.frame.eq(&window))
        {
            Some(client) => {
                self.screens.iter_mut().for_each(|screen| {
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

    /// when the user cursor mover from a monitor to another we need to
    /// switch focus to that monitor,
    pub fn maybe_switch_screen(
        &mut self,
        pointer: xcb::x::QueryPointerReply,
        conn: &Arc<xcb::Connection>,
        atoms: &crate::atoms::Atoms,
    ) {
        let (cursor_x, cursor_y) = (pointer.root_x(), pointer.root_y());

        for (idx, screen) in self.screens.iter().enumerate() {
            if is_cursor_inside(cursor_x.into(), cursor_y.into(), screen.position()) {
                self.active_screen = idx;
                self.update_atoms(atoms, conn);
            }
        }
    }

    /// update every EWMH necessary atoms regarding the current context
    /// of the active screen.
    ///
    /// although techinically some of those atoms could be updated only
    /// when changed, this is a fair tradeoff as the performance impact of
    /// this is negligible
    pub fn update_atoms(&self, atoms: &crate::atoms::Atoms, conn: &Arc<xcb::Connection>) {
        let screen = &self.screens[self.active_screen];
        ewmh_set_desktop_viewport(conn, self.root, &self.screens, atoms).ok();
        ewmh_set_number_of_desktops(conn, self.root, screen, atoms).ok();
        ewmh_set_current_desktop(conn, self.root, screen, atoms).ok();
        ewmh_set_desktop_names(conn, self.root, screen, atoms).ok();
        ewmh_set_wm_desktop(conn, screen, &self.clients, atoms).ok();
        ewmh_set_client_list(conn, self.root, self.clients.keys(), atoms).ok();
        ewmh_set_client_list_stacking(conn, self.root, self.clients.keys(), atoms).ok();
        ewmh_set_showing_desktop(conn, self.root, atoms, self.showing_desktop_status).ok();
    }
}

fn is_cursor_inside(x: i32, y: i32, position: &Position) -> bool {
    x.ge(&position.x)
        && x.lt(&position.x.add(position.width as i32))
        && y.ge(&position.y)
        && y.lt(&position.y.add(position.height as i32))
}

/// calculates distance between two cartesian points.
///
/// the formula is:
/// d=√((x2 – x1)² + (y2 – y1)²)
/// see: https://en.wikipedia.org/wiki/Euclidean_distance
fn euclidean_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> f64 {
    (((x2 - x1).pow(2) + (y2 - y1).pow(2)) as f64).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use xcb::XidNew;

    #[test]
    fn get_screen_to_left() {
        let positions = vec![
            // active screen
            Position::new(1920, 0, 1920, 1080),
            // screen to the left of active screen
            Position::new(0, 0, 1920, 1080),
            // screen to the right of active screen
            Position::new(3840, 0, 1920, 1080),
            // screen to the bottom of active screen
            Position::new(1920, 1080, 1920, 1080),
            // screen to the bottom right of active screen
            Position::new(3840, 1080, 1920, 1080),
            // screen to the bottom left of active screen
            Position::new(0, 1080, 1920, 1080),
            // screen to the top of active screen
            Position::new(1920, -1080, 1920, 1080),
            // screen to the top right of active screen
            Position::new(3840, -1080, 1920, 1080),
            // screen to the top left of active screen
            Position::new(0, -1080, 1920, 1080),
        ];
        let config = Rc::new(RefCell::new(Config::default()));
        let screens = positions
            .clone()
            .into_iter()
            .map(|pos| Screen::new(&config, pos))
            .collect();
        let root = unsafe { xcb::x::Window::new(0) };
        let sm = ScreenManager::new(screens, config.clone(), root);

        let idx = sm.get_relative_screen_idx(Direction::Left).unwrap();
        let expected = Position::new(0, 0, 1920, 1080);
        assert!(sm.screens[idx].position() == &expected);

        let idx = sm.get_relative_screen_idx(Direction::Down).unwrap();
        let expected = Position::new(1920, 1080, 1920, 1080);
        assert!(sm.screens[idx].position() == &expected);

        let idx = sm.get_relative_screen_idx(Direction::Up).unwrap();
        let expected = Position::new(1920, -1080, 1920, 1080);
        assert!(sm.screens[idx].position() == &expected);

        let idx = sm.get_relative_screen_idx(Direction::Right).unwrap();
        let expected = Position::new(3840, 0, 1920, 1080);
        assert!(sm.screens[idx].position() == &expected);

        let root = unsafe { xcb::x::Window::new(0) };
        let sm = ScreenManager::new(
            vec![Screen::new(&config, Position::new(1920, 0, 1920, 1080))],
            config,
            root,
        );
        let idx = sm.get_relative_screen_idx(Direction::Up);
        assert!(idx.is_none());
    }
}
