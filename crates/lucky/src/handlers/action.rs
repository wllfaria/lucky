use crate::event::EventContext;
use crate::handlers::handler::Handler;
use config::keysyms::Keysym;
use config::AvailableActions;

#[derive(Default)]
pub struct ActionHandler {}

impl Handler for ActionHandler {
    fn on_key_press(&mut self, context: EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let keysym = context
            .keyboard
            .state
            .key_get_one_sym(context.event.detail().into());

        if let Ok(keysym) = Keysym::try_from(keysym) {
            if let Some(action) = context.config.borrow().actions().iter().find(|action| {
                action.key().eq(&keysym) && context.event.state().eq(&action.modifiers().into())
            }) {
                tracing::debug!("{action:?}");
                match action.action() {
                    AvailableActions::Close => self.handle_close(&context)?,
                    AvailableActions::FocusLeft => self.handle_focus_left(&context)?,
                    AvailableActions::FocusDown => self.handle_focus_down(&context)?,
                    AvailableActions::FocusUp => self.handle_focus_up(&context)?,
                    AvailableActions::FocusRight => self.handle_focus_right(&context)?,
                    AvailableActions::MoveLeft => self.handle_move_left(&context)?,
                    AvailableActions::MoveDown => self.handle_move_down(&context)?,
                    AvailableActions::MoveUp => self.handle_move_up(&context)?,
                    AvailableActions::MoveRight => self.handle_move_right(&context)?,
                    AvailableActions::Reload => context.action_tx.send(action.action())?,
                    AvailableActions::Workspace1 => todo!(),
                    AvailableActions::Workspace2 => todo!(),
                    AvailableActions::Workspace3 => todo!(),
                    AvailableActions::Workspace4 => todo!(),
                    AvailableActions::Workspace5 => todo!(),
                    AvailableActions::Workspace6 => todo!(),
                    AvailableActions::Workspace7 => todo!(),
                    AvailableActions::Workspace8 => todo!(),
                    AvailableActions::Workspace9 => todo!(),
                    AvailableActions::Workspace0 => todo!(),
                }
            }
        }

        Ok(())
    }
}

impl ActionHandler {
    fn handle_close(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        if let Some(client) = screen_manager.close_focused_client()? {
            drop(screen_manager);
            context.layout_manager.close_client(client, context)?;
            context
                .layout_manager
                .display_screens(&context.screen_manager, context.decorator)?;
        }

        Ok(())
    }

    fn handle_focus_left(
        &self,
        context: &EventContext<xcb::x::KeyPressEvent>,
    ) -> anyhow::Result<()> {
        context.layout_manager.focus_left(context)?;
        Ok(())
    }

    fn handle_focus_down(
        &self,
        context: &EventContext<xcb::x::KeyPressEvent>,
    ) -> anyhow::Result<()> {
        context.layout_manager.focus_down(context)?;
        Ok(())
    }

    fn handle_focus_up(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        context.layout_manager.focus_up(context)?;
        Ok(())
    }

    fn handle_focus_right(
        &self,
        context: &EventContext<xcb::x::KeyPressEvent>,
    ) -> anyhow::Result<()> {
        context.layout_manager.focus_right(context)?;
        Ok(())
    }

    fn handle_move_left(
        &self,
        context: &EventContext<xcb::x::KeyPressEvent>,
    ) -> anyhow::Result<()> {
        context.layout_manager.move_left(context)?;
        Ok(())
    }

    fn handle_move_down(
        &self,
        context: &EventContext<xcb::x::KeyPressEvent>,
    ) -> anyhow::Result<()> {
        context.layout_manager.move_down(context)?;
        Ok(())
    }

    fn handle_move_up(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        context.layout_manager.move_up(context)?;
        Ok(())
    }

    fn handle_move_right(
        &self,
        context: &EventContext<xcb::x::KeyPressEvent>,
    ) -> anyhow::Result<()> {
        context.layout_manager.move_right(context)?;
        Ok(())
    }
}
