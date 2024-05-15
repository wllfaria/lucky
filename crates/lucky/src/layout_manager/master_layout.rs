use crate::clients::{Client, Clients};
use config::Config;
use std::{
    ops::{Div, Mul, Sub},
    rc::Rc,
    sync::Arc,
};

struct ClientPos {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl ClientPos {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        ClientPos {
            x,
            y,
            width,
            height,
        }
    }
}

pub struct MasterLayout {}

impl MasterLayout {
    pub fn display_clients(
        conn: &Arc<xcb::Connection>,
        clients: &Clients,
        config: &Rc<Config>,
    ) -> anyhow::Result<()> {
        let active_workspace = clients.get_active_workspace();
        let visible_clients: Vec<&Client> = clients
            .open_clients
            .iter()
            .filter(|c| c.workspace.eq(&active_workspace.id) && c.visible)
            .collect();
        let visible_clients_len = visible_clients.len();
        let screen = Self::get_screen(conn);
        let screen_width = screen.width_in_pixels();
        let screen_height = screen.height_in_pixels();

        let master_width = if visible_clients_len.eq(&1) {
            screen_width
        } else {
            screen_width.div(2)
        };

        for (i, c) in visible_clients.iter().enumerate() {
            match i {
                0 => Self::display_master_client(conn, c, screen, master_width, config),
                _ => Self::display_sibling_client(
                    conn,
                    c,
                    screen,
                    i,
                    visible_clients_len,
                    master_width,
                    config,
                ),
            }
        }

        conn.flush()?;
        Ok(())
    }

    fn display_master_client(
        conn: &Arc<xcb::Connection>,
        client: &Client,
        screen: &xcb::x::Screen,
        master_width: u16,
        config: &Rc<Config>,
    ) {
        Self::configure_window(
            conn,
            client.frame,
            ClientPos::new(
                0,
                0,
                master_width.sub(config.border_width().mul(2)).into(),
                screen
                    .height_in_pixels()
                    .sub(config.border_width().mul(2))
                    .into(),
            ),
        );
        Self::configure_window(
            conn,
            client.window,
            ClientPos::new(
                0,
                0,
                master_width.sub(config.border_width()).into(),
                screen.height_in_pixels().sub(config.border_width()).into(),
            ),
        );

        conn.send_request(&xcb::x::MapWindow {
            window: client.window,
        });
        conn.send_request(&xcb::x::MapWindow {
            window: client.frame,
        });
    }

    fn display_sibling_client(
        conn: &Arc<xcb::Connection>,
        client: &Client,
        screen: &xcb::x::Screen,
        index: usize,
        total: usize,
        master_width: u16,
        config: &Rc<Config>,
    ) {
        let width = screen.width_in_pixels().sub(master_width);
        let total_siblings = total.sub(1);
        let height = screen.height_in_pixels().div(total_siblings as u16);
        let sibling_index = index.sub(1);

        Self::configure_window(
            conn,
            client.frame,
            ClientPos::new(
                master_width.into(),
                height.mul(sibling_index as u16).into(),
                width.sub(config.border_width().mul(2)).into(),
                height.sub(config.border_width().mul(2)).into(),
            ),
        );
        Self::configure_window(
            conn,
            client.window,
            ClientPos::new(0, 0, width.into(), height.into()),
        );
        conn.send_request(&xcb::x::MapWindow {
            window: client.window,
        });
        conn.send_request(&xcb::x::MapWindow {
            window: client.frame,
        });
    }

    fn configure_window(
        conn: &Arc<xcb::Connection>,
        window: xcb::x::Window,
        client_pos: ClientPos,
    ) {
        conn.send_request(&xcb::x::ConfigureWindow {
            window,
            value_list: &[
                xcb::x::ConfigWindow::X(client_pos.x),
                xcb::x::ConfigWindow::Y(client_pos.y),
                xcb::x::ConfigWindow::Width(client_pos.width),
                xcb::x::ConfigWindow::Height(client_pos.height),
            ],
        });
    }

    fn get_screen(conn: &Arc<xcb::Connection>) -> &xcb::x::Screen {
        conn.get_setup()
            .roots()
            .next()
            .expect("should have at least one screen to manage")
    }
}
