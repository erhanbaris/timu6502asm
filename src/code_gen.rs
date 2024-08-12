use core::str;
use std::{cell::RefCell, collections::HashMap};

use crate::{ast::Ast, opcode::{ModeType, INSTR_NAMES, MODES}};

#[derive(Debug)]
pub enum CodeGeneratorError<'a> {
    NumberNotApplicable,
    UnresolvedBranches(&'a [u8])
}

#[derive(Debug)]
pub struct CodeGenerator<'a> {
    pub data: RefCell<Vec<u8>>,
    pub asts: Vec<Ast<'a>>,
    pub branches: RefCell<HashMap<&'a [u8], usize>>,
    pub unresolved_branches: RefCell<Vec<(&'a [u8], usize)>>
}

impl<'a> CodeGenerator<'a> {
    pub fn new(asts: Vec<Ast<'a>>) -> Self {
        Self {
            data: RefCell::default(),
            branches: Default::default(),
            unresolved_branches: Default::default(),
            asts
        }
    }

    fn push_number(&self, number: u16, mode: ModeType) -> Result<(), CodeGeneratorError<'a>> {
        match mode {
            ModeType::Relative | ModeType::Immediate | ModeType::ZeroPage | ModeType::ZeroPageX | ModeType::ZeroPageY | ModeType::IndirectX | ModeType::IndirectY => {
                self.data.borrow_mut().push(number as u8);
                println!(" {:#02x}", self.data.borrow()[self.data.borrow().len()-1]);
            }
            ModeType::Implied => return Err(CodeGeneratorError::NumberNotApplicable),
            ModeType::Accumulator => return Err(CodeGeneratorError::NumberNotApplicable),
            ModeType::Absolute | ModeType::AbsoluteX | ModeType::AbsoluteY | ModeType::Indirect => {
                self.data.borrow_mut().push(number as u8);
                self.data.borrow_mut().push((number >> 8) as u8);

                println!(" {:#02x} {:#02x}", self.data.borrow()[self.data.borrow().len()-2], self.data.borrow()[self.data.borrow().len()-1]);
            }
        };

        Ok(())
    }

    fn generate_instr(&self, instr: usize, number: u16, mode: ModeType) -> Result<(), CodeGeneratorError<'a>> {
        let modes = MODES[instr];
        for search_mode in modes.iter() {
            if search_mode.mode == mode {

                print!("{} ({:#02x})", INSTR_NAMES[instr], search_mode.opcode);
                self.data.borrow_mut().push(search_mode.opcode);
                self.push_number(number, mode)?;
            }
        }
        Ok(())
    }

    fn generate_instr_branch(&self, position: usize, branch_name: &'a [u8]) -> Result<(), CodeGeneratorError<'a>> {
        let branch_position = match self.branches.borrow().get(branch_name) {
            Some(branch_position) => {
                let distance_position = *branch_position as i8 - (self.data.borrow().len() + 2) as i8;
                distance_position as u16
            },
            None => {
                self.unresolved_branches.borrow_mut().push((branch_name, self.data.borrow().len() + 1));
                0
            }
        };

        let modes = MODES[position];
        self.data.borrow_mut().push(modes[0].opcode);
        print!("{}", INSTR_NAMES[position]);
        self.push_number(branch_position, ModeType::Relative)?;

        Ok(())
    }

    fn generate_implied(&self, position: usize) -> Result<(), CodeGeneratorError<'a>> {
        let modes = MODES[position];
        for search_mode in modes.iter() {
            if search_mode.mode == ModeType::Implied {
                println!("{} ({:#02x})", INSTR_NAMES[position], search_mode.opcode);
                self.data.borrow_mut().push(search_mode.opcode);
                break;
            }
        }
        Ok(())
    }

    fn generate_branch(&self, name: &'a [u8]) -> Result<(), CodeGeneratorError<'a>> {
        println!("{}:", str::from_utf8(&name).unwrap());
        self.branches.borrow_mut().insert(&name, self.data.borrow().len());
        Ok(())
    }

    fn build_unresolved_branches(&self) -> Result<(), CodeGeneratorError<'a>> {
        for (branch_name, position) in self.unresolved_branches.borrow().iter() {
            match self.branches.borrow().get(branch_name) {
                Some(branch_position) => self.data.borrow_mut()[*position] = (*branch_position as i8 - *position as i8 - 1) as u8,
                None => return Err(CodeGeneratorError::UnresolvedBranches(branch_name))
            };
        }

        Ok(())
    }

    pub fn generate(&self) -> Result<(), CodeGeneratorError<'a>> {
        for ast in self.asts.iter() {
            match ast {
                Ast::InstrImplied(position) => self.generate_implied(*position)?,
                Ast::InstrBranch(position, branch) => self.generate_instr_branch(*position, *branch)?,
                Ast::Instr(position, number, mode) => self.generate_instr(*position, *number, *mode)?,
                Ast::Branch(name) => self.generate_branch(*name)?,
            }
        }

        self.build_unresolved_branches()?;
        Ok(())
    }

    pub fn dump(&self) {
        let total_byte_per_row = 16;
        let position = 0x600;
        let mut index = 0;

        print!("{:04x}: ", position);
        for data in self.data.borrow().iter() {
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