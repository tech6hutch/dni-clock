mod buf2d;
mod colors;
mod glyphs;
mod util;

use chrono::{DateTime, DurationRound, Local, Timelike};
use minifb::{Window, WindowOptions};

use crate::{buf2d::Vec2d, colors::BG, glyphs::{Glyphs, TextBuffer}};

const SHOW_SECONDS: bool = true;
const WINDOW_WIDTH: usize = if SHOW_SECONDS { 300 } else { 200 };
const WINDOW_HEIGHT: usize = 70;
const MARGIN: usize = 10;
const LINE_HEIGHT: usize = WINDOW_HEIGHT - MARGIN - MARGIN;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let mut glyphs = Glyphs::with_starting_scale(LINE_HEIGHT as f32);

    let mut buffer = TextBuffer::new();

    // Start with yesterday to make sure the window gets updated right away
    let mut time = Local::today().pred().and_hms(0, 0, 0);

    let mut window = open_window();
    while window.is_open() {
        let new_time =
            if SHOW_SECONDS { local_time_to_the_second() }
            else { local_time_to_the_minute() };
        if new_time != time {
            buffer = update_time(new_time, &mut glyphs);
            time = new_time;
        }
        window.update_with_buffer(buffer.buf.as_1d(), buffer.buf.width(), buffer.buf.height()).unwrap();
    }
}

fn open_window() -> Window {
    Window::new(
        "D'ni Clock",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions::default(),
    ).unwrap()
}

fn update_time(time: DateTime<Local>, glyphs: &mut Glyphs) -> TextBuffer {
    let mut buffer = TextBuffer {
        buf: Vec2d::new(BG, WINDOW_WIDTH, WINDOW_HEIGHT),
        x: MARGIN,
        y: MARGIN,
        height: LINE_HEIGHT,
    };

    buffer.write_glyph(glyphs.get_dni_number_one_digit(time.hour().try_into().unwrap()));

    buffer.write_glyph(glyphs.get_colon());

    buffer.write_glyph(glyphs.get_dni_number_two_digits(time.minute().try_into().unwrap()));

    if SHOW_SECONDS {
        buffer.write_glyph(glyphs.get_colon());

        buffer.write_glyph(glyphs.get_dni_number_two_digits(time.second().try_into().unwrap()));
    }

    buffer
}

fn local_time_to_the_minute() -> DateTime<Local> {
    Local::now()
        .duration_trunc(chrono::Duration::minutes(1))
        .unwrap()
}

fn local_time_to_the_second() -> DateTime<Local> {
    Local::now()
        .duration_trunc(chrono::Duration::seconds(1))
        .unwrap()
}
