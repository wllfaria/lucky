use config::Config;
use std::{cell::RefCell, rc::Rc, sync::Arc};
use xcb::Xid;

use crate::{clients::Clients, event::EventContext};

pub struct LayoutManager {
    config: Rc<Config>,
    conn: Arc<xcb::Connection>,
}

impl LayoutManager {
    pub fn new(conn: Arc<xcb::Connection>, config: Rc<Config>) -> Self {
        LayoutManager { config, conn }
    }

    pub fn enable_client_events(&self, window: xcb::x::Window) -> anyhow::Result<()> {
        self.conn.check_request(self.conn.send_request_checked(
            &xcb::x::ChangeWindowAttributes {
                window,
                value_list: &[(xcb::x::Cw::EventMask(
                    xcb::x::EventMask::PROPERTY_CHANGE
                        | xcb::x::EventMask::SUBSTRUCTURE_NOTIFY
                        | xcb::x::EventMask::ENTER_WINDOW,
                ))],
            },
        ))?;

        Ok(())
    }

    pub fn display_clients(&self, clients: &Rc<RefCell<Clients>>) -> anyhow::Result<()> {
        let clients = clients.borrow();
        let active_workspace = clients.active_workspace;
        for client in clients.open_clients.iter() {
            if client.visible && client.workspace.eq(&active_workspace) {
                let root = self
                    .conn
                    .get_setup()
                    .roots()
                    .next()
                    .expect("should have at least one screen to manage");
                // TODO: we should actually map and divide the screen as needed
                self.conn.check_request(self.conn.send_request_checked(
                    &xcb::x::ConfigureWindow {
                        window: client.window,
                        value_list: &[
                            xcb::x::ConfigWindow::X(0),
                            xcb::x::ConfigWindow::Y(0),
                            xcb::x::ConfigWindow::Width(root.width_in_pixels().into()),
                            xcb::x::ConfigWindow::Height(root.height_in_pixels().into()),
                        ],
                    },
                ))?;

                self.conn
                    .check_request(self.conn.send_request_checked(&xcb::x::MapWindow {
                        window: client.window,
                    }))?;
            }
        }

        Ok(())
    }

    pub fn close_client(
        &self,
        client: xcb::x::Window,
        context: &EventContext<xcb::x::KeyPressEvent>,
    ) -> anyhow::Result<()> {
        let cookie = self.conn.send_request(&xcb::x::GetProperty {
            delete: false,
            window: client,
            property: context.atoms.wm_protocols,
            r#type: xcb::x::ATOM_ATOM,
            long_offset: 0,
            long_length: 1024,
        });

        let protocols_reply = self.conn.wait_for_reply(cookie)?;
        let atoms: &[xcb::x::Atom] = protocols_reply.value();

        let supports_delete = atoms
            .iter()
            .any(|&atom| atom == context.atoms.wm_delete_window);

        if supports_delete {
            let event = xcb::x::ClientMessageEvent::new(
                client,
                context.atoms.wm_protocols,
                xcb::x::ClientMessageData::Data32([
                    context.atoms.wm_delete_window.resource_id(),
                    xcb::x::CURRENT_TIME,
                    0,
                    0,
                    0,
                ]),
            );

            self.conn.send_request(&xcb::x::SendEvent {
                propagate: false,
                destination: xcb::x::SendEventDest::Window(client),
                event_mask: xcb::x::EventMask::NO_EVENT,
                event: &event,
            });
            self.conn.flush()?;
        } else {
            let cookie = self
                .conn
                .send_request_checked(&xcb::x::DestroyWindow { window: client });
            self.conn.check_request(cookie)?;
        }

        Ok(())
    }
}
