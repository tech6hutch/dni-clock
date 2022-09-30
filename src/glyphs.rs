use std::array;

use ab_glyph::{Font, FontRef};

use crate::{buf2d::Vec2d, colors::{BG, FG, self, Color}};

/// A glyph rendered to pixels.
pub type GlyphBuffer = Vec2d<Color>;

fn get_dni_font() -> FontRef<'static> {
    FontRef::try_from_slice(include_bytes!("../fonts/Dni.ttf")).unwrap()
}

fn get_ascii_font() -> FontRef<'static> {
    FontRef::try_from_slice(include_bytes!("../fonts/Source_Sans_Pro/SourceSansPro-Regular.ttf")).unwrap()
}

/// A simple wrapper over a buffer. It lets you write glyphs in a row.
#[derive(Default)]
pub struct TextBuffer {
    pub buf: GlyphBuffer,
    pub x: usize,
    pub y: usize,
    pub height: usize,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Writes a glyph and advances by its width.
    pub fn write_glyph(&mut self, glyph: &GlyphBuffer) {
        let height_diff = self.height.checked_sub(glyph.height())
            .expect("glyph was taller than the line");
        let centered_y = self.y + height_diff / 2;
        self.buf.copy_to_from_if(
            self.x,
            centered_y,
            glyph,
            colors::pixel_is_transparent,
        );
        self.x += glyph.width();
    }
}

pub struct Glyphs {
    dni_font: FontRef<'static>,
    ascii_font: FontRef<'static>,
    cache: Cache,
}

impl Glyphs {
    pub fn new() -> Self {
        Self {
            dni_font: get_dni_font(),
            ascii_font: get_ascii_font(),
            cache: Cache::new(),
        }
    }

    fn dni_font(&self) -> &impl Font {
        &self.dni_font
    }

    fn ascii_font(&self) -> &impl Font {
        &self.ascii_font
    }

    pub fn rescale(&mut self, scale: f32) {
        if self.cache.scale != scale {
            self.cache = Cache::with_scale(scale, self.dni_font(), self.ascii_font());
        }
    }

    /// Get a single-digit numeral (0-24)
    pub fn get_dni_numeral_one_digit(&self, n: u8) -> &GlyphBuffer {
        &self.cache.dni_digits[usize::from(n)]
    }

    /// Get a numeral, padded to two digits (00-59)
    pub fn get_dni_numeral_two_digits(&mut self, n: u8) -> &GlyphBuffer {
        let cache = &mut self.cache;
        cache
            .dni_numerals[usize::from(n)]
            .get_or_insert_with(|| Cache::compose_numeral(cache.scale, &cache.dni_digits, n))
    }

    pub fn get_colon(&self) -> &GlyphBuffer {
        &self.cache.colon
    }
}

struct Cache {
    scale: f32,
    /// The digits 0-24
    dni_digits: [GlyphBuffer; 25],
    /// Numerals from 00-59, padded to two digits
    dni_numerals: [Option<GlyphBuffer>; 60],
    colon: GlyphBuffer,
}

impl Cache {
    fn new() -> Self {
        Self::default()
    }

    /// Generates a cache with the given scale in the given fonts.
    fn with_scale(scale: f32, dni_font: &impl Font, ascii_font: &impl Font) -> Self {
        Self {
            scale,
            dni_digits: array::from_fn(|n|
                render_scaled_glyph(dni_font, n_to_dni(n as u8).into(), scale)),
            colon: render_scaled_glyph(ascii_font, ':', scale),
            ..Default::default()
        }
    }

    /// Composes a two-digit D'ni numeral.
    ///
    /// The fields are separated to prevent borrowing errors.
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
        n_buf.write_glyph(digit2_buf);
        n_buf.x -= overlap;
        n_buf.write_glyph(digit1_buf);
        n_buf.buf
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            scale: 0.0,
            dni_digits: Default::default(),
            dni_numerals: array::from_fn(|_| None),
            colon: Default::default(),
        }
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
    let mut buf = Vec2d::new(BG, width, height);
    glyph.draw(|x, y, c| {
        buf[(x, y)] = colors::darken(FG, c);
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
