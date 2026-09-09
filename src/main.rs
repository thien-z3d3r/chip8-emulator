use chip8_emulator::{disassembler, display, roms, Cpu};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom = if args.len() > 1 {
        println!("Loading ROM from file: {}", args[1]);
        fs::read(&args[1]).expect("Failed to read ROM file")
    } else {
        println!("No ROM provided. Running built-in graphics demo ROM.");
        roms::sample_test_rom()
    };

    let mut cpu = Cpu::new();
    cpu.load_rom(&rom);

    println!("============================================================");
    println!("             CHIP-8 Virtual Machine & Emulator              ");
    println!("============================================================");
    println!("ROM loaded: {} bytes\n", rom.len());

    println!("Executing instructions with real-time disassembly:");
    for cycle in 0..15 {
        let pc = cpu.pc;
        let opcode = cpu.step();
        let mnemonic = disassembler::disassemble(opcode);
        println!("  Cycle {:02} | PC: 0x{:03X} | Opcode: 0x{:04X} -> {}", cycle, pc, opcode, mnemonic);
    }

    println!("\nRendered Display Output (64x32 monochrome buffer):");
    let ascii_screen = display::render_ascii(&cpu.display);
    println!("{}", ascii_screen);

    println!("CHIP-8 emulation cycle completed successfully!");
}
