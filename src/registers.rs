use crate::memory::GameBoyState;

#[derive(Default, Debug)]
pub struct Registers {
    pc: u16,
    sp: u16,
    bc: RegisterPair,
    af: RegisterPair,
    de: RegisterPair,
    hl: RegisterPair,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct RegisterPair {
    pub l: u8,
    pub r: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum R8 {
    B,
    C,
    A,
    F,
    D,
    E,
    H,
    L,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum R16 {
    PC,
    SP,
    BC,
    AF,
    DE,
    HL,
}

pub const ZERO_FLAG: u8 = 0x80;
pub const SUBTRACT_FLAG: u8 = 0x40;
pub const HALF_CARRY_FLAG: u8 = 0x20;
pub const CARRY_FLAG: u8 = 0x10;

impl Registers {
    pub fn get_pc(&self) -> u16 {
        self.read_r16(R16::PC)
    }
    pub fn inc_pc(&mut self, by: u16) {
        self.write_r16(R16::PC, self.get_pc().wrapping_add(by));
    }
    pub fn set_pc(&mut self, pc: u16) {
        self.write_r16(R16::PC, pc)
    }
    pub fn get_flags(&self) -> u8 {
        self.read_r8(R8::F)
    }
    pub fn set_flags(
        &mut self,
        zero: Option<bool>,
        subtract: Option<bool>,
        half_carry: Option<bool>,
        carry: Option<bool>,
    ) {
        let mut flags = self.read_r8(R8::F);
        if let Some(zero) = zero {
            flags = Registers::set_bit_flag(flags, ZERO_FLAG, zero);
        }
        if let Some(subtract) = subtract {
            flags = Registers::set_bit_flag(flags, SUBTRACT_FLAG, subtract);
        }
        if let Some(half_carry) = half_carry {
            flags = Registers::set_bit_flag(flags, HALF_CARRY_FLAG, half_carry);
        }
        if let Some(carry) = carry {
            flags = Registers::set_bit_flag(flags, CARRY_FLAG, carry);
        }
        self.write_r8(R8::F, flags);
    }
    pub fn set_bit_flag(flags: u8, bit: u8, set: bool) -> u8 {
        match set {
            true => flags | bit,
            false => flags & !bit,
        }
    }
    // pub fn zero_flag(&self) -> bool {
    //     self.get_flags() & ZERO_FLAG == ZERO_FLAG
    // }
    // pub fn sub_flag(&self) -> bool {
    //     self.get_flags() & SUBTRACT_FLAG == SUBTRACT_FLAG
    // }
    // pub fn half_carry_flag(&self) -> bool {
    //     self.get_flags() & HALF_CARRY_FLAG == HALF_CARRY_FLAG
    // }
    pub fn carry_flag(&self) -> bool {
        self.get_flags() & CARRY_FLAG == CARRY_FLAG
    }

    pub fn read_r8(&self, reg: R8) -> u8 {
        match reg {
            R8::B => self.bc.l,
            R8::C => self.bc.r,
            R8::A => self.af.l,
            R8::F => self.af.r,
            R8::D => self.de.l,
            R8::E => self.de.r,
            R8::H => self.hl.l,
            R8::L => self.hl.r,
        }
    }

    pub fn write_r8(&mut self, reg: R8, value: u8) {
        match reg {
            R8::B => self.bc.l = value,
            R8::C => self.bc.r = value,
            R8::A => self.af.l = value,
            R8::F => self.af.r = value,
            R8::D => self.de.l = value,
            R8::E => self.de.r = value,
            R8::H => self.hl.l = value,
            R8::L => self.hl.r = value,
        }
    }

    pub fn read_r16(&self, reg: R16) -> u16 {
        match reg {
            R16::PC => self.pc,
            R16::SP => self.sp,
            R16::BC => self.bc.into(),
            R16::AF => self.af.into(),
            R16::DE => self.de.into(),
            R16::HL => self.hl.into(),
        }
    }

    pub fn write_r16(&mut self, reg: R16, value: u16) {
        match reg {
            R16::PC => self.pc = value,
            R16::SP => self.sp = value,
            R16::BC => self.bc = RegisterPair::from(value),
            R16::AF => self.af = RegisterPair::from(value),
            R16::DE => self.de = RegisterPair::from(value),
            R16::HL => self.hl = RegisterPair::from(value),
        }
    }

    // Stack goodness
    pub fn stack_push16(&mut self, value: u16, memory: &mut GameBoyState) {
        let new_sp = self.sp - 2;
        memory.write_u16(new_sp, value);
        self.sp = new_sp;
    }
    pub fn stack_pop16(&mut self, memory: &mut GameBoyState) -> u16 {
        let value = memory.read_u16(self.sp);
        self.sp += 2;
        value
    }
    pub fn stack_peek16(&self, memory: &GameBoyState) -> u16 {
        let lower = memory.read_u8(self.sp);
        let upper = memory.read_u8(self.sp + 1);
        ((upper as u16) << 8) | (lower as u16)
    }
}

impl From<u16> for RegisterPair {
    fn from(value: u16) -> RegisterPair {
        Self {
            l: (value >> 8) as u8,
            r: (value & 0xFF) as u8,
        }
    }
}

impl From<RegisterPair> for u16 {
    fn from(value: RegisterPair) -> u16 {
        (value.l as u16) << 8 | value.r as u16
    }
}
