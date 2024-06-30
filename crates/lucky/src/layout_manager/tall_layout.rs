use anyhow::Context;
use config::Config;

use crate::decorator::Decorator;
use crate::position::Position;
use crate::screen::{Client, Screen};
use crate::screen_manager::{Direction, ScreenManager};
use crate::xcb_utils::xcb_map_win;

use std::cell::RefCell;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;
use std::sync::Arc;

pub struct TallLayout {}

impl TallLayout {
    pub fn display_clients(
        conn: &Arc<xcb::Connection>,
        config: &Rc<RefCell<Config>>,
        screen: &Screen,
        clients: Vec<&Client>,
        focused_client: Option<&Client>,
        decorator: &Decorator,
    ) -> anyhow::Result<()> {
        let visible_clients_len = clients.len();
        let available_area = screen.get_available_area();

        let main_width = if visible_clients_len.eq(&1) {
            available_area.width
        } else {
            available_area.width.div(2)
        };

        for client in screen.reserved_clients() {
            Self::configure_window(conn, client.window, client.position.clone());
            conn.send_request(&xcb::x::MapWindow {
                window: client.window,
            });
        }

        for (i, client) in clients.iter().enumerate() {
            decorator
                .unfocus_client(client)
                .context("failed to unfocus client")?;
            match i {
                0 => Self::display_main_client(conn, client, &available_area, main_width, config),
                _ => Self::display_side_client(
                    conn,
                    client,
                    &available_area,
                    i,
                    visible_clients_len,
                    main_width,
                    config,
                ),
            }
        }

        let Some(focused_client) = focused_client else {
            return Ok(());
        };

        clients
            .iter()
            .find(|&&client| client == focused_client)
            .map(|client| decorator.focus_client(client));

        Ok(())
    }

    fn display_main_client(
        conn: &Arc<xcb::Connection>,
        client: &Client,
        available_area: &Position,
        main_width: u32,
        config: &Rc<RefCell<Config>>,
    ) {
        let border_double = config.borrow().border_width().mul(2) as u32;
        let frame_position = Position::new(
            available_area.x,
            available_area.y,
            main_width.sub(border_double),
            available_area.height.sub(border_double),
        );
        let client_position = Position::new(
            0,
            0,
            main_width.sub(config.borrow().border_width() as u32),
            available_area
                .height
                .sub(config.borrow().border_width() as u32),
        );

        Self::configure_window(conn, client.frame, frame_position);
        Self::configure_window(conn, client.window, client_position);

        xcb_map_win!(conn, client.window);
        xcb_map_win!(conn, client.frame);
    }

    fn display_side_client(
        conn: &Arc<xcb::Connection>,
        client: &Client,
        available_area: &Position,
        index: usize,
        total: usize,
        main_width: u32,
        config: &Rc<RefCell<Config>>,
    ) {
        let width = available_area.width.sub(main_width);
        let total_siblings = total.sub(1);
        let height = available_area.height.div_ceil(total_siblings as u32);
        let sibling_index = index.sub(1);
        let border_double = config.borrow().border_width().mul(2) as u32;
        let position_y = height.mul(sibling_index as u32) as i32;

        let height = height.sub(border_double);

        Self::configure_window(
            conn,
            client.frame,
            Position::new(
                available_area.x.add(main_width as i32),
                available_area.y.add(position_y),
                width.sub(border_double),
                height,
            ),
        );
        Self::configure_window(
            conn,
            client.window,
            Position::new(0, 0, width.sub(border_double), height),
        );

        xcb_map_win!(conn, client.window);
        xcb_map_win!(conn, client.frame);
    }

    fn is_first(screen: &mut Screen, client: xcb::x::Window) -> bool {
        screen
            .active_workspace()
            .clients()
            .first()
            .is_some_and(|focused| focused.eq(&client))
    }

    fn is_last(screen: &mut Screen, client: xcb::x::Window) -> bool {
        screen
            .active_workspace()
            .clients()
            .last()
            .is_some_and(|focused| focused.eq(&client))
    }

    fn swap_first(screen: &mut Screen, client: xcb::x::Window) -> anyhow::Result<()> {
        let index = screen
            .active_workspace()
            .clients()
            .iter()
            .position(|c| c.eq(&client))
            .context("workspace clients vector should include selected client")?;

        screen.active_workspace_mut().clients_mut().swap(index, 0);
        Ok(())
    }

