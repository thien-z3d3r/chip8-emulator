# 05 - CHIP-8 Virtual Machine & Emulator

## Introduction

This project is a cycle-accurate CHIP-8 virtual machine, CPU emulator, disassembler, and ASCII terminal renderer written in pure Rust. It is inspired by Austin Morlan's [Building a CHIP-8 Emulator](https://austinmorlan.com/posts/chip8_emulator/) and Bugzmanov's [Writing NES Emulator in Rust](https://bugzmanov.github.io/nes_ebook/) featured in the [Project Based Learning](https://github.com/practical-tutorials/project-based-learning) repository.

CHIP-8 is an interpreted programming language developed in the mid-1970s for early microcomputers (COSMAC VIP, TELMAC 1800). Emulating CHIP-8 is the classic entry point for learning computer architecture, register buses, fetch-decode-execute instruction cycles, and virtual hardware subsystems.

---

## Tutorial: Hardware Architecture

```
                    +-----------------------------+
                    |          CPU Core           |
                    | PC (16-bit)    I (16-bit)   |
                    | V0 - VF Registers (8-bit)   |
                    | SP (16-level Stack)         |
                    | Delay & Sound Timers (60Hz) |
                    +-----------------------------+
                                   |
         +-------------------------+-------------------------+
         |                         |                         |
         v                         v                         v
+------------------+     +--------------------+     +------------------+
|   4KB RAM        |     |  64x32 Display     |     |  16-Key Keypad   |
| 0x000-0x1FF: Sys |     | Monochrome buffer  |     | Hex keys 0x0-0xF |
| 0x050-0x09F: Font|     | XOR sprite blit    |     +------------------+
| 0x200-0xFFF: ROM |     | Collision detector |
+------------------+     +--------------------+
```

### 1. Memory and Register Bus (`src/cpu.rs`)

- **RAM**: 4096 bytes (4KB). Program ROMs are loaded at address `0x200`. The region below `0x200` was historically reserved for the interpreter.
- **Registers**: 16 general-purpose 8-bit registers labeled `V0` through `VF`.
  - Register `VF` doubles as a flag: carry flag for additions, borrow flag for subtractions, and collision detection flag for sprite drawing.
- **Index Register `I`**: 16-bit register pointing to memory locations.
- **Program Counter `PC`**: 16-bit register pointing to the current executing instruction.
- **Stack & Stack Pointer `SP`**: 16-level 16-bit call stack for subroutines (`CALL` and `RET`).
- **Timers**: 8-bit delay and sound timers decremented at 60 Hz.

### 2. Complete Opcode Engine (`src/cpu.rs`)

CHIP-8 has 35 two-byte (16-bit) opcodes stored big-endian. The emulator decodes and executes all of them:

- **Flow Control**: `00EE` (return), `1NNN` (jump), `2NNN` (subroutine call), `BNNN` (jump with offset).
- **Conditional Skips**: `3XNN` (skip if equal), `4XNN` (skip if not equal), `5XY0` (skip if registers equal), `9XY0` (skip if registers not equal).
- **Arithmetic & Logic (`8XY_`)**: Move, bitwise OR, AND, XOR, addition with carry, subtraction with borrow, right-shift, and left-shift.
- **Graphics (`DXYN`)**: Draws an 8-pixel-wide by N-pixel-tall sprite from memory location `I` at screen coordinates `(Vx, Vy)`. Pixels are XORed with the screen buffer. If any set pixel is erased, `VF` is set to `1` (hardware collision detection).
- **BCD Conversion (`FX33`)**: Decodes an 8-bit integer into its base-10 hundreds, tens, and units digits stored at `I`, `I+1`, and `I+2`.
- **Memory Dump & Load (`FX55`, `FX65`)**: Saves or restores registers `V0` through `Vx` to/from RAM starting at address `I`.

### 3. Real-Time Disassembler (`src/disassembler.rs`)

Translates raw 16-bit hexadecimal opcodes into human-readable assembly instructions (e.g. `0xD015` -> `DRW V0, V1, 5`, `0x6004` -> `LD V0, 0x04`).

### 4. Display Rasterizer (`src/display.rs`)

Maps the 64x32 monochrome boolean pixel buffer into formatted ASCII graphics (`#` for active pixels, spaces for inactive pixels) framed by borders.

### 5. Built-in Demonstration ROM (`src/roms.rs`)

Includes an embedded machine-code test ROM that clears the display, loads font sprites from memory, draws digits across the screen, and loops.

---

## Running the Project

### Run Built-in Demonstration ROM

```bash
cargo run -p chip8-emulator
```

The output shows instruction execution with real-time disassembly followed by the rendered 64x32 display buffer:

```
Cycle 00 | PC: 0x200 | Opcode: 0x00E0 -> CLS
Cycle 01 | PC: 0x202 | Opcode: 0x6004 -> LD V0, 0x04
Cycle 02 | PC: 0x204 | Opcode: 0x6104 -> LD V1, 0x04
Cycle 03 | PC: 0x206 | Opcode: 0xA050 -> LD I, 0x050
Cycle 04 | PC: 0x208 | Opcode: 0xD015 -> DRW V0, V1, 5
...
```

### Run an External ROM File

```bash
cargo run -p chip8-emulator -- path/to/game.ch8
```

### Run Unit Tests

```bash
cargo test -p chip8-emulator
```
