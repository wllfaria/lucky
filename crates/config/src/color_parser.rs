use std::ops::Div;

#[derive(Debug)]
pub struct Color(pub u32);

#[derive(Debug)]
pub enum ColorParserError {
    InvalidFormat(String),
}

impl std::fmt::Display for ColorParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat(msg) => f.write_str(msg),
        }
    }
}

impl TryFrom<String> for Color {
    type Error = ColorParserError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if let Some(hex) = value.strip_prefix('#') {
            if hex.len() == 6 {
                let color = u32::from_str_radix(hex, 16).map_err(|_| {
                    ColorParserError::InvalidFormat(format!("color {value} is not a valid format"))
                })?;
                return Ok(Color(color));
            }
        }

        if let Some(hsl) = value.strip_prefix("hsl(") {
            if let Some(hsl) = hsl.strip_suffix(')') {
                let parts: Vec<&str> = hsl.split(',').collect();
                if parts.len() == 3 {
                    let h = parts[0].trim().parse::<f64>().map_err(|_| {
                        ColorParserError::InvalidFormat(format!(
                            "color {value} is not a valid format"
                        ))
                    })?;
                    let s = parts[1]
                        .trim()
                        .trim_end_matches('%')
                        .parse::<f64>()
                        .map_err(|_| {
                            ColorParserError::InvalidFormat(format!(
                                "color {value} is not a valid format"
                            ))
                        })?
                        .div(100.0);
                    let l = parts[2]
                        .trim()
                        .trim_end_matches('%')
                        .parse::<f64>()
                        .map_err(|_| {
                            ColorParserError::InvalidFormat(format!(
                                "color {value} is not a valid format"
                            ))
                        })?
                        .div(100.0);
                    return Ok(Color(hsl_to_rgb(h, s, l)));
                }
            }
        }

        if let Some(rgb) = value.strip_prefix("rgb(") {
            if let Some(rgb) = rgb.strip_suffix(')') {
                let parts: Vec<&str> = rgb.split(',').collect();
                if parts.len() == 3 {
                    let r = parts[0].trim().parse::<u8>().map_err(|_| {
                        ColorParserError::InvalidFormat(format!(
                            "color {value} is not a valid format"
                        ))
                    })?;
                    let g = parts[1].trim().parse::<u8>().map_err(|_| {
                        ColorParserError::InvalidFormat(format!(
                            "color {value} is not a valid format"
                        ))
                    })?;
                    let b = parts[2].trim().parse::<u8>().map_err(|_| {
                        ColorParserError::InvalidFormat(format!(
                            "color {value} is not a valid format"
                        ))
                    })?;
                    return Ok(Color(rgb_to_u32(r, g, b)));
                }
            }
        }

        Err(ColorParserError::InvalidFormat(format!(
            "color: {value} has invalid format"
        )))
    }
}

/// Neat trick to convert color components to a single integer.
///
/// we cast the u8s to u32 to accomodate the final 24bits color int
/// and shift each color to their bit position in the number.
///
/// r moves 16 bits
/// g moves 8 bits
/// b statys in place
///
/// using bitwise OR just combines the 3 ints as they are guaranteed
/// to not overlap
///
/// Eg:
/// 0xFF0000 | 0x00FF00 | 0x0000FF
/// becomes:
/// 0xFFFFFF
fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// magic formula to convert from HSL to RGB
/// reference: https://gist.github.com/mjackson/5311256
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> u32 {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match h {
        _ if h.ge(&0.0) && h.le(&60.0) => (c, x, 0.0),
        _ if h.ge(&60.0) && h.le(&120.0) => (x, c, 0.0),
        _ if h.ge(&120.0) && h.le(&180.0) => (0.0, c, x),
        _ if h.ge(&180.0) && h.le(&240.0) => (0.0, x, c),
        _ if h.ge(&240.0) && h.le(&300.0) => (x, 0.0, c),
        _ if h.ge(&300.0) && h.le(&360.0) => (c, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };
    rgb_to_u32(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

impl Default for Color {
    fn default() -> Self {
        Self(0x252525)
    }
}