    fn swap_prev(screen: &mut Screen, client: xcb::x::Window) -> anyhow::Result<()> {
        let index = screen
            .active_workspace()
            .clients()
            .iter()
            .position(|c| c.eq(&client))
            .context("workspace clients vector should include selected client")?;

        screen
            .active_workspace_mut()
            .clients_mut()
            .swap(index, index.sub(1));

        Ok(())
    }

    fn swap_next(screen: &mut Screen, client: xcb::x::Window) -> anyhow::Result<()> {
        let index = screen
            .active_workspace()
            .clients()
            .iter()
            .position(|c| c.eq(&client))
            .context("workspace clients vector should include selected client")?;

        screen
            .active_workspace_mut()
            .clients_mut()
            .swap(index, index.add(1));

        Ok(())
    }

    fn focus_first(screen: &mut Screen) -> Option<xcb::x::Window> {
        let first_client = screen.active_workspace().clients().first().copied();

        screen
            .active_workspace_mut()
            .set_focused_client(first_client);

        first_client
    }

    fn focus_last(screen: &mut Screen) -> Option<xcb::x::Window> {
        let last_client = screen.active_workspace().clients().last().copied();
        screen
            .active_workspace_mut()
            .set_focused_client(last_client);

        last_client
    }

    fn focus_prev(screen: &mut Screen, client: xcb::x::Window) -> Option<xcb::x::Window> {
        let index = screen
            .active_workspace()
            .clients()
            .iter()
            .position(|c| c.eq(&client))
            .expect("workspace clients vector should include selected client");

        let client = screen
            .active_workspace()
            .clients()
            .get(index.sub(1))
            .copied();

        screen.active_workspace_mut().set_focused_client(client);

        client
    }

    fn focus_next(screen: &mut Screen, client: xcb::x::Window) -> Option<xcb::x::Window> {
        let index = screen
            .active_workspace()
            .clients()
            .iter()
            .position(|c| c.eq(&client))
            .expect("workspace clients vector should include selected client");

        let client = screen
            .active_workspace()
            .clients()
            .get(index.add(1))
            .copied();

        screen.active_workspace_mut().set_focused_client(client);

        client
    }

    /// focus a client in a given direction. Possibly focusing a client on an adjacent
    /// screen, this function will never fail, but might return a few set of different
    /// data, explained below:
    ///
    /// - `None` -> `Screen` has no clients that can be focused
    /// - `Some((None, Some(_)))` -> `Screen` has no focused client, and we focused one
    /// - `Some((Some(_), Some(_)))` -> `Screen` had a focused client, and we changed focus
    ///
    /// The order is always `Some((Some(old_client), Some(new_client)))`
    ///
    /// TODO: when there are no clients on the current screen, try focusing on the adjacent
    pub fn focus_client(
        screen_manager: &mut ScreenManager,
        direction: Direction,
    ) -> anyhow::Result<Option<(Option<xcb::x::Window>, Option<xcb::x::Window>)>> {
        let index = screen_manager.active_screen_idx();
        let screen = screen_manager.screen_mut(index);

        if screen.active_workspace().clients().is_empty() {
            return Ok(None);
        }

        let Some(client) = screen.focused_client() else {
            let focused_client = match direction {
                Direction::Left => Self::focus_last(screen),
                Direction::Down => Self::focus_first(screen),
                Direction::Up => Self::focus_last(screen),
                Direction::Right => Self::focus_first(screen),
            };

            return Ok(Some((None, focused_client)));
        };

        let should_change_screen = match direction {
            Direction::Left => Self::is_first(screen, client),
            Direction::Down => Self::is_last(screen, client),
            Direction::Up => Self::is_first(screen, client),
            Direction::Right => Self::is_last(screen, client),
        };

        if should_change_screen {
            let Some(new_screen) = screen_manager.get_relative_screen_idx(direction) else {
                return Ok(None);
            };

            screen_manager.set_active_screen(new_screen);
            let screen = screen_manager.screen_mut(new_screen);

            let focused_client = match direction {
                Direction::Left => Self::focus_last(screen),
                Direction::Down => Self::focus_first(screen),
                Direction::Up => Self::focus_last(screen),
                Direction::Right => Self::focus_first(screen),
            };

            return Ok(Some((Some(client), focused_client)));
        }

        let focused_client = match direction {
            Direction::Left => Self::focus_first(screen),
            Direction::Down => Self::focus_next(screen, client),
            Direction::Up => Self::focus_prev(screen, client),
            Direction::Right => Self::focus_next(screen, client),
        };

        Ok(Some((Some(client), focused_client)))
    }

