use crate::xcb_utils::*;

use std::sync::Arc;

pub struct Atoms {
    pub wm_protocols: xcb::x::Atom,
    pub wm_delete_window: xcb::x::Atom,
    pub net_wm_name: xcb::x::Atom,
    pub net_wm_state: xcb::x::Atom,
    pub net_wm_state_focused: xcb::x::Atom,
    pub net_wm_window_type: xcb::x::Atom,
    pub net_current_desktop: xcb::x::Atom,
    pub net_number_of_desktops: xcb::x::Atom,
    pub net_wm_desktop: xcb::x::Atom,
    pub net_supported: xcb::x::Atom,
    pub net_wm_strut_partial: xcb::x::Atom,
    pub net_desktop_viewport: xcb::x::Atom,
    pub net_desktop_names: xcb::x::Atom,
    pub net_active_window: xcb::x::Atom,
    pub net_supporting_wm_check: xcb::x::Atom,
    pub net_client_list: xcb::x::Atom,
    pub net_client_list_stacking: xcb::x::Atom,
    pub net_showing_desktop: xcb::x::Atom,
}

impl Atoms {
    pub fn new(conn: &Arc<xcb::Connection>) -> Self {
        let wm_protocols = Self::get_intern_atom(conn, b"WM_PROTOCOLS");
        let wm_delete_window = Self::get_intern_atom(conn, b"WM_DELETE_WINDOW");
        let net_wm_name = Self::get_intern_atom(conn, b"_NET_WM_NAME");

        let net_wm_state = Self::get_intern_atom(conn, b"_NET_WM_STATE");
        let net_wm_state_focused = Self::get_intern_atom(conn, b"_NET_WM_STATE_FOCUSED");

        let net_supporting_wm_check = Self::get_intern_atom(conn, b"_NET_SUPPORTING_WM_CHECK");

        let net_wm_window_type = Self::get_intern_atom(conn, b"_NET_WM_WINDOW_TYPE");
        let net_current_desktop = Self::get_intern_atom(conn, b"_NET_CURRENT_DESKTOP");
        let net_number_of_desktops = Self::get_intern_atom(conn, b"_NET_NUMBER_OF_DESKTOPS");
        let net_desktop_viewport = Self::get_intern_atom(conn, b"_NET_DESKTOP_VIEWPORT");
        let net_wm_desktop = Self::get_intern_atom(conn, b"_NET_WM_DESKTOP");
        let net_supported = Self::get_intern_atom(conn, b"_NET_SUPPORTED");
        let net_wm_strut_partial = Self::get_intern_atom(conn, b"_NET_WM_STRUT_PARTIAL");
        let net_desktop_names = Self::get_intern_atom(conn, b"_NET_DESKTOP_NAMES");
        let net_active_window = Self::get_intern_atom(conn, b"_NET_ACTIVE_WINDOW");
        let net_client_list = Self::get_intern_atom(conn, b"_NET_CLIENT_LIST");
        let net_showing_desktop = Self::get_intern_atom(conn, b"_NET_SHOWING_DESKTOP");
        let net_client_list_stacking = Self::get_intern_atom(conn, b"_NET_CLIENT_LIST_STACKING");

        Atoms {
            wm_protocols,
            wm_delete_window,
            net_wm_name,
            net_wm_state,
            net_wm_state_focused,
            net_wm_window_type,
            net_client_list,
            net_current_desktop,
            net_number_of_desktops,
            net_wm_desktop,
            net_supported,
            net_wm_strut_partial,
            net_desktop_viewport,
            net_desktop_names,
            net_active_window,
            net_supporting_wm_check,
            net_client_list_stacking,
            net_showing_desktop,
        }
    }

    /// Utility function to get an internal atom from x server
    /// we use this to query EWMH atoms and commonly used atoms at startup time to be reused later
    fn get_intern_atom(conn: &Arc<xcb::Connection>, name: &[u8]) -> xcb::x::Atom {
        let reply = xcb_intern_atom!(conn, name);
        reply.atom()
    }

    pub fn list(&self) -> Vec<xcb::x::Atom> {
        vec![
            self.wm_protocols,
            self.wm_delete_window,
            self.net_wm_name,
            self.net_wm_state,
            self.net_wm_state_focused,
            self.net_wm_window_type,
            self.net_current_desktop,
            self.net_number_of_desktops,
            self.net_wm_desktop,
            self.net_supported,
            self.net_wm_strut_partial,
            self.net_desktop_viewport,
            self.net_desktop_names,
            self.net_active_window,
            self.net_supporting_wm_check,
            self.net_client_list_stacking,
            self.net_client_list,
            self.net_showing_desktop,
        ]
    }
}
