use crate::event::EventContext;
use crate::handlers::handler::Handler;

#[derive(Default, Debug)]
pub struct HoverHandler {}

impl Handler for HoverHandler {
    fn on_enter_notify(
        &mut self,
        context: EventContext<xcb::x::EnterNotifyEvent>,
    ) -> anyhow::Result<()> {
        if context.config.borrow().focus_follow_mouse() {
            let window = context.event.event();
            context.screen_manager.borrow_mut().focus_client(window);
            context
                .layout_manager
                .display_screens(&context.screen_manager, context.decorator)?;
        }

        context
            .screen_manager
            .borrow_mut()
            .update_atoms(context.atoms, &context.conn);

        Ok(())
    }
}