    pub fn move_client(
        screen_manager: &mut ScreenManager,
        direction: Direction,
    ) -> Option<xcb::x::Window> {
        let index = screen_manager.active_screen_idx();
        let screen = screen_manager.screen_mut(index);

        if screen.active_workspace().clients().is_empty() {
            return None;
        }

        let Some(client) = screen.focused_client() else {
            let focused_client = match direction {
                Direction::Left => Self::focus_last(screen),
                Direction::Down => Self::focus_first(screen),
                Direction::Up => Self::focus_last(screen),
                Direction::Right => Self::focus_first(screen),
            };
            return focused_client;
        };

        let should_change_screen = match direction {
            Direction::Left => Self::is_first(screen, client),
            Direction::Down => Self::is_last(screen, client),
            Direction::Up => Self::is_first(screen, client),
            Direction::Right => Self::is_last(screen, client),
        };

        if should_change_screen {
            let Some(new_screen) = screen_manager.get_relative_screen_idx(direction) else {
                return None;
            };

            screen_manager
                .screen_mut(index)
                .active_workspace_mut()
                .remove_client(client);

            screen_manager
                .screen_mut(new_screen)
                .active_workspace_mut()
                .new_client(client);

            screen_manager.set_active_screen(new_screen);

            return None;
        }

        match direction {
            Direction::Left => Self::swap_first(screen, client),
            Direction::Down => Self::swap_next(screen, client),
            Direction::Up => Self::swap_prev(screen, client),
            Direction::Right => Self::swap_next(screen, client),
        };

        None
    }

    fn configure_window(conn: &Arc<xcb::Connection>, window: xcb::x::Window, client_pos: Position) {
        conn.send_request(&xcb::x::ConfigureWindow {
            window,
            value_list: &[
                xcb::x::ConfigWindow::X(client_pos.x),
                xcb::x::ConfigWindow::Y(client_pos.y),
                xcb::x::ConfigWindow::Width(client_pos.width),
                xcb::x::ConfigWindow::Height(client_pos.height),
            ],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;
    use xcb::XidNew;

    fn create_fake_client() -> (xcb::x::Window, xcb::x::Window) {
        let mut rng = rand::thread_rng();
        unsafe {
            (
                xcb::x::Window::new(rng.next_u32()),
                xcb::x::Window::new(rng.next_u32()),
            )
        }
    }

    #[test]
    fn test_client_focusing() {
        let config = Rc::new(RefCell::new(Config::default()));
        let root = unsafe { xcb::x::Window::new(0) };
        let screen_positions = vec![Screen::new(&config, Position::new(0, 0, 100, 100))];
        let mut screen_manager = ScreenManager::new(screen_positions, config, root);

        let (frame_a, client_a) = create_fake_client();
        let (frame_b, client_b) = create_fake_client();
        screen_manager.create_client(frame_a, client_a);
        screen_manager.create_client(frame_b, client_b);
        let screen = screen_manager.screen_mut(0);
        let workspace = screen.active_workspace_mut();

        // ┌──────────┐┌──────────┐
        // │ selected ││          │
        // └──────────┘└──────────┘
        // set the first one to be selected
        workspace.set_focused_client(Some(frame_a));
        assert!(workspace.clients().len().eq(&2));
        assert!(screen.focused_client().eq(&Some(frame_a)));

        // ┌──────────┐┌──────────┐
        // │          ││ selected │
        // └──────────┘└──────────┘
        // select the second one
        TallLayout::focus_client(&mut screen_manager, Direction::Right).unwrap();
        let screen = screen_manager.screen_mut(0);
        assert!(screen.focused_client().eq(&Some(frame_b)));

        // ┌──────────┐┌──────────┐
        // │          ││ selected │
        // └──────────┘└──────────┘
        // since we are at the last, it should do nothing and return Unhandled
        TallLayout::focus_client(&mut screen_manager, Direction::Right).unwrap();
        let screen = screen_manager.screen_mut(0);
        assert!(screen.focused_client().eq(&Some(frame_b)));

        // ┌──────────┐┌──────────┐
        // │ selected ││          │
        // └──────────┘└──────────┘
        // set the first one to be selected
        TallLayout::focus_client(&mut screen_manager, Direction::Left).unwrap();
        let screen = screen_manager.screen_mut(0);
        assert!(screen.focused_client().eq(&Some(frame_a)));

        // ┌──────────┐┌──────────┐
        // │ selected ││          │
        // └──────────┘└──────────┘
        // similarly, when at the first, should do nothing and return unhandled
        TallLayout::focus_client(&mut screen_manager, Direction::Left).unwrap();
        let screen = screen_manager.screen_mut(0);
        assert!(screen.focused_client().eq(&Some(frame_a)));
    }
}
