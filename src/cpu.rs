pub const MEMORY_SIZE: usize = 4096;
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const NUM_REGISTERS: usize = 16;
pub const STACK_SIZE: usize = 16;
pub const FONTSET_START_ADDRESS: usize = 0x50;
pub const PROGRAM_START_ADDRESS: usize = 0x200;

pub const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // a
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // b
    0xF0, 0x80, 0x80, 0x80, 0xF0, // c
    0xE0, 0x90, 0x90, 0x90, 0xE0, // d
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // e
    0xF0, 0x80, 0xF0, 0x80, 0x80, // f
];

pub struct Cpu {
    pub memory: [u8; MEMORY_SIZE],
    pub v: [u8; NUM_REGISTERS],
    pub i: u16,
    pub pc: u16,
    pub stack: [u16; STACK_SIZE],
    pub sp: usize,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    pub keypad: [bool; 16],
    rng_state: u32,
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Cpu {
            memory: [0; MEMORY_SIZE],
            v: [0; NUM_REGISTERS],
            i: 0,
            pc: PROGRAM_START_ADDRESS as u16,
            stack: [0; STACK_SIZE],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            keypad: [false; 16],
            rng_state: 0x12345678,
        };

        // copy font set to memory
        cpu.memory[FONTSET_START_ADDRESS..FONTSET_START_ADDRESS + 80]
            .copy_from_slice(&FONTSET);

