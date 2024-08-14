use strum_macros::EnumDiscriminants;

use crate::opcode::ModeType;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DirectiveEnum {
    Org,
    Incbin,
    Byte
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[derive(EnumDiscriminants)]
#[strum_discriminants(name(DirectiveType))]
pub enum DirectiveValue<'a> {
    Number(u16, ModeType),
    String(&'a [u8])
}

impl<'a> DirectiveValue<'a> {
    pub fn as_u16(&self) -> u16 {
        match self {
            DirectiveValue::Number(number, _) => *number,
            DirectiveValue::String(_) => 0
        }
    }
}

pub const OPTIONS: [&[u8]; 3] = [b"ORG", b"INCBIN", b"BYTE"];
pub const ORG_TYPES: [DirectiveType; 1] = [DirectiveType::Number];
pub const INCBIN_TYPES: [DirectiveType; 1] = [DirectiveType::String];
pub const BYTE_TYPES: [DirectiveType; 2] = [DirectiveType::String, DirectiveType::Number];

pub const OPTION_MODES: [&[DirectiveType]; 3] = [&ORG_TYPES, &INCBIN_TYPES, &BYTE_TYPES];
pub const DIRECTIVE_ENUMS: [DirectiveEnum; 3] = [DirectiveEnum::Org, DirectiveEnum::Incbin, DirectiveEnum::Byte];
