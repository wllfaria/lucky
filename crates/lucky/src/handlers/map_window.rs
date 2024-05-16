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

        context.layout_manager.enable_client_events(window)?;
        context.layout_manager.enable_client_events(frame)?;
        context
            .screen_manager
            .borrow_mut()
            .create_client(frame, window);

        context
            .layout_manager
            .display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }
}
