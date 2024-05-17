use crate::event::EventContext;
use crate::handlers::handler::Handler;
use config::keysyms::Keysym;

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
                .borrow()
                .commands()
                .iter()
                .find(|command| command.key().eq(&keysym))
            {
                match std::process::Command::new(command.command())
                    .args(command.args())
                    .spawn()
                {
                    Ok(_) => tracing::debug!("spawning command {:?} handled successfully", command),
                    Err(_) => {
                        tracing::error!("failed to spawn command {:?}", command);
                        anyhow::bail!("failed to spawn command {:?}", command);
                    }
                }
            }
        }
        Ok(())
    }
}
