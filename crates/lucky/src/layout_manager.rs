mod tall_layout;

use crate::{
    atoms::Atoms,
    decorator::Decorator,
    event::EventContext,
    layout_manager::tall_layout::TallLayout,
    screen::{Client, Workspace, WorkspaceLayout},
    screen_manager::ScreenManager,
};
use config::{AvailableActions, Config};
use std::{
    cell::RefCell,
    ops::{Add, Sub},
    rc::Rc,
    sync::Arc,
};
use xcb::Xid;

#[derive(Debug, PartialEq)]
pub enum ActionHandledStatus {
    FullyHandled,
    Unhandled,
}

pub struct LayoutManager {
    config: Rc<RefCell<Config>>,
    conn: Arc<xcb::Connection>,
}

impl LayoutManager {
    pub fn new(conn: Arc<xcb::Connection>, config: Rc<RefCell<Config>>) -> Self {
        LayoutManager { config, conn }
    }

    pub fn enable_client_events(&self, window: xcb::x::Window) -> anyhow::Result<()> {
        self.conn.send_request(&xcb::x::ChangeWindowAttributes {
            window,
            value_list: &[(xcb::x::Cw::EventMask(
                xcb::x::EventMask::PROPERTY_CHANGE
                    | xcb::x::EventMask::SUBSTRUCTURE_NOTIFY
                    | xcb::x::EventMask::ENTER_WINDOW,
            ))],
        });

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
                self.hide_workspace(workspace)?;
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

    pub fn focus_left(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let active_screen_idx = screen_manager.active_screen_idx();
        let screen = screen_manager.screen(active_screen_idx);
        let workspace = screen.active_workspace();

        match workspace.layout() {
            WorkspaceLayout::Tall => {
                if let ActionHandledStatus::Unhandled = TallLayout::focus_left(&mut screen_manager)
                {
                    if active_screen_idx.gt(&0) {
                        let left_index = active_screen_idx.sub(1);
                        screen_manager.set_active_screen(left_index);
                        // here we dont care about the result of this operations, if there is no
                        // client to focus to the left, we just let it as is
                        //
                        // since we are moving screens, we always select the rightmost client of
                        // the new screen
                        _ = TallLayout::focus_last(&mut screen_manager);
                    }
                }
            }
        }

        drop(screen_manager);
        self.display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }

    pub fn focus_down(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let active_screen_idx = screen_manager.active_screen_idx();
        let screen = screen_manager.screen(active_screen_idx);
        let workspace = screen.active_workspace();

        match workspace.layout() {
            // on tall workspace, selecting right and bottom has the same effect.
            WorkspaceLayout::Tall => {
                if let ActionHandledStatus::Unhandled =
                    TallLayout::focus_right_or_bottom(&mut screen_manager)
                {
                    let total_screens = screen_manager.screens().len().sub(1);
                    if total_screens.gt(&active_screen_idx) {
                        let left_index = active_screen_idx.add(1);
                        screen_manager.set_active_screen(left_index);
                        // here we dont care about the result of this operations, if there is no
                        // client to focus down, we just let it as is
                        //
                        // since we are moving screens, we always select the leftmost client of
                        // the new screen
                        _ = TallLayout::focus_first(&mut screen_manager);
                    }
                }
            }
        }

        drop(screen_manager);
        self.display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }

    pub fn focus_up(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let active_screen_idx = screen_manager.active_screen_idx();
        let screen = screen_manager.screen(active_screen_idx);
        let workspace = screen.active_workspace();

        match workspace.layout() {
            WorkspaceLayout::Tall => {
                if let ActionHandledStatus::Unhandled = TallLayout::focus_up(&mut screen_manager) {
                    // im still thinking about this, should focusing up change screens?
                    if active_screen_idx.gt(&0) {
                        let left_index = active_screen_idx.sub(1);
                        screen_manager.set_active_screen(left_index);
                        // here we dont care about the result of this operations, if there is no
                        // client to focus up, we just let it as is
                        //
                        // since we are moving screens, we always select the rightmost client of
                        // the new screen
                        _ = TallLayout::focus_last(&mut screen_manager);
                    }
                }
            }
        }

        drop(screen_manager);
        self.display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }

    pub fn focus_right(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let active_screen_idx = screen_manager.active_screen_idx();
        let screen = screen_manager.screen(active_screen_idx);
        let workspace = screen.active_workspace();

        match workspace.layout() {
            // on tall workspace, selecting right and bottom has the same effect.
            WorkspaceLayout::Tall => {
                if let ActionHandledStatus::Unhandled =
                    TallLayout::focus_right_or_bottom(&mut screen_manager)
                {
                    let total_screens = screen_manager.screens().len().sub(1);
                    if total_screens.gt(&active_screen_idx) {
                        let left_index = active_screen_idx.add(1);
                        screen_manager.set_active_screen(left_index);
                        // here we dont care about the result of this operations, if there is no
                        // client to focus right, we just let it as is
                        //
                        // since we are moving screens, we always select the leftmost client of
                        // the new screen
                        _ = TallLayout::focus_first(&mut screen_manager);
                    }
                }
            }
        }

        drop(screen_manager);
        self.display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }

    pub fn move_left(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let active_screen_idx = screen_manager.active_screen_idx();
        let screen = screen_manager.screen(active_screen_idx);
        let workspace = screen.active_workspace();

        match workspace.layout() {
            WorkspaceLayout::Tall => match TallLayout::move_left(&mut screen_manager) {
                // we failed to select the next left client on the screen, so we try to switch
                // screens if available, and select the rightmost client there
                ActionHandledStatus::Unhandled => {}
                ActionHandledStatus::FullyHandled => {} // purposefully do nothing here
            },
        }

        drop(screen_manager);
        self.display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }

    pub fn move_down(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let screen = screen_manager.screen(screen_manager.active_screen_idx());
        let workspace = screen.active_workspace();

        match workspace.layout() {
            WorkspaceLayout::Tall => match TallLayout::move_down(&mut screen_manager) {
                ActionHandledStatus::Unhandled => {}
                ActionHandledStatus::FullyHandled => {}
            },
        }

        drop(screen_manager);
        self.display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }

    pub fn move_up(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let screen = screen_manager.screen(screen_manager.active_screen_idx());
        let workspace = screen.active_workspace();

        match workspace.layout() {
            WorkspaceLayout::Tall => match TallLayout::move_up(&mut screen_manager) {
                ActionHandledStatus::Unhandled => {}
                ActionHandledStatus::FullyHandled => {}
            },
        }

        drop(screen_manager);
        self.display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }

    pub fn move_right(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let screen = screen_manager.screen(screen_manager.active_screen_idx());
        let workspace = screen.active_workspace();

        match workspace.layout() {
            WorkspaceLayout::Tall => match TallLayout::move_right(&mut screen_manager) {
                ActionHandledStatus::Unhandled => {}
                ActionHandledStatus::FullyHandled => {}
            },
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
            self.hide_workspace(&screen.workspaces()[active_workspace_id as usize])?;
            drop(screen_manager);
            self.display_screens(&context.screen_manager, context.decorator)?;
        }

        Ok(())
    }

    pub fn hide_workspace(&self, workspace: &Workspace) -> anyhow::Result<()> {
        for client in workspace.clients() {
            self.conn
                .send_request(&xcb::x::UnmapWindow { window: *client });
        }

        Ok(())
    }

    /// Closes an open client.
    ///
    /// we need to query the `WM_PROTOCOLS` defined on the window to define how to properly
    /// close it. Modern clients will usually support `WM_DELETE_WINDOW`, and in this case
    /// we can close by sending a `ClientMessageEvent`, otherwise we have to manually close
    /// it through the `DestroyWindow` event.
    pub fn close_client(&self, client: Client, atoms: &Atoms) -> anyhow::Result<()> {
        let cookie = self.conn.send_request(&xcb::x::GetProperty {
            delete: false,
            window: client.window,
            property: atoms.wm_protocols,
            r#type: xcb::x::ATOM_ATOM,
            long_offset: 0,
            long_length: 1024,
        });
        let supports_wm_delete_window = match self.conn.wait_for_reply(cookie) {
            Ok(protocols) => {
                let protocol_atoms: &[xcb::x::Atom] = protocols.value();
                protocol_atoms
                    .iter()
                    .any(|&atom| atom == atoms.wm_delete_window)
            }
            // if we fail to fetch the property from the client for some reason, we just destroy
            // the frame, as we don't want to have a dirty state
            Err(_) => false,
        };

        if supports_wm_delete_window {
            let event = xcb::x::ClientMessageEvent::new(
                client.window,
                atoms.wm_protocols,
                xcb::x::ClientMessageData::Data32([
                    atoms.wm_delete_window.resource_id(),
                    xcb::x::CURRENT_TIME,
                    0,
                    0,
                    0,
                ]),
            );

            self.conn.send_request(&xcb::x::SendEvent {
                propagate: false,
                destination: xcb::x::SendEventDest::Window(client.window),
                event_mask: xcb::x::EventMask::NO_EVENT,
                event: &event,
            });

            self.conn.send_request(&xcb::x::DestroyWindow {
                window: client.frame,
            });
        } else {
            self.conn.send_request(&xcb::x::DestroyWindow {
                window: client.frame,
            });
        }

        Ok(())
    }
}
