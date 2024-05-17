use crate::event::EventContext;
use crate::handlers::handler::Handler;

#[derive(Default)]
pub struct MapWindowHandler {}

impl Handler for MapWindowHandler {
    fn on_map_request(
        &mut self,
        context: EventContext<xcb::x::MapRequestEvent>,
    ) -> anyhow::Result<()> {
        let window = context.event.window();
        let frame = context.decorator.decorate_client(window)?;
        tracing::debug!("creating window {window:?} with frame {frame:?}");

        match context.layout_manager.enable_client_events(window) {
            Ok(_) => tracing::info!("enabled events for window: {:?}", window),
            Err(e) => {
                tracing::error!("failed to enable events for window: {:?}", window);
                return Err(e);
            }
        }
        match context.layout_manager.enable_client_events(frame) {
            Ok(_) => tracing::info!("enabled events for framw: {:?}", window),
            Err(e) => {
                tracing::error!("failed to enable events for frame: {:?}", window);
                return Err(e);
            }
        }

        context
            .screen_manager
            .borrow_mut()
            .create_client(frame, window);

        match context
            .layout_manager
            .display_screens(&context.screen_manager, context.decorator)
        {
            Ok(_) => tracing::info!(
                "displayed all windows after mapping a new client: {:?}",
                frame
            ),
            Err(e) => {
                tracing::error!(
                    "failed to display windows after mapping a new client: {:?}",
                    frame
                );
                return Err(e);
            }
        }

        Ok(())
    }
}
