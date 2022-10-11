use crate::registers::{SmallRegister, WideRegister};

#[derive(Clone, Copy, Debug)]
pub struct InstructionData {
    pub flag_mask: Option<u8>,
    pub flag_expected: Option<u8>,
    pub small_reg_src: Option<SmallRegister>,
    pub small_reg_dst: Option<SmallRegister>,
    pub wide_reg_src: Option<WideRegister>,
    pub wide_reg_dst: Option<WideRegister>,
    pub code: Option<u8>,
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
            code: None,
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

        pub const fn rst_code(mut self, code: u8) -> Self {
        self.code = Some(code);
        self
    }
}
