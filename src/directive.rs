use strum_macros::EnumDiscriminants;

use crate::code_gen::CodeGeneratorError;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DirectiveEnum {
    Org,
    Incbin,
    Byte,
    Word,
    Ascii,
    Asciiz,
    Warning,
    Include
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[derive(EnumDiscriminants)]
#[strum_discriminants(name(DirectiveType))]
pub enum DirectiveValue<'a> {
    Byte(u8),
    Word(u16),
    String(&'a [u8]),
    Reference(&'a [u8]),
}

impl<'a> DirectiveValue<'a> {
    pub fn get_word(&self) -> Result<u16, CodeGeneratorError> {
        
        match self {
            DirectiveValue::Word(number) => Ok(*number),
            _ => Err(CodeGeneratorError::ExpectedThis("Word information"))
        }
    }

    pub fn get_byte(&self) -> Result<u8, CodeGeneratorError> {
        
        match self {
            DirectiveValue::Byte(number) => Ok(*number),
            _ => Err(CodeGeneratorError::ExpectedThis("Byte information"))
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DirectiveVariableSize {
    None,
    Min(usize),
    Length(usize)
}

#[derive(Debug, PartialEq, Clone)]
pub struct DirectiveInfo {
    pub name: &'static [u8],
    pub directive: DirectiveEnum,
    pub size: DirectiveVariableSize,
    pub values: &'static [DirectiveType]
}

pub const SYSTEM_DIRECTIVES: &[DirectiveInfo] = &[
    DirectiveInfo { name: b"BYTE",    directive: DirectiveEnum::Byte,    size: DirectiveVariableSize::Min(1),      values: &[DirectiveType::Byte, DirectiveType::String] },
    DirectiveInfo { name: b"DB",      directive: DirectiveEnum::Byte,    size: DirectiveVariableSize::Min(1),      values: &[DirectiveType::Byte, DirectiveType::String] },
    DirectiveInfo { name: b"WORD",    directive: DirectiveEnum::Word,    size: DirectiveVariableSize::Min(1),      values: &[DirectiveType::Word] },
    DirectiveInfo { name: b"DW",      directive: DirectiveEnum::Word,    size: DirectiveVariableSize::Min(1),      values: &[DirectiveType::Word] },
    DirectiveInfo { name: b"ORG",     directive: DirectiveEnum::Org,     size: DirectiveVariableSize::Length(1),   values: &[DirectiveType::Word] },
    DirectiveInfo { name: b"INCBIN",  directive: DirectiveEnum::Incbin,  size: DirectiveVariableSize::Length(1),   values: &[DirectiveType::String] },
    DirectiveInfo { name: b"ASCII",   directive: DirectiveEnum::Ascii,   size: DirectiveVariableSize::Min(1),      values: &[DirectiveType::String] },
    DirectiveInfo { name: b"ASCIIZ",  directive: DirectiveEnum::Asciiz,  size: DirectiveVariableSize::Min(1),      values: &[DirectiveType::String] },
    DirectiveInfo { name: b"WARNING", directive: DirectiveEnum::Warning, size: DirectiveVariableSize::Length(1),   values: &[DirectiveType::String] },
    DirectiveInfo { name: b"INCLUDE", directive: DirectiveEnum::Include, size: DirectiveVariableSize::Length(1),   values: &[DirectiveType::String] },
];