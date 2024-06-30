use crate::atoms::Atoms;
use crate::position::Position;
use crate::screen::Screen;
use crate::xcb_utils::*;

use std::collections::HashMap;
use std::sync::Arc;

/// updates _NET_CURRENT_DESKTOP atom with the number of workspaces
/// on the active screen (monitor).
pub fn ewmh_set_number_of_desktops(
    conn: &Arc<xcb::Connection>,
    root: xcb::x::Window,
    screen: &Screen,
    atoms: &Atoms,
) -> anyhow::Result<(), xcb::ProtocolError> {
    xcb_change_prop!(
        conn,
        root,
        xcb::x::PropMode::Replace,
        xcb::x::ATOM_CARDINAL,
        atoms.net_number_of_desktops,
        &[screen.workspaces().len() as u32]
    )
}

/// updates _NET_CURRENT_DESKTOP atom with the id of the current active
/// workspace (0 indexed)
pub fn ewmh_set_current_desktop(
    conn: &Arc<xcb::Connection>,
    root: xcb::x::Window,
    screen: &Screen,
    atoms: &Atoms,
) -> anyhow::Result<(), xcb::ProtocolError> {
    xcb_change_prop!(
        conn,
        root,
        xcb::x::PropMode::Replace,
        xcb::x::ATOM_CARDINAL,
        atoms.net_current_desktop,
        &[screen.active_workspace_id() as u32]
    )
}

/// updates _NET_DESKTOP_VIEWPORT atom, this is a list of pairs (x,y)
/// of the starting position of each screen
pub fn ewmh_set_desktop_viewport(
    conn: &Arc<xcb::Connection>,
    root: xcb::x::Window,
    screens: &[Screen],
    atoms: &Atoms,
) -> anyhow::Result<(), xcb::ProtocolError> {
    xcb_change_prop!(
        conn,
        root,
        xcb::x::PropMode::Replace,
        xcb::x::ATOM_CARDINAL,
        atoms.net_desktop_viewport,
        &screens.iter().fold(vec![], |mut acc, s| {
            acc.push(s.position().x as u32);
            acc.push(s.position().y as u32);
            acc
        })
    )
}

/// updates _NET_DESKTOP_NAMES atom, this is a list of strings in utf8
/// with each name of each desktop as a byte array
pub fn ewmh_set_desktop_names(
    conn: &Arc<xcb::Connection>,
    root: xcb::x::Window,
    screen: &Screen,
    atoms: &Atoms,
) -> anyhow::Result<(), xcb::ProtocolError> {
    xcb_change_prop!(
        conn,
        root,
        xcb::x::PropMode::Replace,
        xcb::x::ATOM_STRING,
        atoms.net_desktop_names,
        &screen
            .workspaces()
            .iter()
            .map(|ws| ws.name().to_owned())
            .collect::<Vec<_>>()
            .iter()
            .flat_map(|s| s.bytes().chain(Some(0)))
            .collect::<Vec<_>>()
    )
}

/// updates _NET_WM_DESKTOP for all clients on all workspaces for the
/// current screen
pub fn ewmh_set_wm_desktop(
    conn: &Arc<xcb::Connection>,
    screen: &Screen,
    client_map: &HashMap<xcb::x::Window, crate::screen::Client>,
    atoms: &Atoms,
) -> anyhow::Result<(), xcb::ProtocolError> {
    for workspace in screen.workspaces() {
        for client in workspace.clients() {
            xcb_change_prop!(
                conn,
                client_map.get(client).unwrap().window,
                xcb::x::PropMode::Replace,
                xcb::x::ATOM_CARDINAL,
                atoms.net_wm_desktop,
                &[workspace.id() as u32],
            )?;
        }
    }
    Ok(())
}

