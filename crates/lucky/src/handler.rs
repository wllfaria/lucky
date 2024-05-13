use crate::event::EventContext;

pub trait Handler {
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
}
