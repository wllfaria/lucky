mod tall_layout;
use crate::ewmh::{ewmh_set_active_window, ewmh_set_focus, EwmhFocusAction};

use crate::xcb_utils::*;
use crate::{
    atoms::Atoms,
    decorator::Decorator,
    event::EventContext,
    layout_manager::tall_layout::TallLayout,
    screen::{Workspace, WorkspaceLayout},
    screen_manager::{Direction, ScreenManager},
};
use config::{AvailableActions, Config};
use std::{cell::RefCell, rc::Rc, sync::Arc};
use xcb::Xid;

pub struct LayoutManager {
    config: Rc<RefCell<Config>>,
    conn: Arc<xcb::Connection>,
}

impl LayoutManager {
    pub fn new(conn: Arc<xcb::Connection>, config: Rc<RefCell<Config>>) -> Self {
        LayoutManager { config, conn }
    }

    pub fn enable_client_events(&self, window: xcb::x::Window) -> anyhow::Result<()> {
        xcb_change_attr!(
            self.conn,
            window,
            &[(xcb::x::Cw::EventMask(
                xcb::x::EventMask::PROPERTY_CHANGE
                    | xcb::x::EventMask::SUBSTRUCTURE_NOTIFY
                    | xcb::x::EventMask::ENTER_WINDOW,
            ))]
        );

        Ok(())
    }

    pub fn display_screens(
        &self,
        screen_manager: &Rc<RefCell<ScreenManager>>,
        decorator: &Decorator,
    ) -> anyhow::Result<()> {
        for screen in screen_manager.borrow().screens() {
            let workspace = screen.active_workspace();
            let screen_manager = screen_manager.borrow();

            let visible_clients = screen_manager
                .get_visible_screen_clients(screen)
                .into_iter()
                .filter(|client| client.visible)
                .collect::<Vec<_>>();

            if visible_clients.is_empty() {
                self.hide_workspace(workspace);
                continue;
            }

            let focused_client = screen_manager.get_focused_client();

            match workspace.layout() {
                WorkspaceLayout::Tall => TallLayout::display_clients(
                    &self.conn,
                    &self.config,
                    screen,
                    visible_clients,
                    focused_client,
                    decorator,
                )?,
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub fn change_focus(
        &self,
        context: &EventContext<xcb::x::KeyPressEvent>,
        direction: Direction,
    ) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let active_screen_idx = screen_manager.active_screen_idx();
        let screen = screen_manager.screen(active_screen_idx);
        let workspace = screen.active_workspace();

        let result = match workspace.layout() {
            WorkspaceLayout::Tall => TallLayout::focus_client(&mut screen_manager, direction)?,
        };

        if let Some((prev_client, curr_client)) = result {
            let prev_client = screen_manager.clients().get(&prev_client).unwrap();
            let curr_client = screen_manager.clients().get(&curr_client).unwrap();

            ewmh_set_focus(
                &context.conn,
                context.atoms,
                prev_client.window,
                EwmhFocusAction::Unfocus,
            )
            .ok();
            ewmh_set_focus(
                &context.conn,
                context.atoms,
                curr_client.window,
                EwmhFocusAction::Focus,
            )
            .ok();
            ewmh_set_active_window(
                &context.conn,
                screen_manager.root(),
                context.atoms,
                curr_client.window,
            )
            .ok();
        }

        drop(screen_manager);
        self.display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub fn move_client(
        &self,
        context: &EventContext<xcb::x::KeyPressEvent>,
        direction: Direction,
    ) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let active_screen_idx = screen_manager.active_screen_idx();
        let screen = screen_manager.screen(active_screen_idx);
        let workspace = screen.active_workspace();

        let result = match workspace.layout() {
            WorkspaceLayout::Tall => TallLayout::move_client(&mut screen_manager, direction)?,
        };

        if let Some(focused_client) = result {
            let focused_client = screen_manager.clients().get(&focused_client).unwrap();
            ewmh_set_focus(
                &context.conn,
                context.atoms,
                focused_client.window,
                EwmhFocusAction::Focus,
            )
            .ok();
            ewmh_set_active_window(
                &context.conn,
                screen_manager.root(),
                context.atoms,
                focused_client.window,
            )
            .ok();
        }

        drop(screen_manager);
        self.display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }

    pub fn change_workspace(
        &self,
        context: &EventContext<xcb::x::KeyPressEvent>,
        action: AvailableActions,
    ) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let index = screen_manager.active_screen_idx();
        let screen = screen_manager.screen_mut(index);
        let active_workspace_id = screen.active_workspace().id();

        match action {
            AvailableActions::Workspace1 => screen.set_active_workspace(0),
            AvailableActions::Workspace2 => screen.set_active_workspace(1),
            AvailableActions::Workspace3 => screen.set_active_workspace(2),
            AvailableActions::Workspace4 => screen.set_active_workspace(3),
            AvailableActions::Workspace5 => screen.set_active_workspace(4),
            AvailableActions::Workspace6 => screen.set_active_workspace(5),
            AvailableActions::Workspace7 => screen.set_active_workspace(6),
            AvailableActions::Workspace8 => screen.set_active_workspace(7),
            AvailableActions::Workspace9 => screen.set_active_workspace(8),
            _ => {}
        };

        if screen.active_workspace().id().ne(&active_workspace_id) {
            self.hide_workspace(&screen.workspaces()[active_workspace_id as usize]);
            drop(screen_manager);
            self.display_screens(&context.screen_manager, context.decorator)?;
        }

        Ok(())
    }

    pub fn move_to_workspace(
        &self,
        context: &EventContext<xcb::x::KeyPressEvent>,
        action: AvailableActions,
    ) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let index = screen_manager.active_screen_idx();
        if let Some(active_client) = screen_manager.get_focused_client() {
            let client_frame = active_client.frame;
            let screen = screen_manager.screen_mut(index);
            let active_workspace_id = screen.active_workspace_id();
            let workspaces = screen.workspaces_mut();
            workspaces[active_workspace_id].remove_client(client_frame);

            let new_workspace_id = match action {
                AvailableActions::MoveToWorkspace1 => 0,
                AvailableActions::MoveToWorkspace2 => 1,
                AvailableActions::MoveToWorkspace3 => 2,
                AvailableActions::MoveToWorkspace4 => 3,
                AvailableActions::MoveToWorkspace5 => 4,
                AvailableActions::MoveToWorkspace6 => 5,
                AvailableActions::MoveToWorkspace7 => 6,
                AvailableActions::MoveToWorkspace8 => 7,
                AvailableActions::MoveToWorkspace9 => 8,
                _ => unreachable!(),
            };

            workspaces[new_workspace_id]
                .clients_mut()
                .push(client_frame);

            self.hide_client(&client_frame);
        }

        drop(screen_manager);
        self.display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }

