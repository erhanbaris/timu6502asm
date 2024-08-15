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
    Warning
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

pub const OPTIONS: [&[u8]; 7] = [b"ORG", b"INCBIN", b"BYTE", b"WORD", b"ASCII", b"ASCIIZ", b"WARNING"];
pub const ORG_TYPES: [DirectiveType; 1] = [DirectiveType::Word];
pub const INCBIN_TYPES: [DirectiveType; 1] = [DirectiveType::String];
pub const BYTE_TYPES: [DirectiveType; 2] = [DirectiveType::Byte, DirectiveType::String];
pub const WORD_TYPES: [DirectiveType; 1] = [DirectiveType::Word];
pub const ASCII_TYPES: [DirectiveType; 1] = [DirectiveType::String];
pub const ASCIIZ_TYPES: [DirectiveType; 1] = [DirectiveType::String];
pub const WARNING_TYPES: [DirectiveType; 1] = [DirectiveType::String];

pub const OPTION_MODES: [&[DirectiveType]; 7] = [&ORG_TYPES, &INCBIN_TYPES, &BYTE_TYPES, &WORD_TYPES, &ASCII_TYPES, &ASCIIZ_TYPES, &WARNING_TYPES];
pub const DIRECTIVE_ENUMS: [DirectiveEnum; 7] = [DirectiveEnum::Org, DirectiveEnum::Incbin, DirectiveEnum::Byte, DirectiveEnum::Word, DirectiveEnum::Ascii, DirectiveEnum::Asciiz, DirectiveEnum::Warning];
