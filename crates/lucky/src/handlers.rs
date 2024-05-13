mod command;

use command::CommandHandler;

use crate::{event::EventContext, handler::Handler};

pub struct Handlers {
    handlers: Vec<Box<dyn Handler>>,
}

impl Default for Handlers {
    fn default() -> Self {
        Self {
            handlers: vec![Box::<CommandHandler>::default()],
        }
    }
}

impl Handlers {
    pub fn on_key_press(&mut self, context: EventContext<xcb::x::KeyPressEvent>) {
        self.handlers
            .iter_mut()
            .for_each(|handler| handler.on_key_press(context.clone()).unwrap());
    }
}
