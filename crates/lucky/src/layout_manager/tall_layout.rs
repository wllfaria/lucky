use config::Config;

use crate::decorator::Decorator;
use crate::screen::{Client, Screen};
use crate::screen_manager::{Direction, Position, ScreenManager};

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
            match decorator.unfocus_client(client) {
                Ok(_) => tracing::info!("removed focus from client: {:?}", client),
                Err(e) => {
                    return Err(e);
                }
            }
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

        tracing::debug!(
            "displaying main client with frame at: {frame_position}, client at: {client_position}"
        );

        Self::configure_window(conn, client.frame, frame_position);
        Self::configure_window(conn, client.window, client_position);

        tracing::debug!("configured window and frame");

        conn.send_request(&xcb::x::MapWindow {
            window: client.window,
        });
        conn.send_request(&xcb::x::MapWindow {
            window: client.frame,
        });
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
        conn.send_request(&xcb::x::MapWindow {
            window: client.window,
        });
        conn.send_request(&xcb::x::MapWindow {
            window: client.frame,
        });
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

    fn swap_first(screen: &mut Screen, client: xcb::x::Window) {
        let index = screen
            .active_workspace()
            .clients()
            .iter()
            .position(|c| c.eq(&client))
            .expect("workspace clients vector should include selected client");

        screen.active_workspace_mut().clients_mut().swap(index, 0);
    }

    fn swap_prev(screen: &mut Screen, client: xcb::x::Window) {
        let index = screen
            .active_workspace()
            .clients()
            .iter()
            .position(|c| c.eq(&client))
            .expect("workspace clients vector should include selected client");

        screen
            .active_workspace_mut()
            .clients_mut()
            .swap(index, index.sub(1));
    }

    fn swap_next(screen: &mut Screen, client: xcb::x::Window) {
        let index = screen
            .active_workspace()
            .clients()
            .iter()
            .position(|c| c.eq(&client))
            .expect("workspace clients vector should include selected client");

        screen
            .active_workspace_mut()
            .clients_mut()
            .swap(index, index.add(1));
    }

    fn focus_first(screen: &mut Screen, _: xcb::x::Window) {
        let first_client = screen
            .active_workspace()
            .clients()
            .first()
            .copied()
            .expect("tried to focus a client on an empty workspace");
        screen
            .active_workspace_mut()
            .set_focused_client(Some(first_client));
    }

    fn focus_last(screen: &mut Screen, _: xcb::x::Window) {
        let last_client = screen
            .active_workspace()
            .clients()
            .last()
            .copied()
            .expect("tried to focus a client on an empty workspace");
        screen
            .active_workspace_mut()
            .set_focused_client(Some(last_client));
    }

    fn focus_prev(screen: &mut Screen, client: xcb::x::Window) {
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
            .copied()
            .expect("should have a next client at this point");

        screen
            .active_workspace_mut()
            .set_focused_client(Some(client));
    }

    fn focus_next(screen: &mut Screen, client: xcb::x::Window) {
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
            .copied()
            .expect("should have a next client at this point");

        screen
            .active_workspace_mut()
            .set_focused_client(Some(client));
    }

    pub fn focus_client(screen_manager: &mut ScreenManager, direction: Direction) {
        let index = screen_manager.active_screen_idx();
        let screen = screen_manager.screen_mut(index);

        if screen.active_workspace().clients().is_empty() {
            return;
        }

        let client = screen
            .focused_client()
            .expect("tried to get the focused client when there was none");

        if screen.focused_client().is_none() {
            return;
        }

        let should_change_screen = match direction {
            Direction::Left => Self::is_first(screen, client),
            Direction::Down => Self::is_last(screen, client),
            Direction::Up => Self::is_first(screen, client),
            Direction::Right => Self::is_last(screen, client),
        };

        if should_change_screen {
            let Some(new_screen) = screen_manager.get_relative_screen_idx(direction) else {
                return;
            };

            screen_manager.set_active_screen(new_screen);
            let screen = screen_manager.screen_mut(index);

            match direction {
                Direction::Left => Self::focus_first(screen, client),
                Direction::Down => Self::focus_first(screen, client),
                Direction::Up => Self::focus_last(screen, client),
                Direction::Right => Self::focus_last(screen, client),
            }

            return;
        }

        match direction {
            Direction::Left => Self::focus_first(screen, client),
            Direction::Down => Self::focus_next(screen, client),
            Direction::Up => Self::focus_prev(screen, client),
            Direction::Right => Self::focus_next(screen, client),
        }
    }

    pub fn move_client(screen_manager: &mut ScreenManager, direction: Direction) {
        let index = screen_manager.active_screen_idx();
        let screen = screen_manager.screen_mut(index);

        if screen.active_workspace().clients().is_empty() {
            return;
        }

        let client = screen
            .focused_client()
            .expect("tried to get the focused client when there was none");

        if screen.focused_client().is_none() {
            match direction {
                Direction::Left => Self::focus_last(screen, client),
                Direction::Down => Self::focus_first(screen, client),
                Direction::Up => Self::focus_last(screen, client),
                Direction::Right => Self::focus_first(screen, client),
            }
            return;
        }

        let should_change_screen = match direction {
            Direction::Left => Self::is_first(screen, client),
            Direction::Down => Self::is_last(screen, client),
            Direction::Up => Self::is_first(screen, client),
            Direction::Right => Self::is_last(screen, client),
        };

        if should_change_screen {
            let Some(new_screen) = screen_manager.get_relative_screen_idx(direction) else {
                return;
            };

            screen_manager
                .screen_mut(index)
                .active_workspace_mut()
                .remove_client(client);

            screen_manager
                .screen_mut(new_screen)
                .active_workspace_mut()
                .new_client(client);

            return;
        }

        match direction {
            Direction::Left => Self::swap_first(screen, client),
            Direction::Down => Self::swap_next(screen, client),
            Direction::Up => Self::swap_prev(screen, client),
            Direction::Right => Self::swap_next(screen, client),
        }
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
        let screen_positions = vec![Position::new(0, 0, 100, 100)];
        let config = Rc::new(RefCell::new(Config::default()));
        let mut screen_manager = ScreenManager::new(screen_positions, config);

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
        TallLayout::focus_client(&mut screen_manager, Direction::Right);
        let screen = screen_manager.screen_mut(0);
        assert!(screen.focused_client().eq(&Some(frame_b)));

        // ┌──────────┐┌──────────┐
        // │          ││ selected │
        // └──────────┘└──────────┘
        // since we are at the last, it should do nothing and return Unhandled
        TallLayout::focus_client(&mut screen_manager, Direction::Right);
        let screen = screen_manager.screen_mut(0);
        assert!(screen.focused_client().eq(&Some(frame_b)));

        // ┌──────────┐┌──────────┐
        // │ selected ││          │
        // └──────────┘└──────────┘
        // set the first one to be selected
        TallLayout::focus_client(&mut screen_manager, Direction::Left);
        let screen = screen_manager.screen_mut(0);
        assert!(screen.focused_client().eq(&Some(frame_a)));

        // ┌──────────┐┌──────────┐
        // │ selected ││          │
        // └──────────┘└──────────┘
        // similarly, when at the first, should do nothing and return unhandled
        TallLayout::focus_client(&mut screen_manager, Direction::Left);
        let screen = screen_manager.screen_mut(0);
        assert!(screen.focused_client().eq(&Some(frame_a)));
    }
}
