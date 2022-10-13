use crate::registers::{R16, R8};

#[derive(Clone, Copy, Debug)]
pub struct InstructionData {
    pub flag_mask: Option<u8>,
    pub flag_expected: Option<u8>,
    pub r8_src: Option<R8>,
    pub r8_dst: Option<R8>,
    pub r16_src: Option<R16>,
    pub r16_dst: Option<R16>,
    pub code: Option<u8>,
    pub bit: Option<u8>,
}

impl InstructionData {
    pub const fn new() -> Self {
        Self {
            flag_mask: None,
            flag_expected: None,
            r8_src: None,
            r8_dst: None,
            r16_src: None,
            r16_dst: None,
            code: None,
            bit: None,
        }
    }
    pub const fn r8_src(mut self, src: R8) -> Self {
        self.r8_src = Some(src);
        self
    }

    pub const fn r8_dst(mut self, dst: R8) -> Self {
        self.r8_dst = Some(dst);
        self
    }

    pub const fn r16_src(mut self, src: R16) -> Self {
        self.r16_src = Some(src);
        self
    }

    pub const fn r16_dst(mut self, dst: R16) -> Self {
        self.r16_dst = Some(dst);
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

    pub const fn bit(mut self, bit: u8) -> Self {
        self.bit = Some(bit);
        self
    }
}
