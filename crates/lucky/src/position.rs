#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl From<&xcb::randr::MonitorInfo> for Position {
    fn from(value: &xcb::randr::MonitorInfo) -> Self {
        Position {
            x: value.x().into(),
            y: value.y().into(),
            width: value.width().into(),
            height: value.height().into(),
        }
    }
}

impl Position {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Position {
            x,
            y,
            width,
            height,
        }
    }

    /// fetches the right of the screen by adding its starting x position
    /// to the width, resulting in its right x position
    pub fn right(&self) -> i32 {
        self.x + self.width as i32
    }

    /// fetches the left of the screen, this exists mainly to have a better
    /// naming than x over the codebase
    pub fn left(&self) -> i32 {
        self.x
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.height as i32
    }

    pub fn top(&self) -> i32 {
        self.y
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "x: {}, y: {}, width: {}, height: {}",
            self.x, self.y, self.width, self.height
        ))
    }
}
