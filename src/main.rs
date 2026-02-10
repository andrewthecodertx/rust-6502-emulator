use mos6502::{Bus, Cpu, bus::SimpleBus, instructions::OPCODES, status::Flag};
use std::{env, fs, process, thread, time::Duration};

const CLEAR_SCREEN: &str = "\x1b[2J";
const CURSOR_HOME: &str = "\x1b[H";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const WHITE: &str = "\x1b[37m";

const ROM_START: u16 = 0x8000;
const ROM_SIZE: usize = 0x8000; // 32KB ROM from $8000-$FFFF

fn display_cpu<B: Bus>(cpu: &mut Cpu<B>, instruction_count: u32, total_cycles: u64) {
    let opcode_byte = cpu.bus.read(cpu.pc);
    let opcode = &OPCODES[opcode_byte as usize];

    // Build operand display based on instruction size
    let operand_str = match opcode.bytes {
        1 => String::new(),
        2 => format!(" ${:02X}", cpu.bus.read(cpu.pc.wrapping_add(1))),
        3 => format!(
            " ${:02X}{:02X}",
            cpu.bus.read(cpu.pc.wrapping_add(2)),
            cpu.bus.read(cpu.pc.wrapping_add(1))
        ),
        _ => String::new(),
    };

    print!("{CURSOR_HOME}");

    println!("{BOLD}{CYAN}╔══════════════════════════════════════════════════════════╗{RESET}");
    println!(
        "{BOLD}{CYAN}║{RESET}                    {BOLD}{WHITE}MOS6502 EMULATOR{RESET}                      {BOLD}{CYAN}║{RESET}"
    );
    println!("{BOLD}{CYAN}╠══════════════════════════════════════════════════════════╣{RESET}");
    println!(
        "{BOLD}{CYAN}║{RESET}  {DIM}Instructions:{RESET} {GREEN}{:5}{RESET}              {DIM}Cycles:{RESET} {GREEN}{:8}{RESET}       {BOLD}{CYAN}║{RESET}",
        instruction_count, total_cycles
    );
    println!("{BOLD}{CYAN}╠══════════════════════════════════════════════════════════╣{RESET}");
    println!(
        "{BOLD}{CYAN}║{RESET}  {BOLD}{YELLOW}REGISTERS{RESET}                                               {BOLD}{CYAN}║{RESET}"
    );
    println!(
        "{BOLD}{CYAN}║{RESET}                                                          {BOLD}{CYAN}║{RESET}"
    );
    println!(
        "{BOLD}{CYAN}║{RESET}    {DIM}A:{RESET}  {GREEN}${:02X}{RESET}  ({:3}) {DIM}PC:{RESET} {GREEN}${:04X}{RESET}                              {BOLD}{CYAN}║{RESET}",
        cpu.a, cpu.a, cpu.pc
    );
    println!(
        "{BOLD}{CYAN}║{RESET}    {DIM}X:{RESET}  {GREEN}${:02X}{RESET}  ({:3}) {DIM}SP:{RESET} {GREEN}${:02X}{RESET}                                {BOLD}{CYAN}║{RESET}",
        cpu.x, cpu.x, cpu.sp
    );
    println!(
        "{BOLD}{CYAN}║{RESET}    {DIM}Y:{RESET}  {GREEN}${:02X}{RESET}  ({:3})                                        {BOLD}{CYAN}║{RESET}",
        cpu.y, cpu.y
    );
    println!(
        "{BOLD}{CYAN}║{RESET}                                                          {BOLD}{CYAN}║{RESET}"
    );
    println!("{BOLD}{CYAN}╠══════════════════════════════════════════════════════════╣{RESET}");
    println!(
        "{BOLD}{CYAN}║{RESET}  {BOLD}{YELLOW}STATUS FLAGS{RESET}                                            {BOLD}{CYAN}║{RESET}"
    );
    println!(
        "{BOLD}{CYAN}║{RESET}                                                          {BOLD}{CYAN}║{RESET}"
    );
    println!(
        "{BOLD}{CYAN}║{RESET}    {DIM}N V - B D I Z C{RESET}                                       {BOLD}{CYAN}║{RESET}"
    );
    println!(
        "{BOLD}{CYAN}║{RESET}    {GREEN}{} {} {} {} {} {} {} {}{RESET}                                       {BOLD}{CYAN}║{RESET}",
        cpu.status.get(Flag::Negative) as u8,
        cpu.status.get(Flag::Overflow) as u8,
        1, // Unused bit, always 1
        cpu.status.get(Flag::Break) as u8,
        cpu.status.get(Flag::DecimalMode) as u8,
        cpu.status.get(Flag::InterruptDisable) as u8,
        cpu.status.get(Flag::Zero) as u8,
        cpu.status.get(Flag::Carry) as u8
    );
    println!(
        "{BOLD}{CYAN}║{RESET}                                                          {BOLD}{CYAN}║{RESET}"
    );
    println!("{BOLD}{CYAN}╠══════════════════════════════════════════════════════════╣{RESET}");
    println!(
        "{BOLD}{CYAN}║{RESET}  {BOLD}{YELLOW}NEXT INSTRUCTION{RESET}                                        {BOLD}{CYAN}║{RESET}"
    );
    println!(
        "{BOLD}{CYAN}║{RESET}                                                          {BOLD}{CYAN}║{RESET}"
    );
    println!(
        "{BOLD}{CYAN}║{RESET}    {GREEN}${:04X}{RESET}: {BOLD}{WHITE}{}{RESET}{:<12}  {DIM}[{:02X}]{RESET}                          {BOLD}{CYAN}║{RESET}",
        cpu.pc, opcode.mnemonic, operand_str, opcode_byte
    );
    println!(
        "{BOLD}{CYAN}║{RESET}                                                          {BOLD}{CYAN}║{RESET}"
    );
    println!("{BOLD}{CYAN}╚══════════════════════════════════════════════════════════╝{RESET}");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("{BOLD}{WHITE}MOS 6502 Emulator{RESET}");
        eprintln!();
        eprintln!(
            "{DIM}Usage:{RESET} {} <rom.bin> [--delay <ms>] [--max <instructions>]",
            args[0]
        );
        eprintln!();
        eprintln!("{DIM}The ROM file should be a 32KB binary image ($8000-$FFFF).{RESET}");
        eprintln!("{DIM}Reset vector at $FFFC-$FFFD, IRQ/BRK at $FFFE-$FFFF.{RESET}");
        eprintln!();
        eprintln!("{DIM}Build ROMs with cc65:{RESET}");
        eprintln!("  ./bin/cl65 -t none -C examples/emu.cfg -o rom.bin program.s");
        process::exit(1);
    }

    let rom_path = &args[1];

    // Parse optional arguments
    let mut delay_ms: u64 = 150;
    let mut max_instructions: u32 = 10000;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--delay" => {
                i += 1;
                if i < args.len() {
                    delay_ms = args[i].parse().unwrap_or(150);
                }
            }
            "--max" => {
                i += 1;
                if i < args.len() {
                    max_instructions = args[i].parse().unwrap_or(10000);
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Load ROM file
    let rom_data = match fs::read(rom_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("{RED}Error:{RESET} Failed to read '{rom_path}': {e}");
            process::exit(1);
        }
    };

    if rom_data.len() != ROM_SIZE {
        eprintln!(
            "{RED}Error:{RESET} ROM must be exactly {} bytes (32KB), got {} bytes",
            ROM_SIZE,
            rom_data.len()
        );
        eprintln!(
            "{DIM}Use the linker config in examples/emu.cfg to generate correct ROMs.{RESET}"
        );
        process::exit(1);
    }

    // Load ROM into memory at $8000
    let mut bus = SimpleBus::new();
    bus.load(ROM_START, &rom_data);

    let mut cpu = Cpu::new(bus);
    cpu.reset();

    // Consume reset cycles
    let mut total_cycles: u64 = 0;
    while cpu.cycles > 0 {
        cpu.step();
        total_cycles += 1;
    }

    print!("{CLEAR_SCREEN}");

    let mut instruction_count: u32 = 0;
    let delay = Duration::from_millis(delay_ms);

    while instruction_count < max_instructions {
        display_cpu(&mut cpu, instruction_count, total_cycles);
        thread::sleep(delay);

        let pc_before = cpu.pc;
        let opcode_byte = cpu.bus.read(pc_before);

        // BRK stops execution
        if opcode_byte == 0x00 {
            break;
        }

        cpu.execute_instruction();
        instruction_count += 1;
        total_cycles += OPCODES[opcode_byte as usize].cycles as u64;
    }

    display_cpu(&mut cpu, instruction_count, total_cycles);

    println!();
    println!(
        "{GREEN}Execution complete! BRK encountered at ${:04X}{RESET}",
        cpu.pc
    );
}