    fn hide_workspace(&self, workspace: &Workspace) {
        for client in workspace.clients() {
            self.hide_client(client);
        }
    }

    fn hide_client(&self, client: &xcb::x::Window) {
        xcb_unmap_win!(self.conn, *client);
    }

    /// Closes an open client.
    ///
    /// we need to query the `WM_PROTOCOLS` defined on the window to define how to properly
    /// close it. Modern clients will usually support `WM_DELETE_WINDOW`, and in this case
    /// we can close by sending a `ClientMessageEvent`, otherwise we have to manually close
    /// it through the `DestroyWindow` event.
    pub fn close_client<C>(&self, client: &C, atoms: &Atoms) -> anyhow::Result<()>
    where
        C: crate::screen::IntoClient,
    {
        let supports_wm_delete_window =
            xcb_get_prop!(self.conn, client.get_window(), atoms.wm_protocols, 1024)
                .map(|cookie| {
                    cookie
                        .value::<xcb::x::Atom>()
                        .iter()
                        .any(|&atom| atom == atoms.wm_delete_window)
                })
                .unwrap_or(false);

        if supports_wm_delete_window {
            let event = xcb::x::ClientMessageEvent::new(
                client.get_window(),
                atoms.wm_protocols,
                xcb::x::ClientMessageData::Data32([
                    atoms.wm_delete_window.resource_id(),
                    xcb::x::CURRENT_TIME,
                    0,
                    0,
                    0,
                ]),
            );

            xcb_send_event!(
                self.conn,
                xcb::x::SendEventDest::Window(client.get_window()),
                &event
            );
            if let Some(frame) = client.get_frame() {
                xcb_destroy_win!(self.conn, frame);
            }
        } else if let Some(frame) = client.get_frame() {
            xcb_destroy_win!(self.conn, frame);
        }

        Ok(())
    }
}
