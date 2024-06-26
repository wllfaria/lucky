use std::sync::Arc;

pub struct Atoms {
    pub wm_protocols: xcb::x::Atom,
    pub wm_delete_window: xcb::x::Atom,
    pub net_wm_name: xcb::x::Atom,
    pub net_wm_state: xcb::x::Atom,
    pub net_wm_window_type: xcb::x::Atom,
    pub net_current_desktop: xcb::x::Atom,
    pub net_number_of_desktops: xcb::x::Atom,
    pub net_wm_desktop: xcb::x::Atom,
    pub net_supported: xcb::x::Atom,
    pub net_wm_strut_partial: xcb::x::Atom,
}

impl Atoms {
    pub fn new(conn: &Arc<xcb::Connection>, root: xcb::x::Window) -> Self {
        let wm_protocols = Self::get_intern_atom(conn, b"WM_PROTOCOLS");
        let wm_delete_window = Self::get_intern_atom(conn, b"WM_DELETE_WINDOW");
        let net_wm_name = Self::get_intern_atom(conn, b"_NET_WM_NAME");
        let net_wm_state = Self::get_intern_atom(conn, b"_NET_WM_STATE");
        let net_wm_window_type = Self::get_intern_atom(conn, b"_NET_WM_WINDOW_TYPE");
        let net_current_desktop = Self::get_intern_atom(conn, b"_NET_CURRENT_DESKTOP");
        let net_number_of_desktops = Self::get_intern_atom(conn, b"_NET_NUMBER_OF_DESKTOPS");
        let net_wm_desktop = Self::get_intern_atom(conn, b"_NET_WM_DESKTOP");
        let net_supported = Self::get_intern_atom(conn, b"_NET_SUPPORTED");
        let net_wm_strut_partial = Self::get_intern_atom(conn, b"_NET_WM_STRUT_PARTIAL");
        tracing::debug!(" getting partial strut {net_wm_strut_partial:?}");

        let atoms = Atoms {
            wm_protocols,
            wm_delete_window,
            net_wm_name,
            net_wm_state,
            net_wm_window_type,
            net_current_desktop,
            net_number_of_desktops,
            net_wm_desktop,
            net_supported,
            net_wm_strut_partial,
        };

        atoms.set_net_supported(conn, root);

        atoms
    }

    /// Utility function to get an internal atom from x server
    /// we use this to query EWMH atoms and commonly used atoms at startup time to be reused later
    fn get_intern_atom(conn: &Arc<xcb::Connection>, name: &[u8]) -> xcb::x::Atom {
        let cookie = conn.send_request(&xcb::x::InternAtom {
            only_if_exists: false,
            name,
        });
        let reply = conn
            .wait_for_reply(cookie)
            .expect("we failed to get an internal atom");
        reply.atom()
    }

    fn set_net_supported(&self, conn: &Arc<xcb::Connection>, root: xcb::x::Window) {
        let atoms = vec![
            self.net_wm_name,
            self.net_wm_state,
            self.net_wm_window_type,
            self.net_current_desktop,
            self.net_number_of_desktops,
            self.net_wm_desktop,
        ];

        conn.send_and_check_request(&xcb::x::ChangeProperty {
            window: root,
            mode: xcb::x::PropMode::Replace,
            r#type: xcb::x::ATOM_ATOM,
            property: self.net_supported,
            data: &atoms,
        })
        .expect("failed to set supported atoms");
    }
}
