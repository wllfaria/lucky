mod action;
mod command;
mod handler;
mod hover;
mod map_window;
mod unmap_window;

use crate::event::EventContext;
use action::ActionHandler;
use command::CommandHandler;
use handler::Handler;
use hover::HoverHandler;
use map_window::MapWindowHandler;
use unmap_window::UnmapWindowHandler;

pub struct Handlers {
    handlers: Vec<Box<dyn Handler>>,
}

impl Default for Handlers {
    fn default() -> Self {
        Handlers {
            handlers: vec![
                Box::<CommandHandler>::default(),
                Box::<ActionHandler>::default(),
                Box::<MapWindowHandler>::default(),
                Box::<UnmapWindowHandler>::default(),
                Box::<HoverHandler>::default(),
            ],
        }
    }
}

impl Handlers {
    pub fn on_key_press(&mut self, context: EventContext<xcb::x::KeyPressEvent>) {
        for handler in self.handlers.iter_mut() {
            match handler.on_key_press(context.clone()) {
                Ok(_) => {}
                Err(e) => tracing::error!("critical error happened: {e:?}"),
            }
        }
        tracing::debug!("key press handled correctly");
    }

    pub fn on_map_request(&mut self, context: EventContext<xcb::x::MapRequestEvent>) {
        for handler in self.handlers.iter_mut() {
            match handler.on_map_request(context.clone()) {
                Ok(_) => {}
                Err(e) => tracing::error!("critical error happened: {e:?}"),
            }
        }
        tracing::debug!("map request handled correctly");
    }

    pub fn on_destroy_notify(&mut self, context: EventContext<xcb::x::DestroyNotifyEvent>) {
        for handler in self.handlers.iter_mut() {
            match handler.on_destroy_notify(context.clone()) {
                Ok(_) => {}
                Err(e) => tracing::error!("critical error happened: {e:?}"),
            }
        }
        tracing::debug!("destroy notify handled correctly");
    }

    pub fn on_enter_notify(&mut self, context: EventContext<xcb::x::EnterNotifyEvent>) {
        for handler in self.handlers.iter_mut() {
            match handler.on_enter_notify(context.clone()) {
                Ok(_) => {}
                Err(e) => tracing::error!("critical error happened: {e:?}"),
            }
        }
        tracing::debug!("enter notify handled correctly");
    }

    pub fn on_unmap_notify(&mut self, context: EventContext<xcb::x::UnmapNotifyEvent>) {
        for handler in self.handlers.iter_mut() {
            match handler.on_unmap_notify(context.clone()) {
                Ok(_) => {}
                Err(e) => tracing::error!("critical error happened: {e:?}"),
            }
        }
        tracing::debug!("unmap handled correctly");
    }
}
