use crate::event::EventContext;
use crate::handlers::handler::Handler;

#[derive(Default)]
pub struct MapWindowHandler {}

impl Handler for MapWindowHandler {
    fn on_map_request(
        &mut self,
        context: EventContext<xcb::x::MapRequestEvent>,
    ) -> anyhow::Result<()> {
        let mut clients = context.clients.borrow_mut();
        clients.create(context.event.window())?;
        clients.display(context.event.window())?;
        Ok(())
    }
}
