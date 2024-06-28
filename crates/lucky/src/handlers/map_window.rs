use crate::event::EventContext;
use crate::handlers::handler::Handler;
use crate::screen::ReservedClient;
use crate::screen_manager::Position;

#[derive(Default)]
pub struct MapWindowHandler {}

impl MapWindowHandler {
    fn setup_reserved_client(
        &self,
        values: &[u32],
        context: &EventContext<xcb::x::MapRequestEvent>,
    ) {
        // this is the order that the values come for some reason, dont
        // ask me, i didn't make this decision, its on the spec.
        let left = values[0];
        let right = values[1];
        let top = values[2];
        let bottom = values[3];
        let left_start_y = values[4];
        let left_end_y = values[5];
        let right_start_y = values[6];
        let right_end_y = values[7];
        let top_start_x = values[8];
        let top_end_x = values[9];
        let bottom_start_x = values[10];
        let bottom_end_x = values[11];

        let mut screen_manager = context.screen_manager.borrow_mut();
        let screen_idx = screen_manager.active_screen_idx();
        let screen = screen_manager.screen_mut(screen_idx);
        let position = screen.position().clone();

        let position = match (left, bottom, top, right) {
            (_, _, _, _) if left > 0 => {
                screen.add_left_reserved_area(left);
                Position {
                    x: 0,
                    y: left_start_y as i32,
                    width: left,
                    height: left_end_y - left_start_y,
                }
            }
            (_, _, _, _) if bottom > 0 => {
                screen.add_bottom_reserved_area(bottom);
                Position {
                    x: bottom_start_x as i32,
                    y: position.bottom() - bottom as i32,
                    width: bottom_end_x - bottom_start_x,
                    height: bottom,
                }
            }
            (_, _, _, _) if top > 0 => {
                screen.add_top_reserved_area(top);
                Position {
                    x: top_start_x as i32,
                    y: 0,
                    width: top_end_x - top_start_x,
                    height: top,
                }
            }
            (_, _, _, _) if right > 0 => {
                screen.add_right_reserved_area(top);
                Position {
                    x: position.right() - right as i32,
                    y: right_start_y as i32,
                    width: right,
                    height: right_end_y - right_start_y,
                }
            }
            _ => unreachable!(),
        };

        let reserved_client = ReservedClient {
            window: context.event.window(),
            show_on_all_workspaces: true,
            workspace: 0,
            position,
        };

        screen.add_reserved_client(reserved_client);
    }
}

impl Handler for MapWindowHandler {
    fn on_map_request(
        &mut self,
        context: EventContext<xcb::x::MapRequestEvent>,
    ) -> anyhow::Result<()> {
        let window = context.event.window();

        let cookie = context.conn.send_request(&xcb::x::GetProperty {
            delete: false,
            window,
            property: context.atoms.net_wm_strut_partial,
            r#type: xcb::x::ATOM_CARDINAL,
            long_offset: 0,
            long_length: 12,
        });

        // if this window is requesting to reserve space onscreen, so we have
        // to do a few things to ensure it is handled properly, the first is
        // that this should not be handled as a regular client, but as a
        // reserved client.
        let is_reserving_space = context.conn.wait_for_reply(cookie).unwrap();
        if let Some(values) = is_reserving_space.value::<u32>().get(0..12) {
            self.setup_reserved_client(values, &context);
            context
                .layout_manager
                .display_screens(&context.screen_manager, context.decorator)
                .ok();
            return Ok(());
        }

        let frame = context.decorator.decorate_client(window)?;

        match context.layout_manager.enable_client_events(window) {
            Ok(_) => tracing::info!("enabled events for window: {:?}", window),
            Err(e) => {
                tracing::error!("failed to enable events for window: {:?}", window);
                return Err(e);
            }
        }
        match context.layout_manager.enable_client_events(frame) {
            Ok(_) => tracing::info!("enabled events for framw: {:?}", window),
            Err(e) => {
                tracing::error!("failed to enable events for frame: {:?}", window);
                return Err(e);
            }
        }

        context
            .screen_manager
            .borrow_mut()
            .create_client(frame, window);

        match context
            .layout_manager
            .display_screens(&context.screen_manager, context.decorator)
        {
            Ok(_) => tracing::info!(
                "displayed all windows after mapping a new client: {:?}",
                frame
            ),
            Err(e) => {
                tracing::error!(
                    "failed to display windows after mapping a new client: {:?}",
                    frame
                );
                return Err(e);
            }
        }

        Ok(())
    }
}
