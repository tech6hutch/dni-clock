/// The color of a single pixel, stored as 0xAARRGGBB (but alpha isn't used).
pub type Color = u32;

const BLACK: Color = 0;
const WHITE: Color = from_u8_rgb(255, 255, 255);

/// Background color
pub const BG: Color = BLACK;
/// Foreground color
pub const FG: Color = WHITE;

/// Create a color from red, green, and blue parts. Alpha is set to 0.
pub const fn from_u8_rgb(r: u8, g: u8, b: u8) -> Color {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

/// Unpack a color into red, green, and blue parts. Alpha is ignored.
pub const fn to_u8_rgb(color: Color) -> (u8, u8, u8) {
    let [_a, r, g, b] = color.to_be_bytes();
    (r, g, b)
}

/// Darkens a color to a percent (0.0 to 1.0) of its brightness.
///
///  - 1.0 returns the color unchanged
///  - 0.0 returns black
pub fn darken(color: Color, percent: f32) -> Color {
    let percent = percent.clamp(0.0, u8::MAX.into());
    let (r, g, b) = to_u8_rgb(color);
    from_u8_rgb(
        (f32::from(r) * percent).round() as u8,
        (f32::from(g) * percent).round() as u8,
        (f32::from(b) * percent).round() as u8,
    )
}
