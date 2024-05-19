use crate::{event::EventContext, handlers::Handler};

#[derive(Debug, Default)]
pub struct UnmapWindowHandler {}

impl Handler for UnmapWindowHandler {
    fn on_unmap_notify(
        &mut self,
        context: EventContext<xcb::x::UnmapNotifyEvent>,
    ) -> anyhow::Result<()> {
        let window = context.event.window();
        let screen_manager = context.screen_manager.borrow();

        if let Some(client) = screen_manager
            .clients()
            .values()
            // we only match on the client window, as the frames unmap requests means that
            // we are simply hiding that client
            .find(|client| client.window.eq(&window))
        {
            let frame = client.frame;
            match context
                .layout_manager
                .close_client(client.clone(), context.atoms)
            {
                Ok(_) => tracing::debug!("succesfully unmapped window {:?}", window),
                // some softwares close their clients without waiting for the window manager
                // thus making this fails, it is fine to keep going even though we coudlnt
                // kill the client;
                // if it don't exist on the X server it should not exist on our state
                Err(_) => tracing::error!("failed to unmap client {:?}", window),
            }

            drop(screen_manager);
            let mut screen_manager = context.screen_manager.borrow_mut();

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

        match context
            .layout_manager
            .display_screens(&context.screen_manager, context.decorator)
        {
            Ok(_) => tracing::info!(
                "displayed all screens after unmapping a window {:?}",
                window
            ),
            Err(e) => {
                tracing::error!(
                    "failed to display screens after unmapping a window {:?}",
                    window
                );
                return Err(e);
            }
        }

        Ok(())
    }
}
