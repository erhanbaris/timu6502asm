use strum_macros::{EnumDiscriminants, EnumIter, EnumString};

use crate::opcode::ModeType;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CompilerOptionEnum {
    Org,
    Incbin
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[derive(EnumDiscriminants)]
#[strum_discriminants(name(CompilerValueType))]
pub enum CompilerValue<'a> {
    Number(u16, ModeType),
    String(&'a [u8])
}

impl<'a> CompilerValue<'a> {
    pub fn as_u16(&self) -> u16 {
        match self {
            CompilerValue::Number(number, _) => *number,
            CompilerValue::String(_) => 0
        }
    }
}

pub const OPTIONS: [&[u8]; 2] = [b"ORG", b"INCBIN"];
pub const ORG_TYPES: [CompilerValueType; 1] = [CompilerValueType::Number];
pub const INCBIN_TYPES: [CompilerValueType; 1] = [CompilerValueType::String];

pub const OPTION_MODES: [&[CompilerValueType]; 2] = [&ORG_TYPES, &INCBIN_TYPES];
pub const OPTION_ENUMS: [CompilerOptionEnum; 2] = [CompilerOptionEnum::Org, CompilerOptionEnum::Incbin];
