use std::{collections::HashMap, ops::Add};

use crate::opcodes;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Flag {
    Carry = 1 << 0,
    Zero = 1 << 1,
    Interrupt = 1 << 2,
    Decimal = 1 << 3,
    Overflow = 1 << 6,
    Negative = 1 << 7,
}

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub status: u8,
    memory: [u8; 0xFFFF],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            program_counter: 0,
            stack_pointer: 0,
            status: 0,
            memory: [0; 0xFFFF],
        }
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub fn mem_read_u16(&self, pos: u16) -> u16 {
        let low: u16 = self.mem_read(pos) as u16;
        let high: u16 = self.mem_read(pos + 1) as u16;
        (high << 8) | (low as u16)
    }

    pub fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let low = (data & 0x0F) as u8;
        let high = (data >> 8) as u8;
        self.mem_write(pos, low);
        self.mem_write(pos + 1, high);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.run();
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..0x8000 + program.len()].copy_from_slice(&program[..]);
        self.program_counter = 0x8000;
    }

    pub fn run(&mut self) {
        let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;

        loop {
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;

            let opcode = opcodes
                .get(&code)
                .expect(&format!("OpCode {:x} is not recognized", code));

            // OpCode::new(0xa9, "LDA", 2, 2, AddressingMode::Immediate),
            // OpCode::new(0xa5, "LDA", 2, 3, AddressingMode::ZeroPage),
            // OpCode::new(0xb5, "LDA", 2, 4, AddressingMode::ZeroPage_X),
            // OpCode::new(0xad, "LDA", 3, 4, AddressingMode::Absolute),
            // OpCode::new(0xbd, "LDA", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
            // OpCode::new(0xb9, "LDA", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
            // OpCode::new(0xa1, "LDA", 2, 6, AddressingMode::Indirect_X),
            // OpCode::new(0xb1, "LDA", 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

            match code {
                0xe8 => {
                    self.inx();
                }
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => {
                    self.lda(&opcode.mode);
                }
                0xaa => {
                    self.tax();
                }
                0xc0 => {
                    // let param = self.mem_read(self.program_counter);
                    // self.program_counter += 1;
                    // let cmp = self.register_y - param;
                    self.cpy(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0xa8 => {
                    self.tay();
                }
                /*
                 * BRK - Force Interrupt
                 * The BRK instruction forces the generation of an interrupt request.
                 * The program counter and processor status are pushed on the stack then the IRQ interrupt vector
                 * at $FFFE/F is loaded into the PC and the break flag in the status set to one.
                 */
                0x00 => return,
                _ => {}
            }

            self.program_counter += (opcode.len - 1) as u16;
        }
    }

    fn inx(&mut self) {
        if self.register_x != 0xFF {
            self.register_x += 1;
            self.set_flag(Flag::Negative, self.register_x & 0x80 > 0);
        } else {
            self.register_x = 0;
            self.set_flag(Flag::Zero, true);
        }
    }
    /*
     * CPY - Compare Y Register
     * Z,C,N = Y-M
     * This instruction compares the contents of the Y register
     * with another memory held value and sets the zero and carry flags as appropriate.
     */
    fn cpy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let m = self.mem_read(addr);
        let cmp: i16 = self.register_y as i16 - m as i16;

        if cmp == 0 {
            self.set_flag(Flag::Zero, true);
            self.set_flag(Flag::Carry, true);
        }

        if cmp < 0 {
            self.set_flag(Flag::Negative, true);
        } else {
            self.set_flag(Flag::Carry, true);
        }
    }
    /*
     * TAY - Transfer Accumulator to Y
     * Y = A
     * Copies the current contents of the accumulator into the Y register
     * and sets the zero and negative flags as appropriate.
     */
    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.set_zero_and_negative_flag(self.register_y);
    }
    /*
     * TAX - Transfer Accumulator to X
     * X = A
     * Copies the current contents of the accumulator into the X register
     * and sets the zero and negative flags as appropriate.
     */
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.set_zero_and_negative_flag(self.register_x);
    }
    /*
     * LDA - Load Accumulator
     * A,Z,N = M
     * Loads a byte of memory into the accumulator setting the zero
     * and negative flags as appropriate.
     */
    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.register_a = self.mem_read(addr);
        self.set_zero_and_negative_flag(self.register_a);
    }

    fn set_zero_and_negative_flag(&mut self, param: u8) {
        if param == 0 {
            self.set_flag(Flag::Zero, true);
        } else {
            self.set_flag(Flag::Zero, false);
        }

        if param & 0b1000_0000 != 0 {
            self.set_flag(Flag::Negative, true);
        } else {
            self.set_flag(Flag::Negative, false);
        }
    }

    fn set_flag(&mut self, flag: Flag, enabled: bool) {
        if enabled {
            self.status |= flag as u8;
        } else {
            self.status &= !(flag as u8);
        }
    }

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => {
                self.program_counter
            }
            AddressingMode::ZeroPage => {
                self.mem_read(self.program_counter) as u16
            }
            AddressingMode::ZeroPage_X => {
                self.mem_read(self.program_counter) as u16 + self.register_x as u16
            }
            AddressingMode::ZeroPage_Y => {
                self.mem_read(self.program_counter) as u16 + self.register_y as u16
            }
            AddressingMode::Absolute => {
                self.mem_read_u16(self.program_counter)
            }
            AddressingMode::Absolute_X => {
                self.mem_read_u16(self.program_counter) + self.register_x as u16
            }
            AddressingMode::Absolute_Y => {
                self.mem_read_u16(self.program_counter) + self.register_y as u16
            }
            AddressingMode::Indirect_X => {
                todo!()
            }
            AddressingMode::Indirect_Y => {
                todo!()
            }
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

