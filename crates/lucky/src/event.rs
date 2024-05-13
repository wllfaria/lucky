use crate::keyboard::Keyboard;
use config::Config;
use std::sync::Arc;

/// The context required by `Handlers` to properly handle all sorts of events, this context can and
/// should be clones as many handlers may be interested in the same kind of events.
pub struct EventContext<'ec, E> {
    /// The underlying event that needs to be handled, example: `xcb::x::KeyPressEvent`
    pub event: E,
    /// The connection to the X server
    pub conn: Arc<xcb::Connection>,
    /// The window manager configuration, used to get user defined parameters that can influence
    /// how events are handled
    pub config: &'ec Config,
    /// The keyboard state retrieved from `XKB`, this is used to define which key is pressed
    /// through the Keycode we get from `xcb::x::KeyPressEvent`
    pub keyboard: &'ec Keyboard,
}

impl Clone for EventContext<'_, xcb::x::KeyPressEvent> {
    fn clone(&self) -> Self {
        let event = xcb::x::KeyPressEvent::new(
            self.event.detail(),
            self.event.time(),
            self.event.root(),
            self.event.event(),
            self.event.child(),
            self.event.root_x(),
            self.event.root_y(),
            self.event.event_x(),
            self.event.event_y(),
            self.event.state(),
            self.event.same_screen(),
        );
        Self {
            event,
            conn: self.conn.clone(),
            config: self.config,
            keyboard: self.keyboard,
        }
    }
}
