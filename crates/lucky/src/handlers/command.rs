use config::keysyms::Keysym;

use crate::{event::EventContext, handler::Handler};

#[derive(Default)]
pub struct CommandHandler {}

impl Handler for CommandHandler {
    fn on_key_press(&mut self, context: EventContext<xcb::x::KeyPressEvent>) -> anyhow::Result<()> {
        let keysym = context
            .keyboard
            .state
            .key_get_one_sym(context.event.detail().into());
        if let Ok(keysym) = Keysym::try_from(keysym) {
            if let Some(command) = context
                .config
                .commands()
                .iter()
                .find(|command| command.key().eq(&keysym))
            {
                std::process::Command::new(command.command())
                    .spawn()
                    .unwrap();
            }
        }
        Ok(())
    }
}
