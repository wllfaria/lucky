use crate::event::EventContext;

/// Common interface for event handlers to implement
pub trait Handler {
    /// Function used to handle `xcb::x::KeyPressEvent`s
    fn on_key_press(
        &mut self,
        _context: EventContext<xcb::x::KeyPressEvent>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    /// Function used to handle `xcb::x::MapRequestEvent`s
    fn on_map_request(
        &mut self,
        _context: EventContext<xcb::x::MapRequestEvent>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    /// Function used to handle `xcb::x::DestroyNotifyEvent`s
    fn on_destroy_notify(
        &mut self,
        _context: EventContext<xcb::x::DestroyNotifyEvent>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
