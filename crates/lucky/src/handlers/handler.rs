use crate::event::EventContext;

pub trait Handler: std::fmt::Debug {
    fn on_key_press(
        &mut self,
        _context: EventContext<xcb::x::KeyPressEvent>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_map_request(
        &mut self,
        _context: EventContext<xcb::x::MapRequestEvent>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_unmap_notify(
        &mut self,
        _context: EventContext<xcb::x::UnmapNotifyEvent>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_destroy_notify(
        &mut self,
        _context: EventContext<xcb::x::DestroyNotifyEvent>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_enter_notify(
        &mut self,
        _context: EventContext<xcb::x::EnterNotifyEvent>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_property_notify(
        &mut self,
        _context: EventContext<xcb::x::PropertyNotifyEvent>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
