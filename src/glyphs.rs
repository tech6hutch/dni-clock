//! Glyph rendering and types.
//!
//! The fonts used by the crate are included in the binary.

use std::array;

use ab_glyph::{Font, FontRef};

use crate::{buf2d::Vec2d, colors::{self, Color}};

/// A glyph rendered to pixels.
pub type GlyphBuffer = Vec2d<Color>;

/// A simple wrapper over a buffer. It lets you write glyphs in a row.
#[derive(Default)]
pub struct TextBuffer {
    /// The buffer being written into.
    pub buf: GlyphBuffer,
    /// Where the next glyph should start, horizontally.
    pub x: usize,
    /// The top of the line of text (for the next glyph). The glyph is centered
    /// vertically if it's shorter than the line height, however.
    pub y: usize,
    /// The line height.
    pub height: usize,
}

impl TextBuffer {
    /// Create a new empty one. It won't work in this state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Writes a glyph and advances by its width.
    pub fn write_glyph(&mut self, glyph: &GlyphBuffer) {
        self._write_glyph::<false>(glyph)
    }

    /// Writes a glyph and advances by its width, only overwriting pixels that are
    /// somewhat transparent (i.e., so you can compose it with the previous glyph).
    pub fn write_glyph_composing(&mut self, glyph: &GlyphBuffer) {
        self._write_glyph::<true>(glyph)
    }

    fn _write_glyph<const COMPOSE: bool>(&mut self, glyph: &GlyphBuffer) {
        let height_diff = self.height.checked_sub(glyph.height())
            .expect("glyph was taller than the line");
        let centered_y = self.y + height_diff / 2;

        if COMPOSE {
            self.buf.copy_to_from_if(
                self.x,
                centered_y,
                glyph,
                Self::pixel_is_somewhat_transparent,
            );
        } else {
            self.buf.copy_to_from(
                self.x,
                centered_y,
                glyph,
            );
        }

        self.x += glyph.width();
    }

    /// Whether the pixel should be considered transparent against a background of
    /// `colors::BG` (i.e., should be overwritten, when composing glyphs).
    fn pixel_is_somewhat_transparent(px: Color) -> bool {
        #[allow(clippy::assertions_on_constants)] // no duh it's optimized out, clippy
        const _: () = assert!(colors::BG == 0, "this algorithm relies on BG being black");

        /// Adds the red, green, and blue components
        const fn sum_rgb(color: Color) -> u16 {
            let (r, g, b) = colors::to_u8_rgb(color);
            r as u16 + g as u16 + b as u16
        }

        const FG_RGB_SUM: u16 = sum_rgb(colors::FG);

        const THRESHOLD: u16 = 100; // out of u8::MAX

        // The idea here is to check if the brightness of FG minus the brightness of px
        // is less than a certain threshold. The concept of "brightness" here is a
        // simple one (if not very accurate to human perception): just average together
        // the color's RGB values. Averaging requires dividing by the number of elements
        // (3) however, which we avoid by multiplying the threshold instead (by 3).
        FG_RGB_SUM - sum_rgb(px) > THRESHOLD * 3
    }
}

/// Handles glyph rendering and caches them.
// todo: The commented-out fields and rescale method are for when I implement window resizing.
pub struct Glyphs {
    // dni_font: FontRef<'static>,
    // ascii_font: FontRef<'static>,
    cache: Cache,
}

impl Glyphs {
    /// Initializes its cache with the given scale.
    pub fn with_starting_scale(scale: f32) -> Self {
        let dni_font = get_dni_font();
        let ascii_font = get_ascii_font();
        Self {
            cache: Cache::generate(scale, &dni_font, &ascii_font),
            // dni_font,
            // ascii_font,
        }
    }

    // /// Change the text scale of the glyphs
    // pub fn rescale(&mut self, scale: f32) {
    //     if self.cache.scale != scale {
    //         self.cache = Cache::generate(scale, &self.dni_font, &self.ascii_font);
    //     }
    // }

    /// Get a single-digit numeral (0-24)
    pub fn get_dni_number_one_digit(&self, n: u8) -> &GlyphBuffer {
        &self.cache.dni_digits[usize::from(n)]
    }

