use crate::{opcode::{ModeType, INSTR_NAMES, MODES}, parser::Ast};


#[derive(Debug)]
pub struct Generator {
    pub data: Vec<u8>,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            data: Vec::new()
        }
    }

    fn push_number(&mut self, number: u16, mode: ModeType) {
        match mode {
            ModeType::Relative | ModeType::Immediate | ModeType::ZeroPage | ModeType::ZeroPageX | ModeType::ZeroPageY | ModeType::IndirectX | ModeType::IndirectY => {
                self.data.push(number as u8);
                println!(" {:#02x}", self.data[self.data.len()-1]);
            }
            ModeType::Implied => todo!(),
            ModeType::Accumulator => todo!(),
            ModeType::Absolute | ModeType::AbsoluteX | ModeType::AbsoluteY | ModeType::Indirect => {
                self.data.push(number as u8);
                self.data.push((number >> 8) as u8);

                println!(" {:#02x} {:#02x}", self.data[self.data.len()-2], self.data[self.data.len()-1]);
            }
        }
    }

    fn generate_instr(&mut self, instr: usize, number: u16, mode: ModeType) {
        let modes = MODES[instr];
        for search_mode in modes.iter() {
            if search_mode.mode == mode {

                print!("{} ({:#02x})", INSTR_NAMES[instr], search_mode.opcode);
                self.data.push(search_mode.opcode);
                self.push_number(number, mode);
            }
        }
    }

    fn generate_implied(&mut self, instr: usize) {
        let modes = MODES[instr];
        for search_mode in modes.iter() {
            if search_mode.mode == ModeType::Implied {

                println!("{} ({:#02x})", INSTR_NAMES[instr], search_mode.opcode);
                self.data.push(search_mode.opcode);
            }
        }
    }

    pub fn generate(&mut self, asts: Vec<Ast<'_>>) {
        for ast in asts.into_iter() {
            match ast{
                Ast::InstrImplied(instr) => self.generate_implied(instr),
                Ast::InstrBranch(_, _) => todo!(),
                Ast::Instr(inst, number, mode) => self.generate_instr(inst, number, mode),
                Ast::Branch(_) => todo!(),
            }
        }
    }

    pub fn dump(&self) {
        for data in self.data.iter() {
            print!("{:#02x} ", data);
        }
        println!()
    }
}