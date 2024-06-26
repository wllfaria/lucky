use crate::event::EventContext;
use crate::handlers::Handler;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PropertyHandler;

impl Handler for PropertyHandler {
    fn on_property_notify(
        &mut self,
        _context: EventContext<xcb::x::PropertyNotifyEvent>,
    ) -> anyhow::Result<()> {
        tracing::trace!("not handling this yet:");
        Ok(())
    }
}