    /// Get a numeral, padded to two digits (00-59)
    pub fn get_dni_number_two_digits(&mut self, n: u8) -> &GlyphBuffer {
        let cache = &mut self.cache;
        cache
            .dni_numerals[usize::from(n)]
            .get_or_insert_with(|| Cache::compose_numeral(cache.scale, &cache.dni_digits, n))
    }

    /// Get a colon (`':'`) glyph
    pub fn get_colon(&self) -> &GlyphBuffer {
        &self.cache.colon
    }
}

struct Cache {
    /// The amount the glyph is scaled by
    scale: f32,
    /// The digits 0-24
    dni_digits: [GlyphBuffer; 25],
    /// Numerals from 00-59, padded to two digits
    dni_numerals: [Option<GlyphBuffer>; 60],
    /// ASCII colon `':'`
    colon: GlyphBuffer,
}

impl Cache {
    /// Generates a cache with the given scale in the given fonts.
    fn generate(scale: f32, dni_font: &impl Font, ascii_font: &impl Font) -> Self {
        Self {
            scale,
            dni_digits: array::from_fn(|n|
                render_scaled_glyph(dni_font, n_to_dni(n as u8).into(), scale)),
            dni_numerals: array::from_fn(|_| None),
            colon: render_scaled_glyph(ascii_font, ':', scale),
        }
    }

    /// Composes a two-digit D'ni numeral.
    ///
    /// This is a static method to avoid borrowing errors.
    fn compose_numeral(scale: f32, dni_digits: &[GlyphBuffer; 25], n: u8) -> GlyphBuffer {
        let digit1 = n % 25;
        let digit2 = (n - digit1) / 25;
        debug_assert_eq!(digit2 * 25 + digit1, n);
        debug_assert!(digit2 < 25);

        // Single digits are always cached
        let digit1_buf = &dni_digits[usize::from(digit1)];
        let digit2_buf = &dni_digits[usize::from(digit2)];

        let overlap = digit_overlap(scale);

        let width = digit1_buf.width() + digit2_buf.width() - overlap;
        let height = digit1_buf.height();
        debug_assert_eq!(height, digit2_buf.height());
        let mut n_buf = TextBuffer {
            buf: Vec2d::new(colors::BG, width, height),
            x: 0,
            y: 0,
            height,
        };
        n_buf.write_glyph_composing(digit2_buf);
        n_buf.x -= overlap;
        n_buf.write_glyph_composing(digit1_buf);
        n_buf.buf
    }
}

/// Renders `c` at `scale` in the `font`, to a an array of pixels.
///
/// Panics if `Font::outline_glyph` does, however that can happen.
fn render_scaled_glyph(font: &impl Font, c: char, scale: f32) -> GlyphBuffer {
    let glyph = font.glyph_id(c).with_scale(scale);
    let glyph = font.outline_glyph(glyph).unwrap();
    let width = glyph.px_bounds().width() as usize;
    let height = glyph.px_bounds().height() as usize;
    let mut buf = Vec2d::new(colors::BG, width, height);
    glyph.draw(|x, y, c| {
        buf[(x, y)] = colors::darken(colors::FG, c);
    });
    buf
}

/// Converts a number to an ASCII character corresponding to a single D'ni digit.
///
/// Panics if the number is out of range (>25).
fn n_to_dni(n: u8) -> u8 {
    const DIGITS: &[u8] = b"\
    0123456789\
    )!@#$%^&*(\
    []\\{}|\
    ";
    const _: () = assert!(DIGITS[0] == b'0');
    const _: () = assert!(DIGITS[25] == b'|');
    const _: () = assert!(DIGITS.len() == 26);
    DIGITS[usize::from(n)]
}

/// The "walls" of consecutive digits overlap. This is the number of pixels to overlap.
fn digit_overlap(scale: f32) -> usize {
    (scale * 0.25).round() as usize
}

/// Get the D'ni font from the binary.
fn get_dni_font() -> FontRef<'static> {
    FontRef::try_from_slice(include_bytes!("../fonts/Dni.ttf")).unwrap()
}

/// Get the regular font from the binary.
fn get_ascii_font() -> FontRef<'static> {
    FontRef::try_from_slice(include_bytes!("../fonts/Source_Sans_Pro/SourceSansPro-Regular.ttf")).unwrap()
}