        cpu
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        let start = PROGRAM_START_ADDRESS;
        let end = start + rom.len();
        assert!(end <= MEMORY_SIZE, "ROM is too large for CHIP-8 memory");
        self.memory[start..end].copy_from_slice(rom);
        self.pc = PROGRAM_START_ADDRESS as u16;
    }

    fn random_byte(&mut self) -> u8 {
        self.rng_state = self.rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.rng_state >> 16) & 0xFF) as u8
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn step(&mut self) -> u16 {
        let pc = self.pc as usize;
        let opcode = ((self.memory[pc] as u16) << 8) | (self.memory[pc + 1] as u16);
        self.pc += 2;

        self.execute(opcode);
        opcode
    }

    pub fn execute(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as u8;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        match (opcode & 0xF000) >> 12 {
            0x0 => match opcode {
                0x00E0 => {
                    // cls
                    self.display.fill(false);
                }
                0x00EE => {
                    // ret
                    assert!(self.sp > 0, "Stack underflow");
                    self.sp -= 1;
                    self.pc = self.stack[self.sp];
                }
                _ => {} // sys call ignored
            },
            0x1 => {
                // jp addr
                self.pc = nnn;
            }
            0x2 => {
                // call addr
                assert!(self.sp < STACK_SIZE, "Stack overflow");
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            }
            0x3 => {
                // se vx, byte
                if self.v[x] == nn {
                    self.pc += 2;
                }
            }
            0x4 => {
                // sne vx, byte
                if self.v[x] != nn {
                    self.pc += 2;
                }
            }
            0x5 => {
                // se vx, vy
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            0x6 => {
                // ld vx, byte
                self.v[x] = nn;
            }
            0x7 => {
                // add vx, byte
                self.v[x] = self.v[x].wrapping_add(nn);
            }
            0x8 => match opcode & 0x000F {
                0x0 => self.v[x] = self.v[y],
                0x1 => self.v[x] |= self.v[y],
                0x2 => self.v[x] &= self.v[y],
                0x3 => self.v[x] ^= self.v[y],
                0x4 => {
                    let (sum, carry) = self.v[x].overflowing_add(self.v[y]);
                    self.v[x] = sum;
                    self.v[0xF] = if carry { 1 } else { 0 };
                }
                0x5 => {
                    let (sub, borrow) = self.v[x].overflowing_sub(self.v[y]);
                    self.v[x] = sub;
                    self.v[0xF] = if !borrow { 1 } else { 0 };
                }
                0x6 => {
                    let lsb = self.v[x] & 0x1;
                    self.v[x] >>= 1;
                    self.v[0xF] = lsb;
                }
                0x7 => {
                    let (sub, borrow) = self.v[y].overflowing_sub(self.v[x]);
                    self.v[x] = sub;
                    self.v[0xF] = if !borrow { 1 } else { 0 };
                }
                0xE => {
                    let msb = (self.v[x] >> 7) & 0x1;
                    self.v[x] <<= 1;
                    self.v[0xF] = msb;
                }
                _ => {}
            },
            0x9 => {
                // sne vx, vy
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            0xA => {
                // ld i, addr
                self.i = nnn;
            }
            0xB => {
                // jp v0, addr
                self.pc = nnn + self.v[0] as u16;
            }
            0xC => {
                // rnd vx, byte
                let rnd = self.random_byte();
                self.v[x] = rnd & nn;
            }
            0xD => {
                // drw vx, vy, nibble
                let x_coord = (self.v[x] as usize) % DISPLAY_WIDTH;
                let y_coord = (self.v[y] as usize) % DISPLAY_HEIGHT;
                self.v[0xF] = 0;

                for row in 0..n as usize {
                    let py = (y_coord + row) % DISPLAY_HEIGHT;
                    let sprite_byte = self.memory[(self.i as usize) + row];

                    for col in 0..8 {
                        let px = (x_coord + col) % DISPLAY_WIDTH;
                        let sprite_pixel = (sprite_byte & (0x80 >> col)) != 0;
                        let index = py * DISPLAY_WIDTH + px;

                        if sprite_pixel {
                            if self.display[index] {
                                self.v[0xF] = 1; // collision flag
                            }
                            self.display[index] ^= true;
                        }
                    }
                }
            }
            0xE => match opcode & 0x00FF {
                0x9E => {
                    // skp vx
                    let key = self.v[x] as usize;
                    if key < 16 && self.keypad[key] {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    // sknp vx
                    let key = self.v[x] as usize;
                    if key >= 16 || !self.keypad[key] {
                        self.pc += 2;
                    }
                }
                _ => {}
            },
            0xF => match opcode & 0x00FF {
                0x07 => self.v[x] = self.delay_timer,
                0x0A => {
                    // wait for key press
                    let mut pressed = None;
                    for (k, &is_down) in self.keypad.iter().enumerate() {
                        if is_down {
                            pressed = Some(k as u8);
                            break;
                        }
                    }
                    if let Some(key) = pressed {
                        self.v[x] = key;
                    } else {
                        self.pc -= 2; // repeat until key pressed
                    }
                }
                0x15 => self.delay_timer = self.v[x],
                0x18 => self.sound_timer = self.v[x],
                0x1E => self.i = self.i.wrapping_add(self.v[x] as u16),
                0x29 => {
                    // font character offset
                    self.i = (FONTSET_START_ADDRESS + (self.v[x] as usize * 5)) as u16;
                }
                0x33 => {
                    // store bcd representation
                    let val = self.v[x];
                    let idx = self.i as usize;
                    self.memory[idx] = val / 100;
                    self.memory[idx + 1] = (val / 10) % 10;
                    self.memory[idx + 2] = val % 10;
                }
                0x55 => {
                    // dump registers to memory
                    let idx = self.i as usize;
                    for reg in 0..=x {
                        self.memory[idx + reg] = self.v[reg];
                    }
                }
                0x65 => {
                    // load registers from memory
                    let idx = self.i as usize;
                    for reg in 0..=x {
                        self.v[reg] = self.memory[idx + reg];
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ld_and_add() {
        let mut cpu = Cpu::new();
        // ld v0, 5
        // add v0, 10
        cpu.execute(0x6005);
        assert_eq!(cpu.v[0], 5);
        cpu.execute(0x700A);
        assert_eq!(cpu.v[0], 15);
    }

    #[test]
    fn test_stack_call_and_return() {
        let mut cpu = Cpu::new();
        cpu.execute(0x2300); // call 0x300
        assert_eq!(cpu.pc, 0x300);
        assert_eq!(cpu.sp, 1);
        cpu.execute(0x00EE); // ret
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.sp, 0);
    }

    #[test]
    fn test_draw_sprite() {
        let mut cpu = Cpu::new();
        // coordinates (0, 0)
        cpu.v[0] = 0;
        cpu.v[1] = 0;
        // font sprite offset
        cpu.i = FONTSET_START_ADDRESS as u16;
        // draw 5-byte sprite
        cpu.execute(0xD015);
        // verify pixels rendered
        assert!(cpu.display.iter().any(|&p| p));
        assert_eq!(cpu.v[0xF], 0); // no collision on clear screen

        // collision detected on overlap
        cpu.execute(0xD015);
        assert_eq!(cpu.v[0xF], 1);
    }

    #[test]
    fn test_bcd_conversion() {
        let mut cpu = Cpu::new();
        cpu.v[0] = 254;
        cpu.i = 0x400;
        cpu.execute(0xF033);
        assert_eq!(cpu.memory[0x400], 2);
        assert_eq!(cpu.memory[0x401], 5);
        assert_eq!(cpu.memory[0x402], 4);
    }
}
