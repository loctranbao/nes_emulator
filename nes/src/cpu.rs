use std::collections::HashMap;

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
    Accumulator,
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

const STACK: u16 = 0x0100;
const STACK_RESET: u8 = 0xfd;

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
        self.stack_pointer = 0xff;
    }

    pub fn run(&mut self) {
        let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;

        loop {
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let program_counter_state = self.program_counter;

            let opcode = opcodes
                .get(&code)
                .expect(&format!("OpCode {:x} is not recognized", code));
            // OpCode::new(0x08, "PHP", 1, 3, AddressingMode::NoneAddressing),
            // OpCode::new(0x48, "PHA", 1, 3, AddressingMode::NoneAddressing),
            // OpCode::new(0x68, "PLA", 1, 4, AddressingMode::NoneAddressing),
            // OpCode::new(0x28, "PLP", 1, 4, AddressingMode::NoneAddressing),
            match code {
                0x08 => {
                    self.php();
                }
                0x48 => {
                    self.pha();
                }
                0x68 => {
                    self.pla();
                }
                0x28 => {
                    self.plp();
                }
                0x70 => {
                    self.branch(self.get_flag(Flag::Overflow) == true);
                }
                0x50 => {
                    self.branch(self.get_flag(Flag::Overflow) == false);
                }
                0x30 => {
                    self.branch(self.get_flag(Flag::Negative) == true);
                }
                0x10 => {
                    self.branch(self.get_flag(Flag::Negative) == false);
                }
                0xf0 => {
                    self.branch(self.get_flag(Flag::Zero) == true);
                }
                0xb0 => {
                    self.branch(self.get_flag(Flag::Carry) == true);
                }
                0x90 => {
                    self.branch(self.get_flag(Flag::Carry) == false);
                }
                0xc6 | 0xd6 | 0xce | 0xde => {
                    self.dec(&opcode.mode);
                }
                0x88 => {
                    self.dey();
                }
                0xca => {
                    self.dex();
                }
                0xd0 => {
                    self.branch(self.get_flag(Flag::Zero) == false);
                }
                0x6a => {
                    self.ror_a();
                }
                0x66 | 0x76 | 0x6e | 0x7e => {
                    self.ror(&opcode.mode);
                }
                0x38 => {
                    self.sec();
                }
                0x2a => {
                    self.rol_a();
                }
                0x26 | 0x36 | 0x2e | 0x3e => {
                    self.rol(&opcode.mode);
                }
                0x0a => {
                    self.asl_a();
                }
                0x06 | 0x16 | 0x0e | 0x1e => {
                    self.asl(&opcode.mode);
                }
                0x38 => {
                    self.sec();
                }
                0xe9 | 0xe5 | 0xf5 | 0xed | 0xfd | 0xf9 | 0xe1 | 0xf1 => {
                    self.sbc(&opcode.mode);
                }
                0x69 | 0x65 | 0x75 | 0x6d | 0x7d | 0x79 | 0x61 | 0x71 => {
                    self.adc(&opcode.mode);
                }
                0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe => {
                    self.ldx(&opcode.mode);
                }
                0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc => {
                    self.ldy(&opcode.mode);
                }
                0x86 | 0x96 | 0x8e => self.stx(&opcode.mode),
                0x84 | 0x94 | 0x8c => self.sty(&opcode.mode),
                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => {
                    self.sta(&opcode.mode);
                }
                0xe8 => {
                    self.inx();
                }
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => {
                    self.lda(&opcode.mode);
                }
                0xaa => {
                    self.tax();
                }
                0xe0 | 0xe4 | 0xec => {
                    self.cpx(&opcode.mode);
                }
                0xc0 | 0xc4 | 0xcc => {
                    self.cpy(&opcode.mode);
                }
                0xc9 | 0xc5 | 0xd5 | 0xcd | 0xdd | 0xd9 | 0xc1 | 0xd1 => {
                    self.cmp(&opcode.mode);
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
            if program_counter_state == self.program_counter {
                self.program_counter += (opcode.len - 1) as u16;
            }
        }
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        let m = self.mem_read(addr);

        let tmp = self.register_a as u16 + m as u16 + (self.status & 0x01) as u16;

        self.set_flag(Flag::Zero, tmp & 0x00FF == 0);
        self.set_flag(Flag::Negative, tmp & 0x0080 != 0);
        self.set_flag(Flag::Carry, tmp & 0x0100 != 0);
        self.set_flag(
            Flag::Overflow,
            (self.register_a as u16 ^ tmp) & !(self.register_a as u16 ^ m as u16) & 0x0080 != 0,
        );

        self.register_a = (tmp & 0x00FF) as u8;
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        let m = !self.mem_read(addr);

        let tmp = self.register_a as u16 + m as u16 + (self.status & 0x01) as u16;

        self.set_flag(Flag::Zero, tmp & 0x00FF == 0);
        self.set_flag(Flag::Negative, tmp & 0x0080 != 0);
        self.set_flag(Flag::Carry, tmp & 0x0100 != 0);
        self.set_flag(
            Flag::Overflow,
            (self.register_a as u16 ^ tmp) & !(self.register_a as u16 ^ m as u16) & 0x0080 != 0,
        );

        self.register_a = (tmp & 0x00FF) as u8;
    }
    fn sec(&mut self) {
        self.set_flag(Flag::Carry, true);
    }

    fn clc(&mut self) {
        self.set_flag(Flag::Carry, false);
    }

    fn sed(&mut self) {
        self.set_flag(Flag::Decimal, true);
    }

    fn sei(&mut self) {
        self.set_flag(Flag::Interrupt, true);
    }

    fn cld(&mut self) {
        self.set_flag(Flag::Decimal, false);
    }

    fn cli(&mut self) {
        self.set_flag(Flag::Interrupt, false);
    }

    fn clv(&mut self) {
        self.set_flag(Flag::Overflow, false);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        let m = self.mem_read(addr);
        self.register_a &= m;

        self.set_flag(Flag::Zero, self.register_a == 0);
        self.set_flag(Flag::Negative, self.register_a & 0x80 != 0);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        let mut m = self.mem_read(addr);
        self.set_flag(Flag::Carry, m & 0x80 != 0);
        m = m << 1;
        self.set_zero_and_negative_flag(m);
        self.mem_write(addr, m);
    }

    fn asl_a(&mut self) {
        self.set_flag(Flag::Carry, self.register_a & 0x80 != 0);
        self.register_a = self.register_a << 1;
        self.set_zero_and_negative_flag(self.register_a);
    }

    fn branch(&mut self, condition: bool) {
        if condition {
            let jmp = self.mem_read(self.program_counter) as i8;
            let jmp_addr = self
                .program_counter
                .wrapping_add(1)
                .wrapping_add(jmp as u16);
            self.program_counter = jmp_addr;
        }
    }

    fn brk(&mut self, mode: &AddressingMode) {}

    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        let mut m = self.mem_read(addr);
        m = m.wrapping_sub(1);
        self.mem_write(addr, m);
        self.set_zero_and_negative_flag(m);
    }

    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.set_zero_and_negative_flag(self.register_x);
    }

    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.set_zero_and_negative_flag(self.register_y);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        let m = self.mem_read(addr);
        self.register_a ^= m;
        self.set_zero_and_negative_flag(self.register_a);
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        let mut m = self.mem_read(addr);
        m = m.wrapping_add(1);
        self.mem_write(addr, m);
        self.set_zero_and_negative_flag(m);
    }
    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.set_zero_and_negative_flag(self.register_y);
    }
    fn jmp(&mut self, mode: &AddressingMode) {}
    fn jsr(&mut self, mode: &AddressingMode) {}

    fn lsr(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        let mut m = self.mem_read(addr);
        self.set_flag(Flag::Carry, m & 0x01 != 0);
        m = m >> 1;
        self.set_zero_and_negative_flag(m);
        self.mem_write(addr, m);
    }

    fn lsr_a(&mut self) {
        self.set_flag(Flag::Carry, self.register_a & 0x01 != 0);
        self.register_a = self.register_a >> 1;
        self.set_zero_and_negative_flag(self.register_a);
    }

    fn nop(&mut self) {
        //nothing
    }
    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        let m = self.mem_read(addr);
        self.register_a |= m;
        self.set_zero_and_negative_flag(self.register_a);
    }

    fn push_stack(&mut self, value: u8) {
        let addr = STACK as u16 | self.stack_pointer as u16;
        self.mem_write(addr, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn pop_stack(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let addr = STACK as u16 | self.stack_pointer as u16;
        self.mem_read(addr)
    }

    fn pha(&mut self) {
        let accumulator = self.register_a.clone();
        self.push_stack(accumulator);
    }
    fn php(&mut self) {
        let status = self.status.clone();
        self.push_stack(status);
    }
    fn pla(&mut self) {
        self.register_a = self.pop_stack();
    }
    fn plp(&mut self) {
        self.status = self.pop_stack();
    }

    fn rol(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        let mut m = self.mem_read(addr);

        let carry = {
            if self.get_flag(Flag::Carry) {
                1u8
            } else {
                0u8
            }
        };

        self.set_flag(Flag::Carry, m & 0x80 != 0);

        m = m << 1;
        m = m | carry;

        self.set_zero_and_negative_flag(m);
        self.mem_write(addr, m);
    }

    fn rol_a(&mut self) {
        let carry = {
            if self.get_flag(Flag::Carry) {
                1u8
            } else {
                0u8
            }
        };

        self.set_flag(Flag::Carry, self.register_a & 0x80 != 0);

        self.register_a = self.register_a << 1;
        self.register_a = self.register_a | carry;

        self.set_zero_and_negative_flag(self.register_a);
    }

    fn ror(&mut self, mode: &AddressingMode) {
        let carry = {
            if self.get_flag(Flag::Carry) {
                1u8 << 7
            } else {
                0u8
            }
        };

        let addr = self.fetch(mode);
        let mut m = self.mem_read(addr);

        self.set_flag(Flag::Carry, m & 0x01 != 0);
        m = m >> 1;
        m = m | carry;

        self.mem_write(addr, m);
        self.set_zero_and_negative_flag(m);
    }
    fn ror_a(&mut self) {
        let carry = {
            if self.get_flag(Flag::Carry) {
                1u8 << 7
            } else {
                0u8
            }
        };

        self.set_flag(Flag::Carry, self.register_a & 0x01 != 0);
        self.register_a = self.register_a >> 1;
        self.register_a = self.register_a | carry;

        self.set_zero_and_negative_flag(self.register_a);
    }
    fn rti(&mut self, mode: &AddressingMode) {}
    fn rts(&mut self, mode: &AddressingMode) {}

    fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.set_zero_and_negative_flag(self.register_x);
    }
    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.set_zero_and_negative_flag(self.register_a);
    }
    fn txs(&mut self) {
        self.stack_pointer = self.register_x;
        self.set_zero_and_negative_flag(self.stack_pointer);
    }
    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.set_zero_and_negative_flag(self.register_a);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.set_zero_and_negative_flag(self.register_x);
    }
    /*
     * CPY - Compare Y Register
     * Z,C,N = Y-M
     * This instruction compares the contents of the Y register
     * with another memory held value and sets the zero and carry flags as appropriate.
     */
    fn cpy(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        let m = self.mem_read(addr);
        let tmp: i16 = self.register_y as i16 - m as i16;

        self.set_flag(Flag::Zero, tmp == 0);
        self.set_flag(Flag::Carry, tmp >= 0);
        self.set_flag(Flag::Negative, tmp < 0);
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        let m = self.mem_read(addr);
        let cmp: i16 = self.register_x as i16 - m as i16;

        self.set_flag(Flag::Zero, cmp == 0);
        self.set_flag(Flag::Carry, cmp >= 0);
        self.set_flag(Flag::Negative, cmp < 0);
    }

    fn cmp(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        let m = self.mem_read(addr);
        let cmp: i16 = self.register_a as i16 - m as i16;

        self.set_flag(Flag::Zero, cmp == 0);
        self.set_flag(Flag::Carry, cmp >= 0);
        self.set_flag(Flag::Negative, cmp < 0);
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
        let addr = self.fetch(mode);
        self.register_a = self.mem_read(addr);
        self.set_zero_and_negative_flag(self.register_a);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        self.mem_write(addr, self.register_a);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        self.register_x = self.mem_read(addr);
        self.set_zero_and_negative_flag(self.register_x);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        self.mem_write(addr, self.register_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        self.register_y = self.mem_read(addr);
        self.set_zero_and_negative_flag(self.register_y);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.fetch(mode);
        self.mem_write(addr, self.register_y);
    }

    fn set_zero_and_negative_flag(&mut self, param: u8) {
        self.set_flag(Flag::Zero, param == 0);
        self.set_flag(Flag::Negative, param & 0b1000_0000 != 0);
    }

    fn set_flag(&mut self, flag: Flag, enabled: bool) {
        if enabled {
            self.status |= flag as u8;
        } else {
            self.status &= !(flag as u8);
        }
    }

    fn get_flag(&self, flag: Flag) -> bool {
        self.status & flag as u8 != 0
    }

    fn fetch(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::ZeroPage_X => {
                self.mem_read(self.program_counter) as u16 + self.register_x as u16
            }
            AddressingMode::ZeroPage_Y => {
                self.mem_read(self.program_counter) as u16 + self.register_y as u16
            }
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::Absolute_X => {
                self.mem_read_u16(self.program_counter) + self.register_x as u16
            }
            AddressingMode::Absolute_Y => {
                self.mem_read_u16(self.program_counter) + self.register_y as u16
            }
            AddressingMode::Indirect_X => {
                let pos = self.mem_read(self.program_counter) + self.register_x;
                self.mem_read_u16(pos as u16)
            }
            AddressingMode::Indirect_Y => {
                let pos = self.mem_read(self.program_counter) + self.register_y;
                self.mem_read_u16(pos as u16)
            }
            _ => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_stack_function() {
        // a9 aa 08 48 28 68
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xaa, 0x08, 0x48, 0x28, 0x68, 0x00]);
        
        assert_eq!(cpu.register_a, 0x80);
        assert_eq!(cpu.status, 0xaa);
    }

    #[test]
    fn test_bne() {
        // a2 08 ca 8e 00 02 e0 03 d0 f8 8e 01 02 00
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa2, 0x08, 0xca, 0x8e, 0x00, 0x02, 0xe0, 0x03, 0xd0, 0xf8, 0x8e, 0x01, 0x02, 0x00,
        ]);
        assert_eq!(cpu.get_flag(Flag::Zero), true);
        assert_eq!(cpu.register_x, 0x03);
    }

    #[test]
    fn test_ror_a() {
        // 38 a9 ec 6a
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38, 0xa9, 0xec, 0x6a, 0x00]);

        assert_eq!(cpu.register_a, 0xf6);
        assert_eq!(cpu.get_flag(Flag::Negative), true);
        assert_eq!(cpu.get_flag(Flag::Carry), false);
        assert_eq!(cpu.get_flag(Flag::Zero), false);
    }

    #[test]
    fn test_ror() {
        // 38 a9 ed 85 02 66 02
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38, 0xa9, 0xed, 0x85, 0x02, 0x66, 0x02, 0x00]);

        assert_eq!(cpu.mem_read(0x0002), 0xf6);
        assert_eq!(cpu.get_flag(Flag::Negative), true);
        assert_eq!(cpu.get_flag(Flag::Carry), true);
        assert_eq!(cpu.get_flag(Flag::Zero), false);
    }

    #[test]
    fn test_rol() {
        // 38 a9 ec 85 02 26 02
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38, 0xa9, 0xec, 0x85, 0x02, 0x26, 0x02, 0x00]);

        assert_eq!(cpu.mem_read(0x0002), 0xd9);
        assert_eq!(cpu.get_flag(Flag::Carry), true);
        assert_eq!(cpu.get_flag(Flag::Zero), false);
        assert_eq!(cpu.get_flag(Flag::Negative), true);
    }

    #[test]
    fn test_rol_a() {
        //a9 76 2a
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38, 0xa9, 0xec, 0x2a, 0x00]);
        assert_eq!(cpu.get_flag(Flag::Carry), true);
        assert_eq!(cpu.get_flag(Flag::Zero), false);
        assert_eq!(cpu.get_flag(Flag::Negative), true);
        assert_eq!(cpu.register_a, 0xd9);
    }

    #[test]
    fn test_asl() {
        //a9 ec 85 02 06 02
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xec, 0x85, 0x02, 0x06, 0x02, 0x00]);
        assert_eq!(cpu.get_flag(Flag::Carry), true);
        assert_eq!(cpu.get_flag(Flag::Negative), true);
        assert_eq!(cpu.get_flag(Flag::Zero), false);
        assert_eq!(cpu.mem_read(0x0002), 0xd8);
    }

    #[test]
    fn test_asl_a() {
        //38 a9 ec 0a
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38, 0xa9, 0xec, 0x0a, 0x00]);
        assert_eq!(cpu.register_a, 0xd8);
        assert_eq!(cpu.get_flag(Flag::Carry), true);
        assert_eq!(cpu.get_flag(Flag::Negative), true);
        assert_eq!(cpu.get_flag(Flag::Zero), false);
    }

    #[test]
    fn test_sbc() {
        //a9 50 e9 b0 00
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x50, 0xe9, 0xb0, 0x00]);

        assert_eq!(cpu.register_a, 0x9f);
        assert_eq!(cpu.get_flag(Flag::Overflow), true);
        assert_eq!(cpu.get_flag(Flag::Negative), true);
        assert_eq!(cpu.get_flag(Flag::Carry), false);
        assert_eq!(cpu.get_flag(Flag::Zero), false);
    }

    #[test]
    fn test_adc_positive_overflow() {
        //a9 50 69 50
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x50, 0x69, 0x50, 0x00]);

        assert_eq!(cpu.register_a, 0xa0);
        assert_eq!(cpu.get_flag(Flag::Overflow), true);
        assert_eq!(cpu.get_flag(Flag::Zero), false);
        assert_eq!(cpu.get_flag(Flag::Carry), false);
        assert_eq!(cpu.get_flag(Flag::Negative), true);
    }

    #[test]
    fn test_adc_negative_overflow() {
        //a9 d0 69 90
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xd0, 0x69, 0x90, 0x00]);

        assert_eq!(cpu.register_a, 0x60);
        assert_eq!(cpu.get_flag(Flag::Overflow), true);
        assert_eq!(cpu.get_flag(Flag::Zero), false);
        assert_eq!(cpu.get_flag(Flag::Carry), true);
        assert_eq!(cpu.get_flag(Flag::Negative), false);
    }
    /**
     *
     *
     * test cases for LDA
     *
     *
     */
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
    fn test_lda_zero_indirect_x() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x01;
        cpu.register_a = 0x05;
        cpu.mem_write(0x0001, cpu.register_a);
        cpu.register_a = 0x07;
        cpu.mem_write(0x0002, cpu.register_a);
        cpu.register_y = 0x0a;
        cpu.mem_write(0x0705, cpu.register_y);

        cpu.load_and_run(vec![0xa1, 0x00, 0x00]);

        assert_eq!(cpu.register_a, 0x0a);
    }

    #[test]
    fn test_lda_zero_indirect_y() {
        let mut cpu = CPU::new();

        cpu.register_y = 0x02;
        cpu.mem_write_u16(0x0002, 0x0705);
        cpu.mem_write(0x0705, 0xfa);

        cpu.load_and_run(vec![0xb1, 0x00, 0x00]);

        assert_eq!(cpu.register_a, 0xfa);
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

    /*****************************************************/

    /**
     *
     *CPY test cases
     *
     *
     *
     *
     *
     */

    #[test]
    fn test_cpy_immediate() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0, 0x05, 0xc0, 0x05, 0x00]);
        assert!(cpu.status & Flag::Carry as u8 != 0);
        assert!(cpu.status & Flag::Zero as u8 != 0);
        assert!(cpu.status & Flag::Negative as u8 == 0);
    }

    #[test]
    fn test_cpy_zero() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0, 0x05, 0xa2, 0x04, 0x86, 0x02, 0xc4, 0x02, 0x00]);
        println!("{}", cpu.status);
        assert!(cpu.status & Flag::Carry as u8 != 0);
        assert!(cpu.status & Flag::Zero as u8 == 0);
        assert!(cpu.status & Flag::Negative as u8 == 0);
    }

    #[test]
    fn test_cpy_absolute() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa0, 0x05, 0xa2, 0x06, 0x8e, 0x34, 0x12, 0xcc, 0x34, 0x12, 0x00,
        ]);
        assert!(cpu.status & Flag::Carry as u8 == 0);
        assert!(cpu.status & Flag::Zero as u8 == 0);
        assert!(cpu.status & Flag::Negative as u8 != 0);
    }

    #[test]
    fn test_cpy_compare_y_register_set_carry() {
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
