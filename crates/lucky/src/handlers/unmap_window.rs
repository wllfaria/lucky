use crate::ewmh::{ewmh_set_active_window, ewmh_set_focus, EwmhFocusAction};
use crate::{event::EventContext, handlers::Handler};

#[derive(Debug, Default)]
pub struct UnmapWindowHandler {}

impl UnmapWindowHandler {
    fn try_unmap_reserved_client(
        &self,
        context: &EventContext<xcb::x::UnmapNotifyEvent>,
    ) -> anyhow::Result<()> {
        let window = context.event.window();
        let mut screen_manager = context.screen_manager.borrow_mut();

        for screen in screen_manager.screens_mut() {
            let reserved_clients = screen.reserved_clients_mut();
            let reserved_client_idx = reserved_clients
                .iter()
                .position(|client| client.window.eq(&window));
            if let Some(reserved_client_idx) = reserved_client_idx {
                let reserved_client = reserved_clients[reserved_client_idx].clone();
                context
                    .layout_manager
                    .close_client(&reserved_client, context.atoms)?;
                screen.sub_left_reserved_area(reserved_client.reserved_left);
                screen.sub_bottom_reserved_area(reserved_client.reserved_bottom);
                screen.sub_top_reserved_area(reserved_client.reserved_top);
                screen.sub_right_reserved_area(reserved_client.reserved_right);
                screen.remove_reserved_client(reserved_client_idx);
            }
        }
        Ok(())
    }

    fn try_unmap_client(&self, context: &EventContext<xcb::x::UnmapNotifyEvent>) {
        let window = context.event.window();
        let mut screen_manager = context.screen_manager.borrow_mut();

        if let Some(client) = screen_manager
            .clients()
            .values()
            // we only match on the client window, as the frames unmap requests means that
            // we are simply hiding that client
            .find(|client| client.window.eq(&window))
        {
            let frame = client.frame;
            match context.layout_manager.close_client(client, context.atoms) {
                Ok(_) => tracing::debug!("succesfully unmapped window {:?}", window),
                // some softwares close their clients without waiting for the window manager
                // thus making this fails, it is fine to keep going even though we coudlnt
                // kill the client;
                // if it don't exist on the X server it should not exist on our state
                Err(_) => tracing::error!("failed to unmap client {:?}", window),
            }

            screen_manager.screens_mut().iter_mut().for_each(|s| {
                s.workspaces_mut()
                    .iter_mut()
                    .for_each(|ws| ws.remove_client(frame))
            });
            screen_manager.clients_mut().remove(&frame);
            let index = screen_manager.active_screen_idx();
            let workspace = screen_manager.screen_mut(index).active_workspace_mut();
            workspace.set_focused_client(workspace.clients().first().copied());
        }
    }
}

impl Handler for UnmapWindowHandler {
    fn on_unmap_notify(
        &mut self,
        context: EventContext<xcb::x::UnmapNotifyEvent>,
    ) -> anyhow::Result<()> {
        self.try_unmap_reserved_client(&context)?;
        self.try_unmap_client(&context);

        context
            .layout_manager
            .display_screens(&context.screen_manager, context.decorator)?;

        context
            .screen_manager
            .borrow_mut()
            .update_atoms(context.atoms, &context.conn);

        ewmh_set_focus(
            &context.conn,
            context.atoms,
            context.event.window(),
            EwmhFocusAction::Focus,
        )
        .ok();
        ewmh_set_active_window(
            &context.conn,
            context.screen_manager.borrow().root(),
            context.atoms,
            context.event.window(),
        )
        .ok();

        Ok(())
    }
}
