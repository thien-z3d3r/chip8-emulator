use crate::cpu::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

pub fn render_ascii(display: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]) -> String {
    let mut out = String::new();
    out.push('+');
    for _ in 0..DISPLAY_WIDTH {
        out.push('-');
    }
    out.push_str("+\n");

    for y in 0..DISPLAY_HEIGHT {
        out.push('|');
        for x in 0..DISPLAY_WIDTH {
            let pixel = display[y * DISPLAY_WIDTH + x];
            out.push(if pixel { '#' } else { ' ' });
        }
        out.push_str("|\n");
    }

    out.push('+');
    for _ in 0..DISPLAY_WIDTH {
        out.push('-');
    }
    out.push_str("+\n");
    out
}
