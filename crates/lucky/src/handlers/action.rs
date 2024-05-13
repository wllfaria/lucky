use config::{keysyms::Keysym, AvailableActions};
use xcb::{
    x::{DestroyWindow, KillClient, SetCloseDownMode},
    Xid,
};

use crate::{event::EventContext, handler::Handler};

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
                match action.action() {
                    AvailableActions::Close => {
                        if let Some(window) = context.clients.borrow_mut().active_client() {
                            context.conn.send_request(&SetCloseDownMode {
                                mode: xcb::x::CloseDown::DestroyAll,
                            });
                            context.conn.send_request(&KillClient {
                                resource: window.resource_id(),
                            });
                            context
                                .conn
                                .check_request(context.conn.send_request_checked(
                                    &DestroyWindow {
                                        window: context.event.root(),
                                    },
                                ))?;
                        }
                    }
                    AvailableActions::FocusLeft => todo!(),
                    AvailableActions::FocusDown => todo!(),
                    AvailableActions::FocusUp => todo!(),
                    AvailableActions::FocusRight => todo!(),
                    AvailableActions::MoveLeft => todo!(),
                    AvailableActions::MoveDown => todo!(),
                    AvailableActions::MoveUp => todo!(),
                    AvailableActions::MoveRight => todo!(),
                    AvailableActions::Reload => todo!(),
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
