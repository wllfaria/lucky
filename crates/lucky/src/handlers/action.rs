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
                use AvailableActions::*;
                match action.action() {
                    Close => self.handle_close(&context)?,
                    FocusLeft => self.handle_focus_left(&context)?,
                    FocusDown => self.handle_focus_down(&context)?,
                    FocusUp => self.handle_focus_up(&context)?,
                    FocusRight => self.handle_focus_right(&context)?,
                    MoveLeft => self.handle_move_left(&context)?,
                    MoveDown => self.handle_move_down(&context)?,
                    MoveUp => self.handle_move_up(&context)?,
                    MoveRight => self.handle_move_right(&context)?,
                    Reload => context.action_tx.send(action.action())?,
                    Workspace1 => self.handle_change_workspace(&context, action.action())?,
                    Workspace2 => self.handle_change_workspace(&context, action.action())?,
                    Workspace3 => self.handle_change_workspace(&context, action.action())?,
                    Workspace4 => self.handle_change_workspace(&context, action.action())?,
                    Workspace5 => self.handle_change_workspace(&context, action.action())?,
                    Workspace6 => self.handle_change_workspace(&context, action.action())?,
                    Workspace7 => self.handle_change_workspace(&context, action.action())?,
                    Workspace8 => self.handle_change_workspace(&context, action.action())?,
                    Workspace9 => self.handle_change_workspace(&context, action.action())?,
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
            context.layout_manager.close_client(client, context.atoms)?;
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

    fn handle_change_workspace(
        &self,
        context: &EventContext<xcb::x::KeyPressEvent>,
        action: AvailableActions,
    ) -> anyhow::Result<()> {
        context.layout_manager.change_workspace(context, action)?;
        Ok(())
    }
}
