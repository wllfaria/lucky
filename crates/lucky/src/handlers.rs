mod action;
mod command;
mod handler;
mod hover;
mod map_window;
mod property_handler;
mod unmap_window;

use crate::event::EventContext;
use action::ActionHandler;
use command::CommandHandler;
use handler::Handler;
use hover::HoverHandler;
use map_window::MapWindowHandler;
use property_handler::PropertyHandler;
use unmap_window::UnmapWindowHandler;

#[derive(Debug)]
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
                Box::<PropertyHandler>::default(),
            ],
        }
    }
}

impl Handlers {
    #[tracing::instrument(skip_all, err)]
    pub fn on_key_press(
        &mut self,
        context: EventContext<xcb::x::KeyPressEvent>,
    ) -> anyhow::Result<()> {
        for handler in self.handlers.iter_mut() {
            handler.on_key_press(context.clone())?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub fn on_map_request(
        &mut self,
        context: EventContext<xcb::x::MapRequestEvent>,
    ) -> anyhow::Result<()> {
        for handler in self.handlers.iter_mut() {
            handler.on_map_request(context.clone())?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub fn on_destroy_notify(
        &mut self,
        context: EventContext<xcb::x::DestroyNotifyEvent>,
    ) -> anyhow::Result<()> {
        for handler in self.handlers.iter_mut() {
            handler.on_destroy_notify(context.clone())?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub fn on_enter_notify(
        &mut self,
        context: EventContext<xcb::x::EnterNotifyEvent>,
    ) -> anyhow::Result<()> {
        for handler in self.handlers.iter_mut() {
            handler.on_enter_notify(context.clone())?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub fn on_unmap_notify(
        &mut self,
        context: EventContext<xcb::x::UnmapNotifyEvent>,
    ) -> anyhow::Result<()> {
        for handler in self.handlers.iter_mut() {
            handler.on_unmap_notify(context.clone())?;
        }

        Ok(())
    }

    pub fn on_property_notify(
        &mut self,
        context: EventContext<xcb::x::PropertyNotifyEvent>,
    ) -> anyhow::Result<()> {
        for handler in self.handlers.iter_mut() {
            handler.on_property_notify(context.clone())?;
        }

        Ok(())
    }
}
