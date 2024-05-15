use crate::event::EventContext;
use crate::handlers::handler::Handler;

#[derive(Default)]
pub struct MapWindowHandler {}

impl Handler for MapWindowHandler {
    fn on_map_request(
        &mut self,
        context: EventContext<xcb::x::MapRequestEvent>,
    ) -> anyhow::Result<()> {
        let frame = context
            .decorator
            .maybe_decorate_client(context.event.window())?;

        // we enable events for both client and the window
        context
            .layout_manager
            .enable_client_events(context.event.window())?;
        context.layout_manager.enable_client_events(frame)?;

        context
            .clients
            .borrow_mut()
            .create(frame, context.event.window())?;
        context.layout_manager.display_clients(&context.clients)?;
        Ok(())
    }
}
