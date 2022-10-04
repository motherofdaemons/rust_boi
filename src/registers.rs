use std::fmt::Error;

#[derive(Default)]
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

#[derive(Clone, Copy, Debug)]
pub enum SmallRegister {
    B,
    C,
    A,
    F,
    D,
    E,
    H,
    L,
    Unset,
}

#[derive(Clone, Copy, Debug)]
pub enum WideRegister {
    PC,
    SP,
    BC,
    AF,
    DE,
    HL,
    Unset,
}

pub const ZERO_FLAG: u8 = 0x80;
pub const SUBTRACT_FLAG: u8 = 0x40;
pub const HALF_CARRY_FLAG: u8 = 0x20;
pub const CARRY_FLAG: u8 = 0x10;

impl Registers {
    pub fn get_pc(&self) -> u16 {
        self.read_r16(WideRegister::PC)
    }
    pub fn inc_pc(&mut self, by: u16) {
        self.write_r16(WideRegister::PC, self.get_pc() + by);
    }
    pub fn set_pc(&mut self, address: u16) {
        self.write_r16(WideRegister::PC, address)
    }
    pub fn get_sp(&self) -> u16 {
        self.read_r16(WideRegister::SP)
    }
    pub fn inc_sp(&mut self, by: u16) {
        self.write_r16(WideRegister::SP, self.get_sp() + by);
    }
    pub fn get_flags(&self) -> u8 {
        self.read_r8(SmallRegister::F)
    }
    pub fn set_flags(
        &mut self,
        zero: Option<bool>,
        subtract: Option<bool>,
        half_carry: Option<bool>,
        carry: Option<bool>,
    ) {
        let mut flags = self.read_r8(SmallRegister::F);
        if zero.is_some() {
            flags = Registers::set_bit_flag(flags, ZERO_FLAG, zero.unwrap());
        }
        if subtract.is_some() {
            flags = Registers::set_bit_flag(flags, SUBTRACT_FLAG, subtract.unwrap());
        }
        if half_carry.is_some() {
            flags = Registers::set_bit_flag(flags, HALF_CARRY_FLAG, half_carry.unwrap());
        }
        if carry.is_some() {
            flags = Registers::set_bit_flag(flags, CARRY_FLAG, carry.unwrap());
        }
        self.write_r8(SmallRegister::F, flags);
    }
    pub fn set_bit_flag(flags: u8, bit: u8, set: bool) -> u8 {
        match set {
            true => flags | bit,
            false => flags & !bit,
        }
    }

    pub fn read_r8(&self, reg: SmallRegister) -> u8 {
        match reg {
            SmallRegister::B => self.bc.l,
            SmallRegister::C => self.bc.r,
            SmallRegister::A => self.af.l,
            SmallRegister::F => self.af.r,
            SmallRegister::D => self.de.l,
            SmallRegister::E => self.de.r,
            SmallRegister::H => self.hl.l,
            SmallRegister::L => self.hl.r,
            SmallRegister::Unset => panic!("We should never have an unset register!"),
        }
    }

    pub fn write_r8(&mut self, reg: SmallRegister, val: u8) {
        match reg {
            SmallRegister::B => self.bc.l = val,
            SmallRegister::C => self.bc.r = val,
            SmallRegister::A => self.af.l = val,
            SmallRegister::F => self.af.r = val,
            SmallRegister::D => self.de.l = val,
            SmallRegister::E => self.de.r = val,
            SmallRegister::H => self.hl.l = val,
            SmallRegister::L => self.hl.r = val,
            SmallRegister::Unset => panic!("We should never have an unset register!"),
        }
    }

    pub fn read_r16(&self, reg: WideRegister) -> u16 {
        match reg {
            WideRegister::PC => self.pc,
            WideRegister::SP => self.sp,
            WideRegister::BC => self.bc.into(),
            WideRegister::AF => self.af.into(),
            WideRegister::DE => self.de.into(),
            WideRegister::HL => self.hl.into(),
            WideRegister::Unset => panic!("Wide register should never be unset"),
        }
    }

    pub fn write_r16(&mut self, reg: WideRegister, val: u16) {
        match reg {
            WideRegister::PC => self.pc = val,
            WideRegister::SP => self.sp = val,
            WideRegister::BC => self.bc = RegisterPair::from(val),
            WideRegister::AF => self.af = RegisterPair::from(val),
            WideRegister::DE => self.de = RegisterPair::from(val),
            WideRegister::HL => self.hl = RegisterPair::from(val),
            _ => panic!("Unkown write_r16 target {:?}", reg),
        }
    }

}

impl Into<u16> for RegisterPair {
    fn into(self) -> u16 {
        (self.l as u16) << 8 | self.r as u16
    }
}

impl From<u16> for RegisterPair {
    fn from(val: u16) -> Self {
        Self {
            l: (val >> 8) as u8,
            r: (val & 0xFF) as u8,
        }
    }
}
