mod action;
mod command;
mod map_window;

use crate::{event::EventContext, handler::Handler};
use action::ActionHandler;
use command::CommandHandler;
use map_window::MapWindowHandler;

pub struct Handlers {
    handlers: Vec<Box<dyn Handler>>,
}

impl Default for Handlers {
    fn default() -> Self {
        Self {
            handlers: vec![
                Box::<CommandHandler>::default(),
                Box::<ActionHandler>::default(),
                Box::<MapWindowHandler>::default(),
            ],
        }
    }
}

impl Handlers {
    pub fn on_key_press(&mut self, context: EventContext<xcb::x::KeyPressEvent>) {
        for handler in self.handlers.iter_mut() {
            handler.on_key_press(context.clone()).ok();
        }
    }

    pub fn on_map_request(&mut self, context: EventContext<xcb::x::MapRequestEvent>) {
        for handler in self.handlers.iter_mut() {
            handler.on_map_request(context.clone()).ok();
        }
    }

    pub fn on_destroy_notify(&mut self, context: EventContext<xcb::x::DestroyNotifyEvent>) {
        for handler in self.handlers.iter_mut() {
            handler.on_destroy_notify(context.clone()).ok();
        }
    }
}
