use crate::registers::{SmallRegister, WideRegister};

#[derive(Clone, Copy)]
pub struct InstructionData {
    pub flag_mask: Option<u8>,
    pub flag_expected: Option<u8>,
    pub small_reg_src: Option<SmallRegister>,
    pub small_reg_dst: Option<SmallRegister>,
    pub wide_reg_src: Option<WideRegister>,
    pub wide_reg_dst: Option<WideRegister>,
    pub immediate_16: bool,
    pub immediate_8: bool,
}

impl InstructionData {
    pub const fn new() -> Self {
        Self {
            flag_mask: None,
            flag_expected: None,
            small_reg_src: None,
            small_reg_dst: None,
            wide_reg_src: None,
            wide_reg_dst: None,
            immediate_16: false,
            immediate_8: false,
        }
    }
    pub const fn small_src(mut self, src: SmallRegister) -> Self {
        self.small_reg_src = Some(src);
        self
    }

    pub const fn small_dst(mut self, dst: SmallRegister) -> Self {
        self.small_reg_dst = Some(dst);
        self
    }

    pub const fn wide_src(mut self, src: WideRegister) -> Self {
        self.wide_reg_src = Some(src);
        self
    }

    pub const fn wide_dst(mut self, dst: WideRegister) -> Self {
        self.wide_reg_dst = Some(dst);
        self
    }

    pub const fn with_flags(mut self, flag_mask: u8, flag_exptected: u8) -> Self {
        self.flag_mask = Some(flag_mask);
        self.flag_expected = Some(flag_exptected);
        self
    }

    pub const fn immediate_16(mut self) -> Self {
        self.immediate_16 = true;
        self
    }

    pub const fn immediate_8(mut self) -> Self {
        self.immediate_8 = true;
        self
    }
}
