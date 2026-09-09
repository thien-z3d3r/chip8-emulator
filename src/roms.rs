// test rom: render sample font glyphs
pub fn sample_test_rom() -> Vec<u8> {
    vec![
        // cls
        0x00, 0xE0,
        // set v0, v1
        0x60, 0x04,
        0x61, 0x04,
        // set i to font 0
        0xA0, 0x50,
        // draw font sprite
        0xD0, 0x15,
        // advance x
        0x70, 0x08,
        // set i to font 1
        0xA0, 0x55,
        // draw font sprite
        0xD0, 0x15,
        // advance x
        0x70, 0x08,
        // set i to font 2
        0xA0, 0x5A,
        // draw font sprite
        0xD0, 0x15,
        // advance x
        0x70, 0x08,
        // set i to font 3
        0xA0, 0x5F,
        // draw font sprite
        0xD0, 0x15,
        // loop at 0x21a
        0x12, 0x1A,
    ]
}
