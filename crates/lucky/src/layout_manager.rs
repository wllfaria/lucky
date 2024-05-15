mod master_layout;

use crate::{
    clients::{Client, Clients, WorkspaceLayout},
    event::EventContext,
    layout_manager::master_layout::MasterLayout,
};
use config::Config;
use std::{cell::RefCell, rc::Rc, sync::Arc};
use xcb::Xid;

pub struct LayoutManager {
    config: Rc<Config>,
    conn: Arc<xcb::Connection>,
}

impl LayoutManager {
    pub fn new(conn: Arc<xcb::Connection>, config: Rc<Config>) -> Self {
        LayoutManager { config, conn }
    }

    pub fn enable_client_events(&self, window: xcb::x::Window) -> anyhow::Result<()> {
        self.conn.send_request(&xcb::x::ChangeWindowAttributes {
            window,
            value_list: &[(xcb::x::Cw::EventMask(
                xcb::x::EventMask::PROPERTY_CHANGE
                    | xcb::x::EventMask::SUBSTRUCTURE_NOTIFY
                    | xcb::x::EventMask::ENTER_WINDOW,
            ))],
        });

        Ok(())
    }

    pub fn display_clients(&self, clients: &Rc<RefCell<Clients>>) -> anyhow::Result<()> {
        let clients = clients.borrow();
        let active_workspace = clients.get_active_workspace();

        match active_workspace.layout {
            WorkspaceLayout::Master => {
                MasterLayout::display_clients(&self.conn, &clients, &self.config)?
            }
        }

        Ok(())
    }

    /// Closes an open client.
    ///
    /// we need to query the `WM_PROTOCOLS` defined on the window to define how to properly
    /// close it. Modern clients will usually support `WM_DELETE_WINDOW`, and in this case
    /// we can close by sending a `ClientMessageEvent`, otherwise we have to manually close
    /// it through the `DestroyWindow` event.
    pub fn close_client(
        &self,
        client: Client,
        context: &EventContext<xcb::x::KeyPressEvent>,
    ) -> anyhow::Result<()> {
        let cookie = self.conn.send_request(&xcb::x::GetProperty {
            delete: false,
            window: client.window,
            property: context.atoms.wm_protocols,
            r#type: xcb::x::ATOM_ATOM,
            long_offset: 0,
            long_length: 1024,
        });
        let protocols = self.conn.wait_for_reply(cookie)?;
        let atoms: &[xcb::x::Atom] = protocols.value();

        if atoms
            .iter()
            .any(|&atom| atom == context.atoms.wm_delete_window)
        {
            let event = xcb::x::ClientMessageEvent::new(
                client.window,
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
                destination: xcb::x::SendEventDest::Window(client.window),
                event_mask: xcb::x::EventMask::NO_EVENT,
                event: &event,
            });

            self.conn.send_request(&xcb::x::DestroyWindow {
                window: client.frame,
            });

            self.conn.flush()?;
        } else {
            self.conn.send_request(&xcb::x::DestroyWindow {
                window: client.frame,
            });

            self.conn.flush()?;
        }

        Ok(())
    }
}
