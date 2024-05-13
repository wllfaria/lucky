use crate::event::EventContext;

/// Common interface for event handlers to implement
pub trait Handler {
    /// function used to handle `xcb::x::KeyPressEvents`
    fn on_key_press(
        &mut self,
        _context: EventContext<xcb::x::KeyPressEvent>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
