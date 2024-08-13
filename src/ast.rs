use std::cell::{Cell, RefCell};

use crate::{opcode::{ModeType, BRANCH_INSTS, INSTS_SIZE, JUMP_INSTS}, options::{CompilerOptionEnum, CompilerValue, CompilerValueType, OPTIONS, OPTION_ENUMS, OPTION_MODES}, parser::{Token, TokenInfo}};

#[derive(Debug)]
pub enum Ast<'a> {
    InstrImplied(usize),
    InstrBranch(usize, &'a [u8]),
    InstrJump(usize, &'a [u8]),
    Instr(usize, u16, ModeType),
    Branch(&'a [u8]),
    CompilerOption(CompilerOptionEnum, CompilerValue<'a>)
}

#[derive(Debug)]
pub enum AstGeneratorError {
    SyntaxIssue {
        #[allow(dead_code)] line: usize,
        #[allow(dead_code)] column: usize,
        #[allow(dead_code)] end: usize,
        #[allow(dead_code)] message: &'static str
    },
    OutOfScope
}

impl AstGeneratorError {
    pub fn syntax_issue(token: &TokenInfo, message: &'static str) -> Self {
        AstGeneratorError::SyntaxIssue { column: token.column, end: token.end, line: token.line, message  }
    }
}

#[derive(Debug)]
pub struct AstGenerator<'a> {
    pub tokens: Vec<TokenInfo<'a>>,
    pub asts: RefCell<Vec<Ast<'a>>>,
    pub index: Cell<usize>,
    size: usize,
}

impl<'a> AstGenerator<'a> {
    pub fn new(tokens: Vec<TokenInfo<'a>>) -> Self {
        let size = tokens.len();

        Self {
            tokens,
            asts: RefCell::default(),
            index: Cell::new(0),
            size
        }
    }
    
    fn empty_check(&self) -> Result<(), AstGeneratorError> {
        match self.index.get() >= self.size {
            true => Err(AstGeneratorError::OutOfScope),
            false => Ok(()),
        }
    }

    fn eat<'b>(&'b self)-> Result<&'b TokenInfo<'a>, AstGeneratorError> where 'a: 'b {
        self.empty_check()?;
        self.index.set(self.index.get() + 1);
        Ok(&self.tokens[self.index.get() - 1])
    }

    fn peek<'b>(&'b self)-> Result<&'b TokenInfo<'a>, AstGeneratorError> where 'a: 'b {
        self.empty_check()?;
        Ok(&self.tokens[self.index.get()])
    }

    fn eat_space(&self) -> Result<(), AstGeneratorError> {
        let token = self.eat()?;
        match token.token {
            Token::Space(_) => Ok(()),
            _ => Err(AstGeneratorError::syntax_issue(token, "Expected space"))
        }
    }

    fn cleanup_comment(&self) -> Result<(), AstGeneratorError> {
        let token = self.peek()?;
        if let Token::Comment(_) = token.token {
            let _ = self.eat();
        }
        Ok(())
    }

    fn cleanup_space(&self) -> Result<(), AstGeneratorError> {
        let token = self.peek()?;
        if let Token::Space(_) = token.token {
            let _ = self.eat();
        }
        Ok(())
    }

    fn eat_line_finish(&self) -> Result<(), AstGeneratorError> {
        let _ = self.cleanup_space();
        let _ = self.cleanup_comment();

        let token = self.eat()?;
        match token.token {
            Token::NewLine(_) => Ok(()),
            Token::End => Ok(()),
            _ => Err(AstGeneratorError::syntax_issue(token, "Line not finished properly"))
        }
    }

    fn eat_number(&self) -> Result<(u16, ModeType), AstGeneratorError> {
        let token = self.eat()?;
        match token.token {
            Token::Number(number, mode) => Ok((number, mode)),
            _ => Err(AstGeneratorError::syntax_issue(token, "Expected number"))
        }
    }

    fn eat_string(&self) -> Result<&'a [u8], AstGeneratorError> {
        let token = self.eat()?;
        match token.token {
            Token::String(string) => Ok(string),
            _ => Err(AstGeneratorError::syntax_issue(token, "Expected string"))
        }
    }

    fn eat_text(&self) -> Result<&'a [u8], AstGeneratorError> {
        let token = self.eat()?;
        match token.token {
            Token::Keyword(text) => Ok(text),
            _ => Err(AstGeneratorError::syntax_issue(token, "Expected text"))
        }
    }

    fn generate_compiler_option(&self, token: &TokenInfo, option: &'a [u8]) -> Result<(), AstGeneratorError> {
        if let Some(position) = OPTIONS.iter().position(|item| *item == option) {
            let modes = OPTION_MODES[position];
            let compiler_option_type = OPTION_ENUMS[position];
            let mut found = false;

            self.cleanup_space()?;

            for mode in modes.iter() {

                
                match mode {
                    CompilerValueType::Number => {
                        if let Ok((number, mode)) = self.eat_number() {
                            self.asts.borrow_mut().push(Ast::CompilerOption(compiler_option_type, CompilerValue::Number(number, mode)));
                            found = true;
                        }
                    },
                    CompilerValueType::String => {
                        if let Ok(string) = self.eat_string() {
                            self.asts.borrow_mut().push(Ast::CompilerOption(compiler_option_type, CompilerValue::String(string)));
                            found = true;
                        }
                    },
                }                
            }

            if !found {
                return Err(AstGeneratorError::syntax_issue(token, "Missing information"))
            }
        } else {
            return Err(AstGeneratorError::syntax_issue(token, "Unsupported compiler configuration"))
        }
        self.eat_line_finish()?;
        Ok(())
    }

    fn generate_branch(&self, name: &'a [u8]) -> Result<(), AstGeneratorError> {
        self.asts.borrow_mut().push(Ast::Branch(name));
        self.eat_line_finish()?;
        Ok(())
    }
    
    fn generate_code_block(&self, positon: usize) -> Result<(), AstGeneratorError> {
        if INSTS_SIZE[positon] == 1 {
            self.asts.borrow_mut().push(Ast::InstrImplied(positon));
        }

        else if BRANCH_INSTS.contains(&positon) {
            // Branch inst
            self.eat_space()?;
            self.asts.borrow_mut().push(Ast::InstrBranch(positon, self.eat_text()?));
        }

        else if JUMP_INSTS.contains(&positon) {
            // Jump inst
            self.eat_space()?;
            let next_token = self.eat()?;
            let ast = match next_token.token {
                Token::Keyword(name) => Ast::InstrJump(positon, name),
                Token::Number(number, ModeType::Absolute) => Ast::Instr(positon, number, ModeType::Absolute),
                Token::Number(number, ModeType::Indirect) => Ast::Instr(positon, number, ModeType::Indirect),
                _ => return Err(AstGeneratorError::syntax_issue(next_token, "Branch name, absolute address or indirect address expected")),
            };
            self.asts.borrow_mut().push(ast);
        }

        else {
            self.eat_space()?;
            let (number, mode) = self.eat_number()?;
            self.asts.borrow_mut().push(Ast::Instr(positon, number, mode));
        }
        
        self.eat_line_finish()?;
        Ok(())
    }
    
    pub fn generate(&self) -> Result<(), AstGeneratorError> {
        while self.size > self.index.get() {
            let token = self.eat()?;
            match token.token {
                Token::Instr(positon) => self.generate_code_block(positon)?,
                Token::Keyword(_) => return Err(AstGeneratorError::syntax_issue(token, "Text not expected")),
                Token::CompilerOption(option) => self.generate_compiler_option(&token, option)?,
                Token::Comment(_) => (),
                Token::Branch(name) => self.generate_branch(name)?,
                Token::Number(_, _) => return Err(AstGeneratorError::syntax_issue(token, "Number not expected")),
                Token::NewLine(_) => (),
                Token::Space(_) => (),
                Token::String(_) => return Err(AstGeneratorError::syntax_issue(token, "String not expected")),
                Token::End => {
                    break;
                }
            }
        }

        Ok(())
    }
}