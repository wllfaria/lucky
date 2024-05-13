use config::AvailableActions;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use xcb::{
    x::{ChangeWindowAttributes, ConfigureWindow, MapWindow},
    Xid,
};

#[derive(Debug, PartialEq)]
pub struct Client {
    pub window: xcb::x::Window,
    pub workspace: Option<u8>,
    pub visible: bool,
}

pub struct Clients {
    conn: Arc<xcb::Connection>,
    pub active_workspace: u8,
    pub clients: VecDeque<Client>,
    pub active_windows: HashMap<u8, Option<xcb::x::Window>>,
}

impl Clients {
    pub fn new(conn: Arc<xcb::Connection>) -> Self {
        Self {
            conn,
            active_workspace: 1,
            clients: VecDeque::new(),
            active_windows: HashMap::default(),
        }
    }

    pub fn create(&mut self, window: xcb::x::Window) -> anyhow::Result<()> {
        self.clients.push_front(Client {
            window,
            visible: true,
            workspace: Some(self.active_workspace),
        });

        self.enable_events(window)?;

        self.active_windows
            .insert(self.active_workspace, Some(window));

        Ok(())
    }

    pub fn display(&mut self, window: xcb::x::Window) -> anyhow::Result<()> {
        for client in self.clients.iter() {
            if client.window.eq(&window) && client.visible {
                let root_win = self
                    .conn
                    .get_setup()
                    .roots()
                    .next()
                    .expect("should have at least a single window");
                self.conn
                    .check_request(self.conn.send_request_checked(&ConfigureWindow {
                        window: client.window,
                        value_list: &[
                            xcb::x::ConfigWindow::X(0),
                            xcb::x::ConfigWindow::Y(0),
                            xcb::x::ConfigWindow::Width(root_win.width_in_pixels().into()),
                            xcb::x::ConfigWindow::Height(root_win.height_in_pixels().into()),
                        ],
                    }))?;
                self.conn
                    .check_request(self.conn.send_request_checked(&MapWindow { window }))?;
            }
        }

        Ok(())
    }

    fn enable_events(&mut self, window: xcb::x::Window) -> anyhow::Result<()> {
        self.conn
            .check_request(self.conn.send_request_checked(&ChangeWindowAttributes {
                window,
                value_list: &[(xcb::x::Cw::EventMask(
                    xcb::x::EventMask::PROPERTY_CHANGE
                        | xcb::x::EventMask::SUBSTRUCTURE_NOTIFY
                        | xcb::x::EventMask::ENTER_WINDOW,
                ))],
            }))?;

        Ok(())
    }

    pub fn destroy(&mut self, window: xcb::x::Window) -> anyhow::Result<()> {
        self.clients.retain(|client| client.window.ne(&window));

        if self.active_client().eq(&Some(window)) {
            let active_client = self.clients.iter().next().map(|c| c.window);
            self.set_active_client(active_client)?;
        }

        Ok(())
    }

    pub fn active_client(&mut self) -> Option<xcb::x::Window> {
        self.active_windows
            .entry(self.active_workspace)
            .or_insert(None)
            .to_owned()
    }

    pub fn set_active_client(&mut self, window: Option<xcb::x::Window>) -> anyhow::Result<()> {
        if let Some(window) = window {
            self.active_windows
                .insert(self.active_workspace, Some(window));
        }

        self.conn.flush()?;

        Ok(())
    }

    pub fn handle_action(&mut self, action: &AvailableActions) -> anyhow::Result<()> {
        tracing::debug!("{:?}", self.clients);
        let client = self.active_client();

        if let Some(window) = client {
            match action {
                AvailableActions::Close => {
                    let wm_protocols = self
                        .conn
                        .wait_for_reply(self.conn.send_request(&xcb::x::InternAtom {
                            only_if_exists: true,
                            name: b"WM_PROTOCOLS",
                        }))?
                        .atom();

                    let wm_del_window = self
                        .conn
                        .wait_for_reply(self.conn.send_request(&xcb::x::InternAtom {
                            only_if_exists: true,
                            name: b"WM_DELETE_WINDOW",
                        }))?
                        .atom();

                    self.conn.check_request(self.conn.send_request_checked(
                        &xcb::x::ChangeProperty {
                            mode: xcb::x::PropMode::Replace,
                            window,
                            property: wm_protocols,
                            r#type: xcb::x::ATOM_ATOM,
                            data: &[wm_del_window],
                        },
                    ))?;

                    self.conn.flush()?;

                    let event = xcb::x::ClientMessageEvent::new(
                        window,
                        wm_protocols,
                        xcb::x::ClientMessageData::Data32([
                            wm_del_window.resource_id(),
                            xcb::x::CURRENT_TIME,
                            0,
                            0,
                            0,
                        ]),
                    );

                    self.conn.send_request(&xcb::x::SendEvent {
                        propagate: false,
                        destination: xcb::x::SendEventDest::Window(window),
                        event_mask: xcb::x::EventMask::NO_EVENT,
                        event: &event,
                    });

                    self.conn.flush()?;
                    self.destroy(window)?;
                }
                AvailableActions::MoveUp => {}
                _ => {}
            }
        }

        Ok(())
    }
}
