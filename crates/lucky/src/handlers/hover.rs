use crate::event::EventContext;
use crate::handlers::handler::Handler;

#[derive(Default)]
pub struct HoverHandler {}

impl Handler for HoverHandler {
    fn on_enter_notify(
        &mut self,
        context: EventContext<xcb::x::EnterNotifyEvent>,
    ) -> anyhow::Result<()> {
        let window = context.event.event();
        context.screen_manager.borrow_mut().focus_client(window);

        context
            .layout_manager
            .display_screens(&context.screen_manager, context.decorator)?;

        Ok(())
    }
}
