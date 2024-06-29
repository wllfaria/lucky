#[macro_export]
macro_rules! xcb_get_prop {
    ($conn:expr, $client:expr, $atom:expr, $len:expr, $type:expr) => {
        $conn.wait_for_reply($conn.send_request(&xcb::x::GetProperty {
            delete: false,
            window: $client,
            property: $atom,
            r#type: $type,
            long_offset: 0,
            long_length: $len,
        }))
    };
    ($conn:expr, $client:expr, $atom:expr, $len:expr) => {
        $conn.wait_for_reply($conn.send_request(&xcb::x::GetProperty {
            delete: false,
            window: $client,
            property: $atom,
            r#type: xcb::x::ATOM_ATOM,
            long_offset: 0,
            long_length: $len,
        }))
    };
}

#[macro_export]
macro_rules! xcb_change_prop {
    ($conn:expr, $window:expr, $mode:expr, $type:expr, $property:expr, $data:expr$(,)?) => {
        $conn.send_and_check_request(&xcb::x::ChangeProperty {
            window: $window,
            mode: $mode,
            r#type: $type,
            property: $property,
            data: $data,
        })
    };
}

#[macro_export]
macro_rules! xcb_remove_prop {
    ($conn:expr, $window:expr, $type:expr, $prop:expr, $atom:expr$(,)?) => {{
        let prop = xcb_get_prop!($conn, $window, $prop, 4096, xcb::x::ATOM_ANY);
        if let Ok(prop) = prop {
            let atoms = prop
                .value::<xcb::x::Atom>()
                .into_iter()
                .filter(|&atom| *atom != $atom)
                .collect::<Vec<_>>();
            let prop_bytes = atoms
                .iter()
                .map(|atom| *atom.to_owned())
                .collect::<Vec<_>>();
            xcb_change_prop!(
                $conn,
                $window,
                xcb::x::PropMode::Replace,
                xcb::x::ATOM_ATOM,
                $prop,
                &prop_bytes
            )?
        }
        Ok(())
    }};
}

#[macro_export]
macro_rules! xcb_intern_atom {
    ($conn:expr, $name:expr) => {
        $conn
            .wait_for_reply($conn.send_request(&xcb::x::InternAtom {
                only_if_exists: false,
                name: $name,
            }))
            .expect("we failed to get an internal atom")
    };
}

#[macro_export]
macro_rules! xcb_reparent_win {
    ($conn:expr, $client:expr, $frame:expr) => {
        $conn.check_request($conn.send_request_checked(&xcb::x::ReparentWindow {
            window: $client,
            parent: $frame,
            x: 0,
            y: 0,
        }))
    };
}

#[macro_export]
macro_rules! xcb_change_attr {
    ($conn:expr, $client:expr, $attrs:expr) => {
        $conn.send_request(&xcb::x::ChangeWindowAttributes {
            window: $client,
            value_list: $attrs,
        })
    };
}

#[macro_export]
macro_rules! xcb_input_focus {
    ($conn:expr, $client:expr) => {
        $conn.send_request(&xcb::x::SetInputFocus {
            time: xcb::x::CURRENT_TIME,
            focus: $client,
            revert_to: xcb::x::InputFocus::Parent,
        });
    };
}

#[macro_export]
macro_rules! xcb_send_event {
    ($conn:expr, $dest:expr, $event:expr) => {
        $conn.send_request(&xcb::x::SendEvent {
            propagate: false,
            destination: $dest,
            event_mask: xcb::x::EventMask::NO_EVENT,
            event: $event,
        });
    };
    ($conn:expr, $dest:expr, $mask:expr, $event:expr) => {
        $conn.send_request(&xcb::x::SendEvent {
            propagate: false,
            destination: $dest,
            event_mask: $mask,
            event: $event,
        });
    };
}

#[macro_export]
macro_rules! xcb_unmap_win {
    ($conn:expr, $client:expr) => {
        $conn.send_request(&xcb::x::UnmapWindow { window: $client });
    };
}

#[macro_export]
macro_rules! xcb_map_win {
    ($conn:expr, $client:expr) => {
        $conn.send_request(&xcb::x::MapWindow { window: $client });
    };
}

#[macro_export]
macro_rules! xcb_destroy_win {
    ($conn:expr, $client:expr) => {
        $conn.send_request(&xcb::x::DestroyWindow { window: $client });
    };
}

#[macro_export]
macro_rules! xcb_create_win {
    ($conn:expr, $root:expr, $position:expr, $border:expr, $value_list:expr$(,)?) => {{
        let window: xcb::x::Window = $conn.generate_id();
        $conn.send_and_check_request(&xcb::x::CreateWindow {
            depth: xcb::x::COPY_FROM_PARENT as u8,
            wid: window,
            parent: $root,
            border_width: $border,
            class: xcb::x::WindowClass::InputOutput,
            x: $position.x as i16,
            y: $position.y as i16,
            width: $position.width as u16,
            height: $position.height as u16,
            value_list: $value_list,
            visual: xcb::x::COPY_FROM_PARENT,
        })?;

        window
    }};
    ($conn:expr, $root:expr, $position:expr, $value_list:expr$(,)?) => {{
        let window: xcb::x::Window = $conn.generate_id();
        $conn.send_and_check_request(&xcb::x::CreateWindow {
            depth: xcb::x::COPY_FROM_PARENT as u8,
            wid: window,
            parent: $root,
            border_width: 0,
            class: xcb::x::WindowClass::InputOutput,
            x: $position.x as i16,
            y: $position.y as i16,
            width: $position.width as u16,
            height: $position.height as u16,
            value_list: $value_list,
            visual: xcb::x::COPY_FROM_PARENT,
        })?;

        window
    }};
}

pub use xcb_change_attr;
pub use xcb_change_prop;
pub use xcb_create_win;
pub use xcb_destroy_win;
pub use xcb_get_prop;
pub use xcb_input_focus;
pub use xcb_intern_atom;
pub use xcb_map_win;
pub use xcb_remove_prop;
pub use xcb_reparent_win;
pub use xcb_send_event;
pub use xcb_unmap_win;
