use crate::event::EventContext;
use crate::handlers::handler::Handler;
use config::keysyms::Keysym;

#[derive(Default)]
pub struct ActionHandler {}

impl Handler for ActionHandler {
    fn on_key_press(&mut self, context: EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let keysym = context
            .keyboard
            .state
            .key_get_one_sym(context.event.detail().into());

        if let Ok(keysym) = Keysym::try_from(keysym) {
            if let Some(action) = context
                .config
                .actions()
                .iter()
                .find(|action| action.key().eq(&keysym))
            {
                context
                    .clients
                    .borrow_mut()
                    .handle_action(action.action(), &context)?;
            }
        }

        Ok(())
    }
}
