use crate::{atoms::Atoms, clients::Clients, keyboard::Keyboard};
use config::Config;
use std::{cell::RefCell, rc::Rc, sync::Arc};

/// The context required by `Handlers` to properly handle all sorts of events, this context can and
/// should be clones as many handlers may be interested in the same kind of events.
pub struct EventContext<'ec, E> {
    /// The underlying event that needs to be handled, example: `xcb::x::KeyPressEvent`
    pub event: E,
    /// The connection to the X server
    pub conn: Arc<xcb::Connection>,
    /// The window manager configuration, used to get user defined parameters that can influence
    /// how events are handled
    pub config: Rc<Config>,
    /// The keyboard state retrieved from `XKB`, this is used to define which key is pressed
    /// through the Keycode we get from `xcb::x::KeyPressEvent`
    pub keyboard: &'ec Keyboard,
    /// All the clients currently being handled by the window manager, clients are how we call all
    /// the windows opened to avoid naming conflicts with `xcb::x::Window`
    pub clients: Rc<RefCell<Clients>>,
    /// All the atoms that the window manager has cached and may be needed to handle an event
    pub atoms: &'ec Atoms,
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
            config: self.config.clone(),
            keyboard: self.keyboard,
            clients: self.clients.clone(),
            atoms: self.atoms,
        }
    }
}

impl Clone for EventContext<'_, xcb::x::MapRequestEvent> {
    fn clone(&self) -> Self {
        let event = xcb::x::MapRequestEvent::new(self.event.parent(), self.event.window());

        Self {
            event,
            conn: self.conn.clone(),
            config: self.config.clone(),
            keyboard: self.keyboard,
            clients: self.clients.clone(),
            atoms: self.atoms,
        }
    }
}

impl Clone for EventContext<'_, xcb::x::DestroyNotifyEvent> {
    fn clone(&self) -> Self {
        let event = xcb::x::DestroyNotifyEvent::new(self.event.event(), self.event.window());

        Self {
            event,
            conn: self.conn.clone(),
            config: self.config.clone(),
            keyboard: self.keyboard,
            clients: self.clients.clone(),
            atoms: self.atoms,
        }
    }
}

impl Clone for EventContext<'_, xcb::x::EnterNotifyEvent> {
    fn clone(&self) -> Self {
        let event = xcb::x::EnterNotifyEvent::new(
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
            self.event.mode(),
            self.event.same_screen_focus(),
        );

        Self {
            event,
            conn: self.conn.clone(),
            config: self.config.clone(),
            keyboard: self.keyboard,
            clients: self.clients.clone(),
            atoms: self.atoms,
        }
    }
}
