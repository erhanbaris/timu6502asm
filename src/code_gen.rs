use std::{cell::{Cell, RefCell}, collections::HashMap};

use crate::{ast::Ast, opcode::{ModeType, MODES}, options::{CompilerOptionEnum, CompilerValue}};

#[derive(Debug)]
pub enum CodeGeneratorError {
    NumberNotApplicable,
    UnresolvedBranches
}

#[derive(Debug)]
pub struct CodeGenerator<'a> {
    pub start_point: Cell<u16>,
    pub data: RefCell<Vec<u8>>,
    pub asts: Vec<Ast<'a>>,
    pub branches: RefCell<HashMap<&'a [u8], usize>>,
    pub unresolved_branches: RefCell<Vec<(&'a [u8], usize)>>,
    pub unresolved_jumps: RefCell<Vec<(&'a [u8], usize)>>
}

impl<'a> CodeGenerator<'a> {
    pub fn new(asts: Vec<Ast<'a>>) -> Self {
        Self {
            start_point: Cell::new(0),
            data: RefCell::default(),
            branches: Default::default(),
            unresolved_branches: Default::default(),
            unresolved_jumps: Default::default(),
            asts
        }
    }

    fn push_number(&self, number: u16, mode: ModeType) -> Result<(), CodeGeneratorError> {
        match mode {
            ModeType::Relative | ModeType::Immediate | ModeType::ZeroPage | ModeType::ZeroPageX | ModeType::ZeroPageY | ModeType::IndirectX | ModeType::IndirectY => {
                self.data.borrow_mut().push(number as u8);
            }
            ModeType::Implied => return Err(CodeGeneratorError::NumberNotApplicable),
            ModeType::Accumulator => return Err(CodeGeneratorError::NumberNotApplicable),
            ModeType::Absolute | ModeType::AbsoluteX | ModeType::AbsoluteY | ModeType::Indirect => {
                self.data.borrow_mut().push(number as u8);
                self.data.borrow_mut().push((number >> 8) as u8);
            }
        };

        Ok(())
    }

    fn generate_instr(&self, instr: usize, number: u16, mode: ModeType) -> Result<(), CodeGeneratorError> {
        let modes = MODES[instr];
        for search_mode in modes.iter() {
            if search_mode.mode == mode {
                self.data.borrow_mut().push(search_mode.opcode);
                self.push_number(number, mode)?;
            }
        }
        Ok(())
    }

    fn generate_instr_branch(&self, position: usize, branch_name: &'a [u8]) -> Result<(), CodeGeneratorError> {
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
        self.push_number(branch_position, ModeType::Relative)?;

        Ok(())
    }

    fn generate_instr_jump(&self, position: usize, branch_name: &'a [u8]) -> Result<(), CodeGeneratorError> {
        let jump_position = match self.branches.borrow().get(branch_name) {
            Some(jump_position) => self.start_point.get() + *jump_position as u16,
            None => {
                self.unresolved_jumps.borrow_mut().push((branch_name, self.data.borrow().len() + 1));
                0
            }
        };

        let modes = MODES[position];
        self.data.borrow_mut().push(modes[0].opcode);
        self.push_number(jump_position, ModeType::Absolute)?;

        Ok(())
    }

    fn generate_implied(&self, position: usize) -> Result<(), CodeGeneratorError> {
        let modes = MODES[position];
        for search_mode in modes.iter() {
            if search_mode.mode == ModeType::Implied {
                self.data.borrow_mut().push(search_mode.opcode);
                break;
            }
        }
        Ok(())
    }

    fn generate_branch(&self, name: &'a [u8]) -> Result<(), CodeGeneratorError> {
        self.branches.borrow_mut().insert(name, self.data.borrow().len());
        Ok(())
    }

    fn build_unresolved_branches(&self) -> Result<(), CodeGeneratorError> {
        for (branch_name, position) in self.unresolved_branches.borrow().iter() {
            match self.branches.borrow().get(branch_name) {
                Some(branch_position) => self.data.borrow_mut()[*position] = (*branch_position as i8 - *position as i8 - 1) as u8,
                None => return Err(CodeGeneratorError::UnresolvedBranches)
            };
        }

        Ok(())
    }

    fn build_unresolved_jumps(&self) -> Result<(), CodeGeneratorError> {
        for (branch_name, position) in self.unresolved_jumps.borrow().iter() {
            match self.branches.borrow().get(branch_name) {
                Some(branch_position) => {
                    let jump_position = self.start_point.get() + *branch_position as u16;

                    self.data.borrow_mut()[*position] = jump_position as u8;
                    self.data.borrow_mut()[*position + 1] = (jump_position >> 8) as u8;
                }
                None => return Err(CodeGeneratorError::UnresolvedBranches)
            };
        }

        Ok(())
    }

    fn configure_compiler(&self, option: CompilerOptionEnum, value: CompilerValue<'a>) -> Result<(), CodeGeneratorError> {
        match option {
            CompilerOptionEnum::Org => self.start_point.set(value.as_u16()),
            CompilerOptionEnum::Incbin => todo!(),
        };
        Ok(())
    }

    pub fn generate(&self) -> Result<(), CodeGeneratorError> {
        for ast in self.asts.iter() {
            match ast {
                Ast::InstrImplied(position) => self.generate_implied(*position)?,
                Ast::InstrBranch(position, branch) => self.generate_instr_branch(*position, branch)?,
                Ast::InstrJump(position, branch) => self.generate_instr_jump(*position, branch)?,
                Ast::Instr(position, number, mode) => self.generate_instr(*position, *number, *mode)?,
                Ast::Branch(name) => self.generate_branch(name)?,
                Ast::CompilerOption(option, value) => self.configure_compiler(*option, *value)?,
            }
        }

        self.build_unresolved_branches()?;
        self.build_unresolved_jumps()?;
        Ok(())
    }

    pub fn dump(&self) {
        let total_byte_per_row = 16;
        let position = self.start_point.get();
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