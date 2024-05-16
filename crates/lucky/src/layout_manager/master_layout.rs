use crate::{
    clients::{Client, Screen},
    decorator::Decorator,
    screen_manager::Position,
};
use config::Config;
use std::{
    ops::{Div, Mul, Sub},
    rc::Rc,
    sync::Arc,
};

pub struct TallLayout {}

impl TallLayout {
    pub fn display_clients(
        conn: &Arc<xcb::Connection>,
        config: &Rc<Config>,
        screen: &Screen,
        clients: Vec<&Client>,
        focused_client: Option<&Client>,
        decorator: &Decorator,
    ) -> anyhow::Result<()> {
        let visible_clients_len = clients.len();

        let master_width = if visible_clients_len.eq(&1) {
            screen.position.width
        } else {
            screen.position.width.div(2)
        };

        for (i, client) in clients.iter().enumerate() {
            decorator.unfocus_client(client)?;
            match i {
                0 => Self::display_master_client(conn, client, screen, master_width, config),
                _ => Self::display_sibling_client(
                    conn,
                    client,
                    screen,
                    i,
                    visible_clients_len,
                    master_width,
                    config,
                ),
            }
        }

        if let Some(focused_client) = focused_client {
            let client = clients
                .iter()
                .find(|&&client| client.eq(focused_client))
                .expect("focused client must exist within all the clients");
            if focused_client.eq(client) {
                decorator.focus_client(client)?;
            }
        }

        conn.flush()?;
        Ok(())
    }

    fn display_master_client(
        conn: &Arc<xcb::Connection>,
        client: &Client,
        screen: &Screen,
        master_width: u32,
        config: &Rc<Config>,
    ) {
        Self::configure_window(
            conn,
            client.frame,
            Position::new(
                0,
                0,
                master_width.sub(config.border_width().mul(2) as u32),
                screen
                    .position
                    .height
                    .sub(config.border_width().mul(2) as u32),
            ),
        );
        Self::configure_window(
            conn,
            client.window,
            Position::new(
                0,
                0,
                master_width.sub(config.border_width() as u32),
                screen.position.height.sub(config.border_width() as u32),
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
        screen: &Screen,
        index: usize,
        total: usize,
        master_width: u32,
        config: &Rc<Config>,
    ) {
        let width = screen.position.width.sub(master_width);
        let total_siblings = total.sub(1);
        let height = screen.position.height.div(total_siblings as u32);
        let sibling_index = index.sub(1);

        Self::configure_window(
            conn,
            client.frame,
            Position::new(
                master_width as i32,
                height.mul(sibling_index as u32) as i32,
                width.sub(config.border_width().mul(2) as u32),
                height.sub(config.border_width().mul(2) as u32),
            ),
        );
        Self::configure_window(conn, client.window, Position::new(0, 0, width, height));
        conn.send_request(&xcb::x::MapWindow {
            window: client.window,
        });
        conn.send_request(&xcb::x::MapWindow {
            window: client.frame,
        });
    }

    fn configure_window(conn: &Arc<xcb::Connection>, window: xcb::x::Window, client_pos: Position) {
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
}