/// Updates _NET_ACTIVE_WINDOW with the currently focused window.
/// the window ID of the currently active window or None if no window has
/// the focus.
pub fn ewmh_set_active_window(
    conn: &Arc<xcb::Connection>,
    root: xcb::x::Window,
    atoms: &Atoms,
    window: xcb::x::Window,
) -> anyhow::Result<(), xcb::ProtocolError> {
    tracing::debug!("setting active window to: {window:?}");
    xcb_change_prop!(
        conn,
        root,
        xcb::x::PropMode::Replace,
        xcb::x::ATOM_WINDOW,
        atoms.net_active_window,
        &[window]
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EwmhFocusAction {
    Focus,
    Unfocus,
}

/// appends _NET_WM_STATE_FOCUSED to the client window
pub fn ewmh_set_focus(
    conn: &Arc<xcb::Connection>,
    atoms: &Atoms,
    window: xcb::x::Window,
    action: EwmhFocusAction,
) -> anyhow::Result<(), xcb::ProtocolError> {
    match action {
        EwmhFocusAction::Focus => {
            xcb_change_prop!(
                conn,
                window,
                xcb::x::PropMode::Append,
                xcb::x::ATOM_ATOM,
                atoms.net_wm_state,
                &[atoms.net_wm_state_focused],
            )
        }
        EwmhFocusAction::Unfocus => {
            xcb_remove_prop!(
                conn,
                window,
                xcb::x::ATOM_ATOM,
                atoms.net_wm_state,
                atoms.net_wm_state_focused
            )
        }
    }
}

/// list all the clients currently managed by the window manager
/// by order of insertion
pub fn ewmh_set_client_list<'a, I>(
    conn: &Arc<xcb::Connection>,
    root: xcb::x::Window,
    clients: I,
    atoms: &Atoms,
) -> anyhow::Result<(), xcb::ProtocolError>
where
    I: IntoIterator<Item = &'a xcb::x::Window>,
    &'a xcb::x::Window: Copy,
{
    xcb_change_prop!(
        conn,
        root,
        xcb::x::PropMode::Replace,
        xcb::x::ATOM_WINDOW,
        atoms.net_client_list,
        &clients.into_iter().copied().collect::<Vec<_>>(),
    )
}

/// list all the clients currently managed by the window manager
/// by stacking order, since we dont stack windows, this is the same
/// as the other list
pub fn ewmh_set_client_list_stacking<'a, I>(
    conn: &Arc<xcb::Connection>,
    root: xcb::x::Window,
    clients: I,
    atoms: &Atoms,
) -> anyhow::Result<(), xcb::ProtocolError>
where
    I: IntoIterator<Item = &'a xcb::x::Window>,
    &'a xcb::x::Window: Copy,
{
    xcb_change_prop!(
        conn,
        root,
        xcb::x::PropMode::Replace,
        xcb::x::ATOM_WINDOW,
        atoms.net_client_list_stacking,
        &clients.into_iter().copied().collect::<Vec<_>>(),
    )
}

/// sets every atom required for the window manager to starts operating
pub fn ewmh_set_wm_hints(
    conn: &Arc<xcb::Connection>,
    root: xcb::x::Window,
    atoms: &Atoms,
) -> anyhow::Result<(), xcb::ProtocolError> {
    // we need to create a dummy window and set `_NET_WM_NAME` and
    // `_NET_SUPPORTING_WM_CHECK` atoms to it, this is required on
    // EWMH section "Root Window Properties", to indicate a
    // EWMH-compliant window manager is present.
    //
    // we create the window at (-1, -1) so it stays offscreen
    let ewmh_win_id = xcb_create_win!(conn, root, Position::new(-1, -1, 1, 1), &[]);
    xcb_change_prop!(
        conn,
        ewmh_win_id,
        xcb::x::PropMode::Replace,
        xcb::x::ATOM_STRING,
        atoms.net_supporting_wm_check,
        &[ewmh_win_id],
    )?;
    xcb_change_prop!(
        conn,
        ewmh_win_id,
        xcb::x::PropMode::Replace,
        xcb::x::ATOM_STRING,
        atoms.net_wm_name,
        "lucky".as_bytes(),
    )?;
    // we are also setting it on the root window, im not sure if this
    // is required, but oh well.
    xcb_change_prop!(
        conn,
        root,
        xcb::x::PropMode::Replace,
        xcb::x::ATOM_WINDOW,
        atoms.net_supporting_wm_check,
        &[ewmh_win_id],
    )?;
    xcb_change_prop!(
        conn,
        root,
        xcb::x::PropMode::Replace,
        xcb::x::ATOM_STRING,
        atoms.net_wm_name,
        "lucky".as_bytes(),
    )?;
    xcb_change_prop!(
        conn,
        root,
        xcb::x::PropMode::Replace,
        xcb::x::ATOM_ATOM,
        atoms.net_supported,
        &atoms.list(),
    )?;

    xcb_map_win!(conn, ewmh_win_id);

    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EwmhShowingDesktop {
    Show,
    Hide,
}

impl From<EwmhShowingDesktop> for u32 {
    fn from(value: EwmhShowingDesktop) -> u32 {
        match value {
            EwmhShowingDesktop::Show => 1,
            EwmhShowingDesktop::Hide => 0,
        }
    }
}

/// set desktop is a mode where the window manager is solely displaying
/// the background while hiding every other window
pub fn ewmh_set_showing_desktop(
    conn: &Arc<xcb::Connection>,
    root: xcb::x::Window,
    atoms: &Atoms,
    action: EwmhShowingDesktop,
) -> anyhow::Result<(), xcb::ProtocolError> {
    xcb_change_prop!(
        conn,
        root,
        xcb::x::PropMode::Replace,
        xcb::x::ATOM_CARDINAL,
        atoms.net_showing_desktop,
        &[u32::from(action)]
    )
}
