// Reference: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM

use std::fs::{self};

// 2.1 - Memory
// Most Chip-8 programs start at location 0x200 (512), but some begin at
// 0x600 (1536). Programs beginning at 0x600 are intended for the ETI 660
// computer.
const NORMAL_START_INDEX: usize = 512;
// const ETI_660_START_INDEX: usize = 1526;

pub struct Chip8 {
    // 2.1 - Memory
    // The Chip-8 language is capable of accessing up to 4KB (4,096 bytes) of
    // RAM, from location 0x000 (0) to 0xFFF (4095).
    ram: [u8; 4096],

    // 2.2 - Registers
    registers: Registers,
    // There are also some "pseudo-registers" which are not accessable from
    // Chip-8 programs.

    // NOTE: While spec asks for 16-bit numbers, we use usize to simplify direct
    //       access of arrays in rust.

    // The program counter (PC) should be 16-bit, and is used to store the
    // currently executing address.
    pc: usize,

    // The stack pointer (SP) can be 8-bit, it is used to point to the topmost
    // level of the stack.
    sp: usize,

    // The stack is an array of 16 16-bit values, used to store the address that
    // the interpreter shoud return to when finished with a subroutine. Chip-8
    // allows for up to 16 levels of nested subroutines.
    stack: [usize; 16],
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            ram: [0; 4096],
            registers: Registers::new(),
            pc: 0,
            sp: 0,
            stack: [0; 16],
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        let bytes = fs::read(path).expect("Could not open file.");

        // TODO: Take a CLI flag for the start address to load into memory, for
        //       now we just use the more common 0x200 start address.
        for (index, byte) in bytes.iter().enumerate() {
            self.ram[NORMAL_START_INDEX + index] = *byte;
        }

        eprintln!("bytes loaded: {}", bytes.len());
    }

    pub fn run(&mut self) {
        // TODO: Set PC start based on CLI flag for start address
        self.pc = NORMAL_START_INDEX;

        loop {
            let high_byte = self.high_byte();
            let low_byte = self.low_byte();

            match high(high_byte) {
                0xA => {
                    self.pc = self.load_i();
                }
                0x6 => {
                    self.pc = self.load_vx();
                }
                _ => {
                    eprintln!("Unrecognised instrution 0x{:x}{:x}", high_byte, low_byte);
                    break;
                }
            }
        }
    }

    // Annn - LD I, addr
    fn load_i(&mut self) -> usize {
        // The value of register I is set to nnn
        self.registers.i = self.addr();

        self.pc + 2
    }

    // 6xkk - LD Vx, byte
    fn load_vx(&mut self) -> usize {
        // The interpreter puts the value kk into register Vx.
        let register = low(self.high_byte());
        self.registers.put(register, *self.low_byte());

        self.pc + 2
    }

    // 3.0 - Chip-8 Instrutions
    // All instructions are 2 bytes long and are stored
    // most-significant-byte first. In memory, the first byte of each
    // instruction should be located at an even addresses.
    fn high_byte(&self) -> &u8 {
        &self.ram[self.pc]
    }

    fn low_byte(&self) -> &u8 {
        &self.ram[self.pc + 1]
    }

    fn addr(&self) -> u16 {
        let mask = (1 << 12) - 1;
        self.instruction() & mask
    }

    fn instruction(&self) -> u16 {
        ((*self.high_byte() as u16) << 8) | *self.low_byte() as u16
    }

    pub fn dump_to_stdout(&self) {
        println!("=== MEMORY ===");
        for line in self.ram.chunks(64) {
            for instruction in line.chunks(2) {
                print!("{:02X}{:02X} ", instruction[0], instruction[1]);
            }
            print!("\n");
        }

        println!();
        println!("=== REGISTERS ===");
        self.registers.dump_to_stdout();

        println!();
        println!("=== CPU STATE ===");
        println!("pc: {:04X}", self.pc);
        println!("sp: {:04X}", self.sp);
        println!("stack: {:?}", self.stack);
    }
}

fn high(byte: &u8) -> u8 {
    let mask = (1 << 4) - 1;
    (byte & mask << 4) >> 4
}

fn low(byte: &u8) -> u8 {
    let mask = (1 << 4) - 1;
    byte & mask
}

// 2.2 - Registers
pub struct Registers {
    // Chip-8 has 16 general purpose 8-bit registers, usually referred to as Vx,
    // where x is a hexadecimal digit (0 through F).
    v_0: u8,
    v_1: u8,
    v_2: u8,
    v_3: u8,
    v_4: u8,
    v_5: u8,
    v_6: u8,
    v_7: u8,
    v_8: u8,
    v_9: u8,
    v_a: u8,
    v_b: u8,
    v_c: u8,
    v_d: u8,
    v_e: u8,
    v_f: u8,

    // There is also a 16-bit register called I. This register is generally
    // used to store memory addresses
    i: u16,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            v_0: 0,
            v_1: 0,
            v_2: 0,
            v_3: 0,
            v_4: 0,
            v_5: 0,
            v_6: 0,
            v_7: 0,
            v_8: 0,
            v_9: 0,
            v_a: 0,
            v_b: 0,
            v_c: 0,
            v_d: 0,
            v_e: 0,
            v_f: 0,
            i: 0,
        }
    }

    pub fn put(&mut self, register: u8, value: u8) {
        match register {
            0x0 => self.v_0 = value,
            0x1 => self.v_1 = value,
            0x2 => self.v_2 = value,
            0x3 => self.v_3 = value,
            0x4 => self.v_4 = value,
            0x5 => self.v_5 = value,
            0x6 => self.v_6 = value,
            0x7 => self.v_7 = value,
            0x8 => self.v_8 = value,
            0x9 => self.v_9 = value,
            0xa => self.v_a = value,
            0xb => self.v_b = value,
            0xc => self.v_c = value,
            0xd => self.v_d = value,
            0xe => self.v_e = value,
            0xf => self.v_f = value,
            _ => panic!(
                "Tried to set a register that doesn't exist v_{:x}",
                register
            ),
        }
    }

    pub fn dump_to_stdout(&self) {
        print!("v_0: {:02X} ", self.v_0);
        print!("v_1: {:02X} ", self.v_1);
        print!("v_2: {:02X} ", self.v_2);
        print!("v_3: {:02X} ", self.v_3);
        print!("v_4: {:02X} ", self.v_4);
        print!("v_5: {:02X} ", self.v_5);
        print!("v_6: {:02X} ", self.v_6);
        print!("v_7: {:02X} ", self.v_7);
        print!("v_8: {:02X} ", self.v_8);
        print!("v_9: {:02X} ", self.v_9);
        print!("v_a: {:02X} ", self.v_a);
        print!("v_b: {:02X} ", self.v_b);
        print!("v_c: {:02X} ", self.v_c);
        print!("v_d: {:02X} ", self.v_d);
        print!("v_e: {:02X} ", self.v_e);
        print!("v_f: {:02X} ", self.v_f);
        println!();
        println!("i: {:04X}", self.i)
    }
}
