use crate::registers::SmallRegister;

#[derive(Clone, Copy)]
pub struct InstructionData {
    pub flag_mask: u8,
    pub flag_expected: u8,
    pub small_reg_dst: SmallRegister,
}

impl InstructionData {
    pub const fn const_default() -> Self {
        Self {
            flag_mask: 0,
            flag_expected: 0,
            small_reg_dst: SmallRegister::Unset,
        }
    }

    pub const fn small_dst(target: SmallRegister) -> Self {
        let mut data = InstructionData::const_default();
        data.small_reg_dst = target;
        data
    }

    pub const fn with_flags(flag_mask: u8, flag_exptected: u8) -> Self {
        let mut data = InstructionData::const_default();
        data.flag_mask = flag_mask;
        data.flag_expected = flag_exptected;
        data
    }
}
