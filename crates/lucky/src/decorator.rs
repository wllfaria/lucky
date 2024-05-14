use config::Config;
use std::{rc::Rc, sync::Arc};

pub struct Decorator {
    config: Rc<Config>,
    conn: Arc<xcb::Connection>,
}

impl Decorator {
    pub fn new(conn: Arc<xcb::Connection>, config: Rc<Config>) -> Self {
        Decorator { conn, config }
    }
}
