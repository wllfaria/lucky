mod action;
mod command;
mod map_window;

use action::ActionHandler;
use command::CommandHandler;
use map_window::MapWindowHandler;

use crate::{event::EventContext, handler::Handler};

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
        self.handlers
            .iter_mut()
            .for_each(|handler| handler.on_key_press(context.clone()).unwrap());
    }

    pub fn on_map_request(&mut self, context: EventContext<xcb::x::MapRequestEvent>) {
        self.handlers
            .iter_mut()
            .for_each(|handler| handler.on_map_request(context.clone()).unwrap());
    }
}
