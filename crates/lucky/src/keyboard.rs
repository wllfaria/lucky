use config::Config;
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};
use xcb::x::{GrabKey, GrabMode, ModMask};
use xkbcommon::xkb;

pub struct Keyboard {
    pub state: xkbcommon::xkb::State,
}

impl Keyboard {
    pub fn new(
        conn: &Arc<xcb::Connection>,
        config: Rc<RefCell<Config>>,
        root: xcb::x::Window,
    ) -> anyhow::Result<Self> {
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
            device_spec: xcb::xkb::Id::UseCoreKbd as xcb::xkb::DeviceSpec,
            affect_which: events,
            clear: xcb::xkb::EventType::empty(),
            select_all: events,
            affect_map: map_parts,
            map: map_parts,
            details: &[],
        }))
        .expect("failed to select events from xkb");

        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let device_id = xkb::x11::get_core_keyboard_device_id(conn);
        let keymap = xkb::x11::keymap_new_from_device(
            &context,
            conn,
            device_id,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        );
        let state = xkbcommon::xkb::x11::state_new_from_device(&keymap, conn, device_id);
        let mut keycode_map = HashMap::new();

        keymap.key_for_each(|_, keycode| {
            if let Some(keysym) = state.key_get_one_sym(keycode).name() {
                keycode_map.insert(keysym, keycode.raw());
            }
        });

        for action in config.borrow().actions().iter() {
            let keycode = match keycode_map.get(action.key().canonical_name()) {
                Some(e) => *e as u8,
                None => {
                    tracing::error!("failed to grab key: {}", action.key());
                    anyhow::bail!(format!("failed to grab key: {}", action.key()))
                }
            };
            grab_key(conn.clone(), action.modifiers().inner(), keycode, root);
        }

        for command in config.borrow().commands().iter() {
            let keycode = match keycode_map.get(command.key().canonical_name()) {
                Some(e) => *e as u8,
                None => {
                    tracing::error!("failed to grab key: {}", command.key());
                    anyhow::bail!(format!("failed to grab key: {}", command.key()))
                }
            };
            grab_key(conn.clone(), command.modifiers(), keycode, root);
        }

        Ok(Keyboard { state })
    }
}

fn grab_key(
    conn: Arc<xcb::Connection>,
    modifiers: xkbcommon::xkb::ModMask,
    key: u8,
    grab_window: xcb::x::Window,
) {
    conn.check_request(
        conn.send_request_checked(&GrabKey {
            modifiers: ModMask::from_bits(modifiers)
                .expect("no invalid modifiers should be exist at this point"),
            grab_window,
            key,
            keyboard_mode: GrabMode::Async,
            pointer_mode: GrabMode::Async,
            owner_events: true,
        }),
    )
    .expect("failed to grab keyboard key");
}
