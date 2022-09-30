pub type Color = u32;

const BLACK: Color = 0;
const WHITE: Color = from_u8_rgb(255, 255, 255);

pub const BG: Color = BLACK;
pub const FG: Color = WHITE;

pub const fn from_u8_rgb(r: u8, g: u8, b: u8) -> Color {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

pub const fn to_u8_rgb(color: Color) -> (u8, u8, u8) {
    let [_a, r, g, b] = color.to_be_bytes();
    (r, g, b)
}

pub fn darken(color: Color, opacity: f32) -> Color {
    let opacity = opacity.clamp(0.0, u8::MAX.into());
    let (r, g, b) = to_u8_rgb(color);
    from_u8_rgb(
        (f32::from(r) * opacity).round() as u8,
        (f32::from(g) * opacity).round() as u8,
        (f32::from(b) * opacity).round() as u8,
    )
}

/// Whether the pixel should be considered transparent.
pub fn pixel_is_transparent(px: u32) -> bool {
    #[allow(clippy::assertions_on_constants)] // no duh it's optimized out, clippy
    const _: () = assert!(BG == 0, "this algorithm relies on BG being black");

    const fn sum_rgb(color: Color) -> u16 {
        let (r, g, b) = to_u8_rgb(color);
        r as u16 + g as u16 + b as u16
    }

    const FG_RGB_SUM: u16 = sum_rgb(FG);

    // Out of u8::MAX, times three because RGB are three
    const OVERWRITE_THRESHOLD: u16 = 100 * 3;

    FG_RGB_SUM - sum_rgb(px) > OVERWRITE_THRESHOLD
}