///
///
///
/// test cases for LDA
///
///
///
#[test]
fn test_lda_zero_page() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x0006, 0xFA);
    cpu.load_and_run(vec![0xa5, 0x06, 0x00]);
    assert_eq!(cpu.register_a, 0xFA);
}

#[test]
fn test_lda_zero_page_x() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x0006, 0xFA);
    cpu.register_x = 0x05;
    cpu.load_and_run(vec![0xb5, 0x01, 0x00]);
    assert_eq!(cpu.register_a, 0xFA);
}

#[test]
fn test_lda_zero_page_y() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x0006, 0xFA);
    cpu.register_x = 0x03;
    cpu.load_and_run(vec![0xb5, 0x03, 0x00]);
    assert_eq!(cpu.register_a, 0xFA);
}

#[test]
fn test_lda_zero_absolute() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x1234, 0xFA);
    cpu.load_and_run(vec![0xad, 0x34, 0x12, 0x00]);
    assert_eq!(cpu.register_a, 0xFA);
}

#[test]
fn test_lda_zero_absolute_x() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x1234, 0xFA);
    cpu.register_x = 0x10;
    cpu.load_and_run(vec![0xbd, 0x24, 0x12, 0x00]);
    assert_eq!(cpu.register_a, 0xFA);
}

#[test]
fn test_lda_zero_absolute_y() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x1234, 0xFA);
    cpu.register_y = 0x20;
    cpu.load_and_run(vec![0xb9, 0x14, 0x12, 0x00]);
    assert_eq!(cpu.register_a, 0xFA);
}

#[test]
fn test_0xa9_lda_immediate_load_data() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
    assert_eq!(cpu.register_a, 0x05);
    assert!(cpu.status & 0b0000_0010 == 0b00);
    assert!(cpu.status & 0b1000_0000 == 0);
}

#[test]
fn test_0xa9_lda_zero_flag() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
    assert!(cpu.status & 0b0000_0010 == 0b10);
}

    #[test]
    fn test_0xc0_cpy_compare_y_register_set_carry() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x30;
        cpu.load_and_run(vec![0xc0, 0x29, 0x00]);
        assert!(cpu.status & 0b1000_0011 == 0b0000_0001);
    }

    #[test]
    fn test_0xc0_cpy_compare_y_register_set_zero() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x29;
        cpu.load_and_run(vec![0xc0, 0x29, 0x00]);
        assert_eq!(cpu.status & Flag::Zero as u8, Flag::Zero as u8);
    }

    #[test]
    fn test_0xc0_cpy_compare_y_register_set_negative() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x20;
        cpu.load_and_run(vec![0xc0, 0x29, 0x00]);
        assert!(cpu.status & 0b1000_0011 == 0b1000_0000);
    }

    #[test]
    fn test_0xa8_tay_transfer_accumulator_to_y() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x23;
        cpu.load_and_run(vec![0xa8, 0x00]);
        assert_eq!(cpu.register_y, cpu.register_a);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa8_tay_transfer_accumulator_to_y_zero_flag() {
        let mut cpu = CPU::new();
        cpu.register_a = 0;
        cpu.load_and_run(vec![0xa8, 0x00]);
        assert_eq!(cpu.register_y, cpu.register_a);
        assert!(cpu.status & 0b0000_0010 == 0b0000_0010);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa8_tay_transfer_accumulator_to_y_negative_flag() {
        let mut cpu = CPU::new();
        cpu.register_a = 0xF0;
        cpu.load_and_run(vec![0xa8, 0x00]);
        assert_eq!(cpu.register_y, cpu.register_a);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 != 0);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 10;
        cpu.load_and_run(vec![0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x_zero_flag_on() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x00;
        cpu.load_and_run(vec![0xaa, 0x00]);
        assert!(cpu.status & 0b0000_0010 != 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x_zero_negative_flag_on() {
        let mut cpu = CPU::new();
        cpu.register_a = 0xf1;
        cpu.load_and_run(vec![0xaa, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 != 0);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.load_and_run(vec![0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0);
        assert!(cpu.status & Flag::Zero as u8 != 0);
    }

    #[test]
    fn test_inx_positive() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x11;
        cpu.load_and_run(vec![0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0x13);
        assert!(cpu.status & Flag::Zero as u8 == 0);
        assert!(cpu.status & Flag::Negative as u8 == 0);
    }


}
