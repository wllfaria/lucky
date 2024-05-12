use std::sync::Arc;
use xcb::x::{GrabKey, GrabMode, ModMask};
use xkbcommon::xkb;

pub struct Keyboard {
    pub state: xkbcommon::xkb::State,
}

impl Keyboard {
    pub fn new(conn: Arc<xcb::Connection>, _root: xcb::x::Window) -> Self {
        conn.wait_for_reply(conn.send_request(&xcb::xkb::UseExtension {
            wanted_major: xkb::x11::MIN_MAJOR_XKB_VERSION,
            wanted_minor: xkb::x11::MIN_MINOR_XKB_VERSION,
        }))
        .expect("failed to initialize xkb extension");

        let events = xcb::xkb::EventType::NEW_KEYBOARD_NOTIFY
            | xcb::xkb::EventType::MAP_NOTIFY
            | xcb::xkb::EventType::STATE_NOTIFY;
        let map_parts = xcb::xkb::MapPart::KEY_TYPES
            | xcb::xkb::MapPart::KEY_SYMS
            | xcb::xkb::MapPart::MODIFIER_MAP
            | xcb::xkb::MapPart::EXPLICIT_COMPONENTS
            | xcb::xkb::MapPart::KEY_ACTIONS
            | xcb::xkb::MapPart::KEY_BEHAVIORS
            | xcb::xkb::MapPart::VIRTUAL_MODS
            | xcb::xkb::MapPart::VIRTUAL_MOD_MAP;

        conn.check_request(conn.send_request_checked(&xcb::xkb::SelectEvents {
            device_spec: unsafe { std::mem::transmute::<_, u32>(xcb::xkb::Id::UseCoreKbd) }
                as xcb::xkb::DeviceSpec,
            affect_which: events,
            clear: xcb::xkb::EventType::empty(),
            select_all: events,
            affect_map: map_parts,
            map: map_parts,
            details: &[],
        }))
        .expect("failed to select events from xkb");

        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let device_id = xkb::x11::get_core_keyboard_device_id(&conn);
        let keymap = xkb::x11::keymap_new_from_device(
            &context,
            &conn,
            device_id,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        );
        let state = xkbcommon::xkb::x11::state_new_from_device(&keymap, &conn, device_id);

        Self { state }
    }
}

fn grab_key(conn: Arc<xcb::Connection>, root: xcb::x::Window) {
    conn.check_request(conn.send_request_checked(&GrabKey {
        modifiers: ModMask::ANY,
        grab_window: root,
        key: xcb::x::GRAB_ANY,
        keyboard_mode: GrabMode::Async,
        pointer_mode: GrabMode::Async,
        owner_events: true,
    }))
    .expect("failed to grab keyboard key");
}
