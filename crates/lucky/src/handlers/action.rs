use crate::event::EventContext;
use crate::handlers::handler::Handler;
use crate::screen_manager::Direction;
use config::keysyms::Keysym;
use config::AvailableActions;

#[derive(Default, Debug)]
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
                    Quit => std::process::exit(1),
                    Close => self.handle_close(&context)?,
                    FocusLeft => self.handle_focus_client(&context, Direction::Left)?,
                    FocusDown => self.handle_focus_client(&context, Direction::Down)?,
                    FocusUp => self.handle_focus_client(&context, Direction::Up)?,
                    FocusRight => self.handle_focus_client(&context, Direction::Right)?,
                    MoveLeft => self.handle_move_client(&context, Direction::Left)?,
                    MoveDown => self.handle_move_client(&context, Direction::Down)?,
                    MoveUp => self.handle_move_client(&context, Direction::Up)?,
                    MoveRight => self.handle_move_client(&context, Direction::Right)?,
                    Reload => context.action_tx.send(action.action())?,
                    Fullscreen => self.handle_fullscreen(&context)?,
                    Workspace1 => self.handle_change_workspace(&context, action.action())?,
                    Workspace2 => self.handle_change_workspace(&context, action.action())?,
                    Workspace3 => self.handle_change_workspace(&context, action.action())?,
                    Workspace4 => self.handle_change_workspace(&context, action.action())?,
                    Workspace5 => self.handle_change_workspace(&context, action.action())?,
                    Workspace6 => self.handle_change_workspace(&context, action.action())?,
                    Workspace7 => self.handle_change_workspace(&context, action.action())?,
                    Workspace8 => self.handle_change_workspace(&context, action.action())?,
                    Workspace9 => self.handle_change_workspace(&context, action.action())?,
                    MoveToWorkspace1 => self.handle_move_to_workspace(&context, action.action())?,
                    MoveToWorkspace2 => self.handle_move_to_workspace(&context, action.action())?,
                    MoveToWorkspace3 => self.handle_move_to_workspace(&context, action.action())?,
                    MoveToWorkspace4 => self.handle_move_to_workspace(&context, action.action())?,
                    MoveToWorkspace5 => self.handle_move_to_workspace(&context, action.action())?,
                    MoveToWorkspace6 => self.handle_move_to_workspace(&context, action.action())?,
                    MoveToWorkspace7 => self.handle_move_to_workspace(&context, action.action())?,
                    MoveToWorkspace8 => self.handle_move_to_workspace(&context, action.action())?,
                    MoveToWorkspace9 => self.handle_move_to_workspace(&context, action.action())?,
                }
            }
        }

        context
            .screen_manager
            .borrow_mut()
            .update_atoms(context.atoms, &context.conn);

        Ok(())
    }
}

impl ActionHandler {
    fn handle_close(&self, context: &EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let mut screen_manager = context.screen_manager.borrow_mut();
        if let Some(client) = screen_manager.close_focused_client()? {
            drop(screen_manager);
            match context.layout_manager.close_client(&client, context.atoms) {
                Ok(_) => {
                    tracing::debug!(
                        "focus left handled correctly for window {:?}",
                        context.event.event()
                    );
                }
                Err(e) => return Err(e),
            };

            match context
                .layout_manager
                .display_screens(&context.screen_manager, context.decorator)
            {
                Ok(_) => {
                    tracing::debug!("displayed all clients successfully");
                    return Ok(());
                }
                Err(e) => {
                    tracing::error!(
                        "failed to display the available windows: {:?}",
                        context.event.event()
                    );
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    fn handle_focus_client(
        &self,
        context: &EventContext<xcb::x::KeyPressEvent>,
        direction: Direction,
    ) -> anyhow::Result<()> {
        match context.layout_manager.change_focus(context, direction) {
            Ok(_) => {
                tracing::debug!(
                    "focus left handled correctly for window {:?}",
                    context.event.event()
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    "error while focusing client {:?} left",
                    context.event.event()
                );
                Err(e)
            }
        }
    }

    fn handle_move_client(
        &self,
        context: &EventContext<xcb::x::KeyPressEvent>,
        direction: Direction,
    ) -> anyhow::Result<()> {
        match context.layout_manager.move_client(context, direction) {
            Ok(_) => {
                tracing::debug!(
                    "moving left handled correctly for window {:?}",
                    context.event.event()
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!("error while moving client {:?} left", context.event.event());
                Err(e)
            }
        }
    }

    fn handle_change_workspace(
        &self,
        context: &EventContext<xcb::x::KeyPressEvent>,
        action: AvailableActions,
    ) -> anyhow::Result<()> {
        match context.layout_manager.change_workspace(context, action) {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!(
                    "error while changing workspace {:?} ",
                    context.event.event()
                );
                Err(e)
            }
        }
    }

    fn handle_move_to_workspace(
        &self,
        context: &EventContext<xcb::x::KeyPressEvent>,
        action: AvailableActions,
    ) -> anyhow::Result<()> {
        match context.layout_manager.move_to_workspace(context, action) {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!(
                    "error while moving client to workspace {:?} ",
                    context.event.event()
                );
                Err(e)
            }
        }
    }

    fn handle_fullscreen(
        &self,
        _context: &EventContext<xcb::x::KeyPressEvent>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
