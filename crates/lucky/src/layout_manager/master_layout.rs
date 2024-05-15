use std::{ops::Mul, rc::Rc, sync::Arc};

use config::Config;

use crate::clients::Clients;

pub struct MasterLayout {}

impl MasterLayout {
    pub fn display_clients(
        conn: &Arc<xcb::Connection>,
        clients: &Clients,
        config: &Rc<Config>,
    ) -> anyhow::Result<()> {
        let active_workspace = clients.get_active_workspace();
        for client in clients.open_clients.iter() {
            if client.visible && client.workspace.eq(&active_workspace.id) {
                let root = conn
                    .get_setup()
                    .roots()
                    .next()
                    .expect("should have at least one screen to manage");
                conn.send_request(&xcb::x::ConfigureWindow {
                    window: client.window,
                    value_list: &[
                        xcb::x::ConfigWindow::X(0),
                        xcb::x::ConfigWindow::Y(0),
                        xcb::x::ConfigWindow::Width(
                            root.width_in_pixels() as u32 - config.border_width().mul(2) as u32,
                        ),
                        xcb::x::ConfigWindow::Height(
                            root.height_in_pixels() as u32 - config.border_width().mul(2) as u32,
                        ),
                    ],
                });
                conn.send_request(&xcb::x::ConfigureWindow {
                    window: client.frame,
                    value_list: &[
                        xcb::x::ConfigWindow::X(0),
                        xcb::x::ConfigWindow::Y(0),
                        xcb::x::ConfigWindow::Width(
                            root.width_in_pixels() as u32 - config.border_width().mul(2) as u32,
                        ),
                        xcb::x::ConfigWindow::Height(
                            root.height_in_pixels() as u32 - config.border_width().mul(2) as u32,
                        ),
                    ],
                });
                let pixmap = conn.generate_id();
                let gc = conn.generate_id();
                conn.send_request(&xcb::x::CreatePixmap {
                    depth: root.root_depth(),
                    pid: pixmap,
                    drawable: xcb::x::Drawable::Window(root.root()),
                    width: root.width_in_pixels() - config.border_width().mul(2),
                    height: root.height_in_pixels() - config.border_width().mul(2),
                });

                conn.send_request(&xcb::x::CreateGc {
                    cid: gc,
                    drawable: xcb::x::Drawable::Pixmap(pixmap),
                    value_list: &[],
                });

                // we only map the window as it can take some time to initialize and properly run,
                // we later listen to MapNotify event and map the frame to that window
                // accordingly.
                conn.send_request(&xcb::x::MapWindow {
                    window: client.window,
                });

                conn.flush()?;
            }
        }

        Ok(())
    }
}
