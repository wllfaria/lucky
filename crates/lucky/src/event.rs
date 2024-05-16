use crate::{
    atoms::Atoms, decorator::Decorator, keyboard::Keyboard, layout_manager::LayoutManager,
    screen_manager::ScreenManager,
};
use config::{AvailableActions, Config};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{mpsc::Sender, Arc},
};

/// The context required by `Handlers` to properly handle all sorts of events, this context can and
/// should be clones as many handlers may be interested in the same kind of events.
pub struct EventContext<'ec, E> {
    /// The underlying event that needs to be handled, example: `xcb::x::KeyPressEvent`
    pub event: E,
    /// The connection to the X server
    pub conn: Arc<xcb::Connection>,
    /// The window manager configuration, used to get user defined parameters that can influence
    /// how events are handled
    pub config: Rc<RefCell<Config>>,
    /// The keyboard state retrieved from `XKB`, this is used to define which key is pressed
    /// through the Keycode we get from `xcb::x::KeyPressEvent`
    pub keyboard: &'ec Keyboard,
    /// The entity responsible for handling which screen and clients should handle events and to
    /// orchestrating their behavior
    pub screen_manager: Rc<RefCell<ScreenManager>>,
    /// Layout manager handles how to arrange the windows based on the current active layout for
    /// the during runtime
    pub layout_manager: &'ec LayoutManager,
    /// Decorator decorates a window with all the user-defined properties in the configuration file
    /// prior to handling the client to the layout manager
    pub decorator: &'ec Decorator,
    /// All the atoms that the window manager has cached and may be needed to handle an event
    pub atoms: &'ec Atoms,
    /// Channel where actions can be sent back to the window manager for actions that should affect
    /// global behavior, like `AvailableActions::Reload` for example. Which should reload the
    /// entire configuration for the window manager
    pub action_tx: Sender<AvailableActions>,
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
            screen_manager: self.screen_manager.clone(),
            atoms: self.atoms,
            decorator: self.decorator,
            layout_manager: self.layout_manager,
            action_tx: self.action_tx.clone(),
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
            screen_manager: self.screen_manager.clone(),
            atoms: self.atoms,
            decorator: self.decorator,
            layout_manager: self.layout_manager,
            action_tx: self.action_tx.clone(),
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
            screen_manager: self.screen_manager.clone(),
            atoms: self.atoms,
            decorator: self.decorator,
            layout_manager: self.layout_manager,
            action_tx: self.action_tx.clone(),
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
            screen_manager: self.screen_manager.clone(),
            atoms: self.atoms,
            decorator: self.decorator,
            layout_manager: self.layout_manager,
            action_tx: self.action_tx.clone(),
        }
    }
}

impl Clone for EventContext<'_, xcb::x::MapNotifyEvent> {
    fn clone(&self) -> Self {
        let event = xcb::x::MapNotifyEvent::new(
            self.event.event(),
            self.event.window(),
            self.event.override_redirect(),
        );

        Self {
            event,
            conn: self.conn.clone(),
            config: self.config.clone(),
            keyboard: self.keyboard,
            screen_manager: self.screen_manager.clone(),
            atoms: self.atoms,
            decorator: self.decorator,
            layout_manager: self.layout_manager,
            action_tx: self.action_tx.clone(),
        }
    }
}
