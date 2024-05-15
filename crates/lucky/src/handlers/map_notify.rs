use crate::event::EventContext;
use crate::handlers::handler::Handler;

#[derive(Default)]
pub struct MapNotifyHandler {}

impl Handler for MapNotifyHandler {
    fn on_map_notify(
        &mut self,
        context: EventContext<xcb::x::MapNotifyEvent>,
    ) -> anyhow::Result<()> {
        if context
            .clients
            .borrow()
            .frames
            .contains(&context.event.window())
        {
            return Ok(());
        }

        context
            .layout_manager
            .display_client_frame(context.event.event())?;

        Ok(())
    }
}
