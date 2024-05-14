use crate::event::EventContext;
use crate::handlers::handler::Handler;

#[derive(Default)]
pub struct HoverHandler {}

impl Handler for HoverHandler {
    fn on_enter_notify(
        &mut self,
        _context: EventContext<xcb::x::EnterNotifyEvent>,
    ) -> anyhow::Result<()> {
        tracing::debug!("mouse entered window");
        Ok(())
    }
}
