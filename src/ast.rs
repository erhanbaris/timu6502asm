use std::{cell::{Cell, RefCell}};

use crate::{opcode::{ModeType, BRANCH_INSTS, INSTS_SIZE}, parser::{Token, TokenInfo}};

#[derive(Debug)]
pub enum Ast<'a> {
    InstrImplied(usize),
    InstrBranch(usize, &'a [u8]),
    Instr(usize, u16, ModeType),
    Branch(&'a [u8])
}

#[derive(Debug)]
pub enum AstGeneratorError {
    SyntaxIssue {
        line: usize,
        column: usize,
        end: usize,
        message: &'static str
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

    fn eat_comment(&self) -> Result<(), AstGeneratorError> {
        let token = self.eat()?;
        match token.token {
            Token::Comment(_) => Ok(()),
            _ => Err(AstGeneratorError::syntax_issue(token, "Expected comment"))
        }
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
            _ => Err(AstGeneratorError::syntax_issue(token, "Expected comment"))
        }
    }

    fn eat_text(&self) -> Result<&'a [u8], AstGeneratorError> {
        let token = self.eat()?;
        match token.token {
            Token::Text(text) => Ok(text),
            _ => Err(AstGeneratorError::syntax_issue(token, "Expected text"))
        }
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
                Token::Text(_) => return Err(AstGeneratorError::syntax_issue(token, "Text not expected")),
                Token::Comment(_) => (),
                Token::Branch(name) => self.generate_branch(name)?,
                Token::Number(_, _) => return Err(AstGeneratorError::syntax_issue(token, "Number not expected")),
                Token::NewLine(_) => (),
                Token::Space(_) => (),
                Token::End => {
                    break;
                }
            }
        }

        Ok(())
    }
}