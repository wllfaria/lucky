use crate::{
    decorator::Decorator,
    screen::{Client, Screen},
    screen_manager::{Position, ScreenManager},
};
use config::Config;
use std::{
    cell::RefCell,
    ops::{Add, Div, Mul, Sub},
    rc::Rc,
    sync::Arc,
};

use super::ActionHandledStatus;

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

        let master_width = if visible_clients_len.eq(&1) {
            screen.position().width
        } else {
            screen.position().width.div(2)
        };

        for (i, client) in clients.iter().enumerate() {
            decorator.unfocus_client(client)?;
            match i {
                0 => Self::display_master_client(conn, client, screen, master_width, config),
                _ => Self::display_sibling_client(
                    conn,
                    client,
                    screen,
                    i,
                    visible_clients_len,
                    master_width,
                    config,
                ),
            }
        }

        if let Some(focused_client) = focused_client {
            let client = clients
                .iter()
                .find(|&&client| client.eq(focused_client))
                .expect("focused client must exist within all the clients");
            if focused_client.eq(client) {
                decorator.focus_client(client)?;
            }
        }

        conn.flush()?;
        Ok(())
    }

    fn display_master_client(
        conn: &Arc<xcb::Connection>,
        client: &Client,
        screen: &Screen,
        master_width: u32,
        config: &Rc<RefCell<Config>>,
    ) {
        Self::configure_window(
            conn,
            client.frame,
            Position::new(
                0,
                0,
                master_width.sub(config.borrow().border_width().mul(2) as u32),
                screen
                    .position()
                    .height
                    .sub(config.borrow().border_width().mul(2) as u32),
            ),
        );
        Self::configure_window(
            conn,
            client.window,
            Position::new(
                0,
                0,
                master_width.sub(config.borrow().border_width() as u32),
                screen
                    .position()
                    .height
                    .sub(config.borrow().border_width() as u32),
            ),
        );

        conn.send_request(&xcb::x::MapWindow {
            window: client.window,
        });
        conn.send_request(&xcb::x::MapWindow {
            window: client.frame,
        });
    }

    fn display_sibling_client(
        conn: &Arc<xcb::Connection>,
        client: &Client,
        screen: &Screen,
        index: usize,
        total: usize,
        master_width: u32,
        config: &Rc<RefCell<Config>>,
    ) {
        let width = screen.position().width.sub(master_width);
        let total_siblings = total.sub(1);
        let height = screen.position().height.div(total_siblings as u32);
        let sibling_index = index.sub(1);

        Self::configure_window(
            conn,
            client.frame,
            Position::new(
                master_width as i32,
                height.mul(sibling_index as u32) as i32,
                width.sub(config.borrow().border_width().mul(2) as u32),
                height.sub(config.borrow().border_width().mul(2) as u32),
            ),
        );
        Self::configure_window(conn, client.window, Position::new(0, 0, width, height));
        conn.send_request(&xcb::x::MapWindow {
            window: client.window,
        });
        conn.send_request(&xcb::x::MapWindow {
            window: client.frame,
        });
    }

    pub fn focus_left(screen_manager: &mut ScreenManager) -> ActionHandledStatus {
        let index = screen_manager.active_screen_idx();
        let screen = screen_manager.screen_mut(index);

        // If the active workspace has no clients, we return as unhandled for the layout manager to
        // decide what to do
        if screen.active_workspace().clients().is_empty() {
            return ActionHandledStatus::Unhandled;
        }

        // If the active workspace has no focused client, but has any number of clients, we
        // select the first one
        //
        // Im not sure if this scenario is pratically possible, but since it theoretically is,
        // we handle it
        if screen.focused_client().is_none() {
            let last_client = screen
                .active_workspace()
                .clients()
                .last()
                .copied()
                .expect("tried to focus a client on an empty workspace");
            screen
                .active_workspace_mut()
                .set_focused_client(Some(last_client));
            return ActionHandledStatus::FullyHandled;
        }

        // if the active workspace has the leftmost client focused, we return to the layout
        // manager as unhandled for it to decide what to do
        let client = screen
            .focused_client()
            .expect("tried to get the focused client when there was none");

        let is_first = screen
            .active_workspace()
            .clients()
            .first()
            .is_some_and(|focused| focused.eq(&client));

        if is_first {
            return ActionHandledStatus::Unhandled;
        }

        // since we are trying to focus left, the only client to the left is the main, so
        // we set the focus to it
        let client = screen
            .active_workspace()
            .clients()
            .first()
            .copied()
            .expect("should have a client at this point");
        screen
            .active_workspace_mut()
            .set_focused_client(Some(client));

        ActionHandledStatus::FullyHandled
    }

    pub fn focus_right_or_bottom(screen_manager: &mut ScreenManager) -> ActionHandledStatus {
        let index = screen_manager.active_screen_idx();
        let screen = screen_manager.screen_mut(index);

        // If the active workspace has no clients, we return as unhandled for the layout manager to
        // decide what to do
        if screen.active_workspace().clients().is_empty() {
            return ActionHandledStatus::Unhandled;
        }

        // If the active workspace has no focused client, but has any number of clients, we
        // select the first one
        //
        // Im not sure if this scenario is pratically possible, but since it theoretically is,
        // we handle it
        if screen.focused_client().is_none() {
            let first_client = screen
                .active_workspace()
                .clients()
                .first()
                .copied()
                .expect("tried to focus a client on an empty workspace");
            screen
                .active_workspace_mut()
                .set_focused_client(Some(first_client));
            return ActionHandledStatus::FullyHandled;
        }

        // if the active workspace has the rightmost client focused, we return to the layout
        // manager as unhandled for it to decide what to do
        let client = screen
            .focused_client()
            .expect("tried to get the focused client when there was none");

        let is_last = screen
            .active_workspace()
            .clients()
            .last()
            .is_some_and(|focused| focused.eq(&client));

        if is_last {
            return ActionHandledStatus::Unhandled;
        }

        // since we are trying to select right, we select the next client
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

        ActionHandledStatus::FullyHandled
    }

    pub fn focus_up(screen_manager: &mut ScreenManager) -> ActionHandledStatus {
        let index = screen_manager.active_screen_idx();
        let screen = screen_manager.screen_mut(index);

        // If the active workspace has no clients, we return as unhandled for the layout manager to
        // decide what to do
        if screen.active_workspace().clients().is_empty() {
            return ActionHandledStatus::Unhandled;
        }

        // If the active workspace has no focused client, but has any number of clients, we
        // select the last one
        //
        // Im not sure if this scenario is pratically possible, but since it theoretically is,
        // we handle it
        if screen.focused_client().is_none() {
            let last_client = screen
                .active_workspace()
                .clients()
                .last()
                .copied()
                .expect("tried to focus a client on an empty workspace");
            screen
                .active_workspace_mut()
                .set_focused_client(Some(last_client));
            return ActionHandledStatus::FullyHandled;
        }

        // if the active workspace has the leftmost client focused, we return to the layout
        // manager as unhandled for it to decide what to do
        let client = screen
            .focused_client()
            .expect("tried to get the focused client when there was none");

        let is_first = screen
            .active_workspace()
            .clients()
            .first()
            .is_some_and(|focused| focused.eq(&client));

        if is_first {
            return ActionHandledStatus::Unhandled;
        }

        // since we are trying to select up, we select the previous client
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

        ActionHandledStatus::FullyHandled
    }

    pub fn move_left(screen_manager: &mut ScreenManager) -> ActionHandledStatus {
        let index = screen_manager.active_screen_idx();
        let screen = screen_manager.screen_mut(index);

        // If the active workspace has no clients, we return as unhandled for the layout manager to
        // decide what to do
        if screen.active_workspace().clients().is_empty() {
            return ActionHandledStatus::Unhandled;
        }

        // If the active workspace has no focused client, but has any number of clients, we
        // select the last one, we cannot move a non-selected client
        if screen.focused_client().is_none() {
            let last_client = screen
                .active_workspace()
                .clients()
                .last()
                .copied()
                .expect("tried to focus a client on an empty workspace");
            screen
                .active_workspace_mut()
                .set_focused_client(Some(last_client));
            return ActionHandledStatus::FullyHandled;
        }

        // if the active workspace has the leftmost client focused, we return to the layout
        // manager as unhandled for it to decide what to do
        let client = screen
            .focused_client()
            .expect("tried to get the focused client when there was none");

        let is_first = screen
            .active_workspace()
            .clients()
            .first()
            .is_some_and(|focused| focused.eq(&client));

        if is_first {
            return ActionHandledStatus::Unhandled;
        }

        // since we are trying to move_left we swap with the first client on tall layouts
        let index = screen
            .active_workspace()
            .clients()
            .iter()
            .position(|c| c.eq(&client))
            .expect("workspace clients vector should include selected client");

        screen.active_workspace_mut().clients_mut().swap(index, 0);

        ActionHandledStatus::FullyHandled
    }

    pub fn move_down(screen_manager: &mut ScreenManager) -> ActionHandledStatus {
        let index = screen_manager.active_screen_idx();
        let screen = screen_manager.screen_mut(index);

        // If the active workspace has no clients, we return as unhandled for the layout manager to
        // decide what to do
        if screen.active_workspace().clients().is_empty() {
            return ActionHandledStatus::Unhandled;
        }

        // If the active workspace has no focused client, but has any number of clients, we
        // select the first one, we cannot move a non-selected client
        if screen.focused_client().is_none() {
            let first_client = screen
                .active_workspace()
                .clients()
                .first()
                .copied()
                .expect("tried to focus a client on an empty workspace");
            screen
                .active_workspace_mut()
                .set_focused_client(Some(first_client));
            return ActionHandledStatus::FullyHandled;
        }

        // if the active workspace has the rightmost client focused, we return to the layout
        // manager as unhandled for it to decide what to do
        let client = screen
            .focused_client()
            .expect("tried to get the focused client when there was none");

        let is_last = screen
            .active_workspace()
            .clients()
            .last()
            .is_some_and(|focused| focused.eq(&client));

        if is_last {
            return ActionHandledStatus::Unhandled;
        }

        // since we are trying to move_down we swap with the next client
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

        ActionHandledStatus::FullyHandled
    }

    pub fn move_up(screen_manager: &mut ScreenManager) -> ActionHandledStatus {
        let index = screen_manager.active_screen_idx();
        let screen = screen_manager.screen_mut(index);

        // If the active workspace has no clients, we return as unhandled for the layout manager to
        // decide what to do
        if screen.active_workspace().clients().is_empty() {
            return ActionHandledStatus::Unhandled;
        }

        // If the active workspace has no focused client, but has any number of clients, we
        // select the last one, we cannot move a non-selected client
        if screen.focused_client().is_none() {
            let last_client = screen
                .active_workspace()
                .clients()
                .last()
                .copied()
                .expect("tried to focus a client on an empty workspace");
            screen
                .active_workspace_mut()
                .set_focused_client(Some(last_client));
            return ActionHandledStatus::FullyHandled;
        }

        // if the active workspace has the leftmost client focused, we return to the layout
        // manager as unhandled for it to decide what to do
        let client = screen
            .focused_client()
            .expect("tried to get the focused client when there was none");

        let is_first = screen
            .active_workspace()
            .clients()
            .first()
            .is_some_and(|focused| focused.eq(&client));

        if is_first {
            return ActionHandledStatus::Unhandled;
        }

        // since we are trying to move_up we swap with the previous client
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

        ActionHandledStatus::FullyHandled
    }

    pub fn move_right(screen_manager: &mut ScreenManager) -> ActionHandledStatus {
        let index = screen_manager.active_screen_idx();
        let screen = screen_manager.screen_mut(index);

        // If the active workspace has no clients, we return as unhandled for the layout manager to
        // decide what to do
        if screen.active_workspace().clients().is_empty() {
            return ActionHandledStatus::Unhandled;
        }

        // If the active workspace has no focused client, but has any number of clients, we
        // select the last one, we cannot move a non-selected client
        if screen.focused_client().is_none() {
            let last_client = screen
                .active_workspace()
                .clients()
                .last()
                .copied()
                .expect("tried to focus a client on an empty workspace");
            screen
                .active_workspace_mut()
                .set_focused_client(Some(last_client));
            return ActionHandledStatus::FullyHandled;
        }

        // if the active workspace has the rightmost client focused, we return to the layout
        // manager as unhandled for it to decide what to do
        let client = screen
            .focused_client()
            .expect("tried to get the focused client when there was none");

        let is_last = screen
            .active_workspace()
            .clients()
            .last()
            .is_some_and(|focused| focused.eq(&client));

        if is_last {
            return ActionHandledStatus::Unhandled;
        }

        // since we are trying to move_right we swap with next client
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

        ActionHandledStatus::FullyHandled
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
        let action_handled_status = TallLayout::focus_right_or_bottom(&mut screen_manager);
        let screen = screen_manager.screen_mut(0);
        assert!(screen.focused_client().eq(&Some(frame_b)));
        assert!(action_handled_status.eq(&ActionHandledStatus::FullyHandled));

        // ┌──────────┐┌──────────┐
        // │          ││ selected │
        // └──────────┘└──────────┘
        // since we are at the last, it should do nothing and return Unhandled
        let action_handled_status = TallLayout::focus_right_or_bottom(&mut screen_manager);
        let screen = screen_manager.screen_mut(0);
        assert!(screen.focused_client().eq(&Some(frame_b)));
        assert!(action_handled_status.eq(&ActionHandledStatus::Unhandled));

        // ┌──────────┐┌──────────┐
        // │ selected ││          │
        // └──────────┘└──────────┘
        // set the first one to be selected
        let action_handled_status = TallLayout::focus_left(&mut screen_manager);
        let screen = screen_manager.screen_mut(0);
        assert!(screen.focused_client().eq(&Some(frame_a)));
        assert!(action_handled_status.eq(&ActionHandledStatus::FullyHandled));

        // ┌──────────┐┌──────────┐
        // │ selected ││          │
        // └──────────┘└──────────┘
        // similarly, when at the first, should do nothing and return unhandled
        let action_handled_status = TallLayout::focus_left(&mut screen_manager);
        let screen = screen_manager.screen_mut(0);
        assert!(screen.focused_client().eq(&Some(frame_a)));
        assert!(action_handled_status.eq(&ActionHandledStatus::Unhandled));
    }
}
