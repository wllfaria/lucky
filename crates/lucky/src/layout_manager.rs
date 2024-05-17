mod master_layout;

use crate::{
    decorator::Decorator,
    event::EventContext,
    layout_manager::master_layout::TallLayout,
    screen::{Client, WorkspaceLayout},
    screen_manager::ScreenManager,
};
use config::Config;
use std::{cell::RefCell, rc::Rc, sync::Arc};
use xcb::Xid;

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
        let screen = screen_manager.screen(screen_manager.active_screen);
        let workspace = screen.active_workspace();

        match workspace.layout() {
            WorkspaceLayout::Tall => match TallLayout::focus_left(&mut screen_manager) {
                ActionHandledStatus::Unhandled => {}
                ActionHandledStatus::FullyHandled => {}
            },
        }

        drop(screen_manager);
        self.display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }

    pub fn focus_down(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let screen = screen_manager.screen(screen_manager.active_screen);
        let workspace = screen.active_workspace();

        match workspace.layout() {
            // on tall workspace, selecting right and bottom has the same effect.
            WorkspaceLayout::Tall => match TallLayout::focus_right_or_bottom(&mut screen_manager) {
                ActionHandledStatus::Unhandled => {}
                ActionHandledStatus::FullyHandled => {}
            },
        }

        drop(screen_manager);
        self.display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }

    pub fn focus_up(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let screen = screen_manager.screen(screen_manager.active_screen);
        let workspace = screen.active_workspace();

        match workspace.layout() {
            WorkspaceLayout::Tall => match TallLayout::focus_up(&mut screen_manager) {
                ActionHandledStatus::Unhandled => {}
                ActionHandledStatus::FullyHandled => {}
            },
        }

        drop(screen_manager);
        self.display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }

    pub fn focus_right(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let screen = screen_manager.screen(screen_manager.active_screen);
        let workspace = screen.active_workspace();

        match workspace.layout() {
            // on tall workspace, selecting right and bottom has the same effect.
            WorkspaceLayout::Tall => match TallLayout::focus_right_or_bottom(&mut screen_manager) {
                ActionHandledStatus::Unhandled => {}
                ActionHandledStatus::FullyHandled => {}
            },
        }

        drop(screen_manager);
        self.display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }

    pub fn move_left(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let screen = screen_manager.screen(screen_manager.active_screen);
        let workspace = screen.active_workspace();

        match workspace.layout() {
            WorkspaceLayout::Tall => match TallLayout::move_left(&mut screen_manager) {
                ActionHandledStatus::Unhandled => {}
                ActionHandledStatus::FullyHandled => {}
            },
        }

        drop(screen_manager);
        self.display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }

    pub fn move_down(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        let screen = screen_manager.screen(screen_manager.active_screen);
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
        let screen = screen_manager.screen(screen_manager.active_screen);
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
        let screen = screen_manager.screen(screen_manager.active_screen);
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

    /// Closes an open client.
    ///
    /// we need to query the `WM_PROTOCOLS` defined on the window to define how to properly
    /// close it. Modern clients will usually support `WM_DELETE_WINDOW`, and in this case
    /// we can close by sending a `ClientMessageEvent`, otherwise we have to manually close
    /// it through the `DestroyWindow` event.
    pub fn close_client(
        &self,
        client: Client,
        context: &EventContext<xcb::x::KeyPressEvent>,
    ) -> anyhow::Result<()> {
        let cookie = self.conn.send_request(&xcb::x::GetProperty {
            delete: false,
            window: client.window,
            property: context.atoms.wm_protocols,
            r#type: xcb::x::ATOM_ATOM,
            long_offset: 0,
            long_length: 1024,
        });
        let protocols = self.conn.wait_for_reply(cookie)?;
        let atoms: &[xcb::x::Atom] = protocols.value();

        if atoms
            .iter()
            .any(|&atom| atom == context.atoms.wm_delete_window)
        {
            let event = xcb::x::ClientMessageEvent::new(
                client.window,
                context.atoms.wm_protocols,
                xcb::x::ClientMessageData::Data32([
                    context.atoms.wm_delete_window.resource_id(),
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

            self.conn.flush()?;
        } else {
            self.conn.send_request(&xcb::x::DestroyWindow {
                window: client.frame,
            });

            self.conn.flush()?;
        }

        Ok(())
    }
}
