//! Bus interface for memory access
//!
//! The bus is responsible for routing memory read/write operations to the
//! appropriate memory regions (RAM, ROM, memory-mapped I/O devices, etc.)
//! and coordinating peripheral device updates.

/// Bus trait that all system buses must implement.
///
/// This is the primary interface between the CPU and the memory system.
/// Implementations handle address decoding and routing to appropriate
/// memory regions or peripherals.
pub trait Bus {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);

    fn read_word(&mut self, address: u16) -> u16 {
        let low = self.read(address) as u16;
        let high = self.read(address.wrapping_add(1)) as u16;
        (high << 8) | low
    }

    fn tick(&mut self);
}

/// Provides 64KB of RAM for testing.
pub struct SimpleBus {
    memory: [u8; 65536],
}

impl Default for SimpleBus {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleBus {
    pub fn new() -> Self {
        Self { memory: [0; 65536] }
    }

    pub fn load(&mut self, address: u16, data: &[u8]) {
        let start = address as usize;
        let end = start + data.len();
        self.memory[start..end].copy_from_slice(data);
    }

    #[allow(dead_code)]
    pub fn get_memory(&self, start: u16, len: usize) -> &[u8] {
        let start = start as usize;
        &self.memory[start..start + len]
    }
}

impl Bus for SimpleBus {
    fn read(&mut self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    fn tick(&mut self) {
        // No peripherals to update in simple bus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_bus_read_write() {
        let mut bus = SimpleBus::new();

        bus.write(0x1234, 0xAB);
        assert_eq!(bus.read(0x1234), 0xAB);

        bus.write(0x00, 0x42);
        assert_eq!(bus.read(0x00), 0x42);

        bus.write(0xFFFF, 0xFF);
        assert_eq!(bus.read(0xFFFF), 0xFF);
    }

    #[test]
    fn test_simple_bus_read_word() {
        let mut bus = SimpleBus::new();

        bus.write(0x1000, 0x34); // low byte
        bus.write(0x1001, 0x12); // high byte

        assert_eq!(bus.read_word(0x1000), 0x1234);
    }

    #[test]
    fn test_simple_bus_load() {
        let mut bus = SimpleBus::new();
        let data: [u8; 4] = [0x01, 0x02, 0x03, 0x04];

        bus.load(0x8000, data.as_slice());

        assert_eq!(bus.read(0x8000), 0x01);
        assert_eq!(bus.read(0x8001), 0x02);
        assert_eq!(bus.read(0x8002), 0x03);
        assert_eq!(bus.read(0x8003), 0x04);
    }

    #[test]
    fn test_read_word_wrapping() {
        let mut bus = SimpleBus::new();

        // Test reading word at end of memory (should wrap)
        bus.write(0xFFFF, 0x34); // low byte at last address
        bus.write(0x0000, 0x12); // high byte wraps to first address

        assert_eq!(bus.read_word(0xFFFF), 0x1234);
    }
}
