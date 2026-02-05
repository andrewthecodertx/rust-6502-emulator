# Rust 6502 Emulator

A cycle-accurate emulator for the MOS 6502 processor, written in Rust.

The 6502 was everywhere in the late 70s and 80s — it powered the Apple II,
Commodore 64, Atari 2600, and the original NES. It's a beautifully simple
8-bit CPU with only a handful of registers and a 64KB address space, which
makes it a great target for learning about low-level programming and computer
architecture.

This emulator runs 6502 machine code and shows you exactly what's happening
inside the CPU as it executes. Watch the registers change, see the flags flip,
and step through instructions one at a time.

## Getting Started

You'll need Rust installed. Then:

```bash
cargo build --release
cargo run --release -- examples/count.bin
```

You should see a live display of the CPU state as it runs a simple counting
program.

### Command Line Options

```bash
cargo run -- <rom.bin> [--delay <ms>] [--max <instructions>]
```

- `--delay` controls how fast instructions execute (default: 150ms)
- `--max` sets a limit on instructions before stopping (default: 10000)

For example, to run faster:

```bash
cargo run -- examples/count.bin --delay 20
```

## Writing Programs

The emulator expects a 32KB ROM image that gets loaded at address $8000. The
reset vector at $FFFC tells the CPU where to start executing.

The `bin/` directory includes the cc65 assembler toolchain (ca65, ld65), and
`examples/` has a linker config that sets everything up correctly.

Here's what the example program looks like:

```asm
; count.s - Count from 0 to 10

.segment "CODE"

reset:
    lda #$00        ; Start at 0
loop:
    clc
    adc #$01        ; Add 1
    cmp #$0A        ; Reached 10?
    bne loop        ; Nope, keep going
    brk             ; Done

nmi:
irq:
    rti

.segment "VECTORS"
    .word nmi       ; $FFFA - NMI vector
    .word reset     ; $FFFC - Reset vector
    .word irq       ; $FFFE - IRQ vector
```

To assemble it:

```bash
cd examples
../bin/ca65 count.s -o count.o
../bin/ld65 -C emu.cfg -o count.bin count.o
```

The linker config (`emu.cfg`) handles placing code at the right addresses and
filling out the ROM to exactly 32KB.

## Using as a Library

The emulator is also a library you can use in your own projects. The CPU is
generic over a `Bus` trait, so you can implement your own memory maps and I/O.

```rust
use rust_6502_emulator::{Cpu, Bus, bus::SimpleBus};

// SimpleBus gives you flat 64KB of RAM
let mut bus = SimpleBus::new();
bus.load(0x8000, &program_bytes);

let mut cpu = Cpu::new(bus);
cpu.reset();

// Run one instruction at a time
cpu.execute_instruction();

// Or step one cycle at a time for precise timing
cpu.step();
```

To add memory-mapped I/O or bank switching, implement the `Bus` trait:

```rust
impl Bus for MyCustomBus {
    fn read(&mut self, address: u16) -> u8 { /* ... */ }
    fn write(&mut self, address: u16, value: u8) { /* ... */ }
    fn tick(&mut self) { /* called every cycle */ }
}
```

## What's Emulated

All official 6502 instructions work, including:

- Proper cycle timing (each instruction takes the right number of cycles)
- The infamous indirect JMP bug when crossing page boundaries
- NMI and IRQ interrupts
- Decimal mode for BCD arithmetic

The stack lives at $0100-$01FF and wraps around like the real chip. Status
flags behave correctly, including the quirky behavior of the B flag during
interrupts.

## Running Tests

```bash
cargo test
```

There are tests for individual instructions, addressing modes, and interrupt
handling.

## Can I Burn This to Real Hardware?

The emulator itself is Rust code that runs on your computer — you can't put
that on a 6502.

But the programs you assemble with ca65/ld65 are real 6502 machine code. If
you're building a 6502 computer (like Ben Eater's breadboard build), those
same binaries would run on actual hardware. You'd just need to burn the ROM
image to an EEPROM.

The memory map this emulator uses ($8000-$FFFF for ROM, vectors at the top)
is pretty standard and should work with most simple 6502 builds.
