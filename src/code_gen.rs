use std::{collections::HashMap, str::Utf8Error};
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use thiserror::Error;
use strum_macros::EnumDiscriminants;

use crate::context::Context;
use crate::tool::print_error;
use crate::{ast::{Ast, BranchType}, opcode::{ModeType, MODES}, options::{CompilerOptionEnum, CompilerValue}};

#[derive(Error, Debug)]
pub enum CodeGeneratorError {
    #[error("Internal error")]
    InternalError,
    #[error("Number not applicable")]
    NumberNotApplicable,
    #[error("Branch information not found")]
    UnresolvedBranches,
    #[error("Reference information not found")]
    UnresolvedReference,
    #[error("Invalid reference value")]
    InvalidReferenceValue,
    #[error("Expected string")]
    StringExpected,
    #[error("IO Error ({0})")]
    IOError(#[from] std::io::Error),
    #[error("Text convertion issue ({0})")]
    Utf8Error(#[from] Utf8Error)
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[derive(EnumDiscriminants)]
#[strum_discriminants(name(ReferenceType))]
pub enum ReferenceValue {
    AbsoluteAddress(u16),
    RelativeAddress(u16),
    Value(u16, ModeType),
}

#[derive(Debug)]
pub struct CodeGenerator<'a> {
    pub index: usize,
    pub size: usize,

    pub start_point: u16,
    pub branches: HashMap<&'a [u8], usize>,
    pub references: HashMap<&'a [u8], ReferenceValue>,
    pub unresolved_branches: Vec<(&'a [u8], usize, usize)>,
    pub unresolved_jumps: Vec<(&'a [u8], usize, usize)>
}

impl<'a> CodeGenerator<'a> {
    pub fn new() -> Self {
        Self {
            index: 0,
            size: 0,
            start_point: Default::default(),
            branches: Default::default(),
            unresolved_branches: Default::default(),
            unresolved_jumps: Default::default(),
            references: Default::default()
        }
    }

    fn empty_check(&self) -> Result<(), CodeGeneratorError> {
        match self.index >= self.size {
            true => Err(CodeGeneratorError::InternalError),
            false => Ok(()),
        }
    }

    fn eat(&mut self)-> Result<usize, CodeGeneratorError> {
        self.empty_check()?;
        self.index += 1;
        Ok(self.index - 1)
    }

    fn push_number(&mut self, target: &mut Vec<u8>, number: u16, mode: ModeType) -> Result<(), CodeGeneratorError> {
        match mode {
            ModeType::Relative | ModeType::Immediate | ModeType::ZeroPage | ModeType::ZeroPageX | ModeType::ZeroPageY | ModeType::IndirectX | ModeType::IndirectY => {
                target.push(number as u8);
            }
            ModeType::Implied => return Err(CodeGeneratorError::NumberNotApplicable),
            ModeType::Accumulator => return Err(CodeGeneratorError::NumberNotApplicable),
            ModeType::Absolute | ModeType::AbsoluteX | ModeType::AbsoluteY | ModeType::Indirect => {
                target.push(number as u8);
                target.push((number >> 8) as u8);
            }
        };

        Ok(())
    }

    fn generate_instr(&mut self, target: &mut Vec<u8>, instr: usize, number: u16, mode: ModeType) -> Result<(), CodeGeneratorError> {
        let modes = MODES[instr];
        for search_mode in modes.iter() {
            if search_mode.mode == mode {
                target.push(search_mode.opcode);
                self.push_number(target, number, mode)?;
            }
        }
        Ok(())
    }

    fn generate_instr_reference(&mut self, target: &mut Vec<u8>, instr: usize, reference: &'a [u8]) -> Result<(), CodeGeneratorError> {
        let modes = MODES[instr];
        let value = match self.references.get(reference) {
            Some(value) => value,
            None=> return Err(CodeGeneratorError::UnresolvedReference)
        };

        let (number, mode) = match value {
            ReferenceValue::AbsoluteAddress(_) => return Err(CodeGeneratorError::InvalidReferenceValue),
            ReferenceValue::RelativeAddress(_) => return Err(CodeGeneratorError::InvalidReferenceValue),
            ReferenceValue::Value(number, mode) => (*number, *mode),
        };

        for search_mode in modes.iter() {
            if search_mode.mode == mode {
                target.push(search_mode.opcode);
                self.push_number(target, number, mode)?;
            }
        }
        
        Ok(())
    }

    fn generate_instr_branch(&mut self, target: &mut Vec<u8>, ast_index: usize, position: usize, branch_name: &'a [u8]) -> Result<(), CodeGeneratorError> {
        let branch_position = match self.branches.get(branch_name) {
            Some(branch_position) => {
                let distance_position = *branch_position as i8 - (target.len() + 2) as i8;
                distance_position as u16
            },
            None => {
                self.unresolved_branches.push((branch_name, target.len() + 1, ast_index));
                0
            }
        };

        let modes = MODES[position];
        target.push(modes[0].opcode);
        self.push_number(target, branch_position, ModeType::Relative)?;

        Ok(())
    }

    fn generate_instr_jump(&mut self, target: &mut Vec<u8>, ast_index: usize, position: usize, branch_name: &'a [u8]) -> Result<(), CodeGeneratorError> {
        let jump_position = match self.branches.get(branch_name) {
            Some(jump_position) => self.start_point + *jump_position as u16,
            None => {
                self.unresolved_jumps.push((branch_name, target.len() + 1, ast_index));
                0
            }
        };

        let modes = MODES[position];
        target.push(modes[0].opcode);
        self.push_number(target, jump_position, ModeType::Absolute)?;
        Ok(())
    }

    fn configure_assign(&mut self, name: &'a [u8], number: u16, mode: ModeType) -> Result<(), CodeGeneratorError> {
        self.references.insert(name, ReferenceValue::Value(number, mode));
        Ok(())
    }

    fn generate_implied(&mut self, target: &mut Vec<u8>, position: usize) -> Result<(), CodeGeneratorError> {
        let modes = MODES[position];
        for search_mode in modes.iter() {
            if search_mode.mode == ModeType::Implied {
                target.push(search_mode.opcode);
                break;
            }
        }
        Ok(())
    }

    fn generate_branch(&mut self, target: &mut Vec<u8>, name: &'a [u8], branch_type: BranchType) -> Result<(), CodeGeneratorError> {
        self.branches.insert(name, target.len());
        self.references.insert(name, ReferenceValue::AbsoluteAddress(0));
        Ok(())
    }

    fn build_unresolved_branches(&mut self, target: &mut Vec<u8>) -> Result<(), CodeGeneratorError> {
        for (branch_name, position, _) in self.unresolved_branches.iter() {
            match self.branches.get(branch_name) {
                Some(branch_position) => target[*position] = (*branch_position as i8 - *position as i8 - 1) as u8,
                None => return Err(CodeGeneratorError::UnresolvedBranches)
            };
        }

        Ok(())
    }

    fn build_unresolved_jumps(&mut self, target: &mut Vec<u8>) -> Result<(), CodeGeneratorError> {
        for (branch_name, position, _) in self.unresolved_jumps.iter() {
            match self.branches.get(branch_name) {
                Some(branch_position) => {
                    let jump_position = self.start_point + *branch_position as u16;

                    target[*position] = jump_position as u8;
                    target[*position + 1] = (jump_position >> 8) as u8;
                }
                None => return Err(CodeGeneratorError::UnresolvedBranches)
            };
        }

        Ok(())
    }

    fn configure_compiler(&mut self, target: &mut Vec<u8>, option: CompilerOptionEnum, value: CompilerValue<'a>) -> Result<(), CodeGeneratorError> {
        match option {
            CompilerOptionEnum::Org => self.start_point = value.as_u16(),
            CompilerOptionEnum::Incbin => {

                let file_path = match value {
                    CompilerValue::String(name) => name,
                    _ => return Err(CodeGeneratorError::StringExpected)
                };

                let file_path = match std::str::from_utf8(file_path) {
                    Ok(file_path) => file_path,
                    Err(error) => return Err(CodeGeneratorError::Utf8Error(error))
                };
                
                let file = match File::open(file_path) {
                    Ok(file) => file,
                    Err(error) => return Err(CodeGeneratorError::IOError(error))
                };

                let buffer_reader: BufReader<File> = BufReader::new(file);
                for buffer in buffer_reader.bytes() {
                    match buffer {
                        Ok(byte) => target.push(byte),
                        Err(error) => return Err(CodeGeneratorError::IOError(error))
                    }
                }
            },
        };
        Ok(())
    }

    fn inner_generate(&mut self, context: &mut Context<'a>) -> Result<(), CodeGeneratorError> {
        self.size = context.asts.borrow().len();
        let asts = context.asts.borrow();
        
        while self.size > self.index {
            let ast_index = self.eat()?;
            let ast = asts.get(ast_index).map(|item| &item.ast);

            match ast {
                Some(Ast::InstrImplied(position)) => self.generate_implied(&mut context.target, *position)?,
                Some(Ast::InstrBranch(position, branch)) => self.generate_instr_branch(&mut context.target, ast_index, *position, *branch)?,
                Some(Ast::InstrJump(position, branch)) => self.generate_instr_jump(&mut context.target, ast_index, *position, *branch)?,
                Some(Ast::Instr(position, number, mode)) => self.generate_instr(&mut context.target, *position, *number, *mode)?,
                Some(Ast::InstrRef(position, reference)) => self.generate_instr_reference(&mut context.target, *position, *reference)?,
                Some(Ast::Branch(name, branch_type)) => self.generate_branch(&mut context.target, name, *branch_type)?,
                Some(Ast::CompilerOption(option, value)) => self.configure_compiler(&mut context.target, *option, *value)?,
                Some(Ast::Assign(name, number, mode)) => self.configure_assign(*name, *number, *mode)?,
                None => return Err(CodeGeneratorError::InternalError)
            };
        }

        self.build_unresolved_branches(&mut context.target)?;
        self.build_unresolved_jumps(&mut context.target)?;
        Ok(())
    }

    pub fn generate(&mut self, context: Context<'a>) -> Result<Context<'a>, CodeGeneratorError> {
        let mut context = context;
        
        match self.inner_generate(&mut context) {
            Ok(_) => Ok(context),
            Err(error) => {
                let asts = context.asts.borrow();
                let ast = &asts[self.index - 1];
                print_error(&context.source, &error, ast.line, ast.column, ast.end);
                Err(error)
            }
        }
    }

    pub fn dump(&self, context: &Context<'a>) {
        let total_byte_per_row = 16;
        let position = self.start_point;
        let mut index = 0;

        print!("{:04x}: ", position);
        for data in context.target.iter() {
            print!("{:02x} ", data);
            index += 1;
            
            if index % total_byte_per_row == 0 {
                println!();
                print!("{:04x}: ", position + index);
        
            }
        }
        println!()
    }
}