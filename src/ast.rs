use std::{cell::Cell, marker::PhantomData};

use crate::{context::Context, opcode::{ModeType, BRANCH_INSTS, INSTS_SIZE, JUMP_INSTS}, options::{DirectiveEnum, DirectiveType, DirectiveValue, DIRECTIVE_ENUMS, OPTIONS, OPTION_MODES}, parser::{Token, TokenInfo, TokenType}, tool::{print_error, upper_case}};

#[derive(Debug, Copy, Clone)]
pub enum BranchType {
    Generic,
    Next
}

#[derive(Debug)]
pub enum Ast<'a> {
    InstrImplied(usize),
    InstrBranch(usize, &'a [u8]),
    InstrJump(usize, &'a [u8]),
    Instr(usize, u16, ModeType),
    InstrRef(usize, &'a [u8]),
    Branch(&'a [u8], BranchType),
    Directive(DirectiveEnum, DirectiveValue<'a>),
    Assign(&'a [u8], u16, ModeType)
}

#[derive(Debug)]
pub struct AstInfo<'a> {
    pub line: usize,
    pub column: usize,
    pub ast: Ast<'a>,
    pub end: usize,
}

impl<'a> AstInfo<'a> {
    pub fn new(token: &'a TokenInfo<'a>, ast: Ast<'a>) -> Self {
        Self {
            line: token.line,
            column: token.column,
            end: token.end,
            ast
        }
    }
}

#[derive(Debug)]
pub enum AstGeneratorError {
    SyntaxIssue {
        #[allow(dead_code)] line: usize,
        #[allow(dead_code)] column: usize,
        #[allow(dead_code)] end: usize,
        #[allow(dead_code)] message: &'static str
    },
    OutOfScope,
    InternalError
}

impl AstGeneratorError {
    pub fn syntax_issue<'a>(context: &Context<'a>, token_index: usize, message: &'static str) -> Self {
        let token_info = &context.tokens.borrow()[token_index];
        AstGeneratorError::SyntaxIssue { column: token_info.column, end: token_info.end, line: token_info.line, message  }
    }
}

#[derive(Debug)]
pub struct AstGenerator<'a> {
    pub index: Cell<usize>,
    pub size: Cell<usize>,
    marker: PhantomData<&'a u8>
}

impl<'a> AstGenerator<'a> {
    pub fn new() -> Self {
        Self {
            index: Cell::new(0),
            size: Cell::new(0),
            marker: Default::default()
        }
    }
    
    fn empty_check(&self) -> Result<(), AstGeneratorError> {
        match self.index.get() >= self.size.get() {
            true => Err(AstGeneratorError::OutOfScope),
            false => Ok(()),
        }
    }

    fn eat(&self) -> Result<usize, AstGeneratorError> {
        self.empty_check()?;
        self.index.set(self.index.get() + 1);
        Ok(self.index.get() - 1)
    }

    fn dec(&self) -> Result<(), AstGeneratorError> {
        self.empty_check()?;
        self.index.set(self.index.get() - 1);
        Ok(())
    }

    fn peek(&self)-> Result<usize, AstGeneratorError> {
        self.empty_check()?;
        Ok(self.index.get())
    }

    fn eat_space(&self, context: &Context<'a>) -> Result<(), AstGeneratorError> {
        let token_index= self.eat()?;
        let token = &context.tokens.borrow()[token_index];
        match token.token {
            Token::Space(_) => Ok(()),
            _ => Err(AstGeneratorError::syntax_issue(context, token_index, "Expected space"))
        }
    }

    fn cleanup_space(&self, context: &Context<'a>) -> Result<(), AstGeneratorError> {
        let token_index = self.peek()?;
        let token = &context.tokens.borrow()[token_index];
        if let Token::Space(_) = token.token {
            let _ = self.eat();
        }
        Ok(())
    }

    fn eat_if(&self, context: &Context<'a>, expected: TokenType) -> Option<usize> {
        let token_index = match self.peek() {
            Ok(token_index) => token_index,
            Err(_) => return None
        };

        let token = &context.tokens.borrow()[token_index];
        let token_type: TokenType = TokenType::from(token.token);
        
        match token_type == expected {
            true => {
                self.index.set(self.index.get() + 1);
                Some(token_index)
            }
            false => None
        }
    }

    fn eat_if_string(&self, context: &Context<'a>) -> Option<&'a [u8]> {
        let index = self.eat_if(context, TokenType::String)?;
        let token = &context.tokens.borrow()[index];
        match token.token {
            Token::String(string) => Some(string),
            _ => None
        }
    }

    fn eat_if_number(&self, context: &Context<'a>) -> Option<(u16, ModeType)> {
        let index = self.eat_if(context, TokenType::Number)?;
        let token = &context.tokens.borrow()[index];
        match token.token {
            Token::Number(number, mode) => Some((number, mode)),
            _ => None
        }
    }

    fn eat_number(&self, context: &Context<'a>) -> Result<(u16, ModeType), AstGeneratorError> {
        let token_index= self.eat()?;
        let token = &context.tokens.borrow()[token_index];
        match token.token {
            Token::Number(number, mode) => Ok((number, mode)),
            _ => Err(AstGeneratorError::syntax_issue(context, token_index, "Expected number"))
        }
    }
    
    fn eat_string(&self, context: &Context<'a>) -> Result<&'a [u8], AstGeneratorError> {
        let token_index= self.eat()?;
        let token = &context.tokens.borrow()[token_index];
        match token.token {
            Token::String(string) => Ok(string),
            _ => Err(AstGeneratorError::syntax_issue(context, token_index, "Expected string"))
        }
    }

    fn eat_assign(&self, context: &Context<'a>) -> Result<(), AstGeneratorError> {
        let token_index= self.eat()?;
        let token = &context.tokens.borrow()[token_index];
        match token.token {
            Token::Assign => Ok(()),
            _ => Err(AstGeneratorError::syntax_issue(context, token_index, "Expected assign"))
        }
    }

    fn eat_text(&self, context: &Context<'a>) -> Result<&'a [u8], AstGeneratorError> {
        let token_index= self.eat()?;
        let token = &context.tokens.borrow()[token_index];
        match token.token {
            Token::Keyword(text) => Ok(text),
            _ => Err(AstGeneratorError::syntax_issue(context, token_index, "Expected text"))
        }
    }

    fn generate_directive(&self, context: &Context<'a>, token_index: usize, option: &'a [u8]) -> Result<(), AstGeneratorError> {
        let option = upper_case(option);
        if let Some(position) = OPTIONS.iter().position(|item| *item == &option[..]) {
            let modes = OPTION_MODES[position];
            let directive_type = DIRECTIVE_ENUMS[position];
            let mut found = false;

            self.cleanup_space(context)?;

            for mode in modes.iter() {
                match mode {
                    DirectiveType::Number => {
                        if let Some((number, mode)) = self.eat_if_number(context) {
                            context.add_ast(token_index, Ast::Directive(directive_type, DirectiveValue::Number(number, mode)));
                            found = true;
                            break;
                        }
                    },
                    DirectiveType::String => {
                        if let Some(string) = self.eat_if_string(context) {
                            context.add_ast(token_index,Ast::Directive(directive_type, DirectiveValue::String(string)));
                            found = true;
                            break;
                        }
                    },
                }                
            }

            if !found {
                return Err(AstGeneratorError::syntax_issue(context, token_index, "Missing information"))
            }
        } else {
            return Err(AstGeneratorError::syntax_issue(context, token_index, "Unsupported compiler configuration"))
        }
        Ok(())
    }

    fn generate_branch(&self, context: &Context<'a>, token_index: usize, name: &'a [u8], branch_type: BranchType) -> Result<(), AstGeneratorError> {
        context.add_ast(token_index,Ast::Branch(name, branch_type));
        Ok(())
    }

    fn generate_assign(&self, context: &Context<'a>, token_index: usize, name: &'a [u8]) -> Result<(), AstGeneratorError> {
        self.cleanup_space(context)?;
        self.eat_assign(context)?;
        self.cleanup_space(context)?;
        let (number, mode) = self.eat_number(context)?;
        context.add_ast(token_index,Ast::Assign(name, number, mode));
        Ok(())
    }
    
    fn generate_code_block(&self, context: &Context<'a>, token_index: usize, positon: usize) -> Result<(), AstGeneratorError> {

        if INSTS_SIZE[positon] == 1 {
            context.add_ast(token_index,Ast::InstrImplied(positon));
        }

        else if BRANCH_INSTS.contains(&positon) {
            // Branch inst
            self.eat_space(context)?;
            let text = self.eat_text(context)?;
            context.add_ast(token_index,Ast::InstrBranch(positon, text));
        }

        else if JUMP_INSTS.contains(&positon) {
            // Jump inst
            self.eat_space(context)?;
            let token_index= self.eat()?;
            let token = &context.tokens.borrow()[token_index];
            let ast = match token.token {
                Token::Keyword(name) => Ast::InstrJump(positon, name),
                Token::Number(number, ModeType::Absolute) => Ast::Instr(positon, number, ModeType::Absolute),
                Token::Number(number, ModeType::Indirect) => Ast::Instr(positon, number, ModeType::Indirect),
                _ => return Err(AstGeneratorError::syntax_issue(context, token_index, "Branch name, absolute address or indirect address expected")),
            };
            context.add_ast(token_index, ast);
        }

        else {
            self.eat_space(context)?;

            let token_index= self.eat()?;
            let token = &context.tokens.borrow()[token_index];

            let ast = match &token.token {
                Token::Keyword(keyword) => Ast::InstrRef(positon, keyword),
                Token::Number(number, mode) => Ast::Instr(positon, *number, *mode),
                _ => return Err(AstGeneratorError::syntax_issue(context, token_index, "Keyword or reference expected"))
            };

            context.add_ast(token_index, ast);
        }
        
        Ok(())
    }
    
    fn inline_generate(&self, context: &Context<'a>) -> Result<(), AstGeneratorError> {
        self.size.set(context.tokens.borrow().len());

        while self.size.get() > self.index.get() {
            let token_index = self.eat()?;

            match &context.tokens.borrow().get(token_index).map(|item| item.token) {
                Some(Token::Instr(positon)) => self.generate_code_block(&context, token_index, *positon)?,
                Some(Token::Keyword(keyword)) => self.generate_assign(&context, token_index, keyword)?,
                Some(Token::Directive(option)) => self.generate_directive(&context, token_index, option)?,
                Some(Token::Comment(_)) => (),
                Some(Token::Branch(name)) => self.generate_branch(&context, token_index, name, BranchType::Generic)?,
                Some(Token::Number(_, _)) => return Err(AstGeneratorError::syntax_issue(&context, token_index, "Number not expected")),
                Some(Token::NewLine(_)) => (),
                Some(Token::Space(_)) => (),
                Some(Token::Assign) => return Err(AstGeneratorError::syntax_issue(&context, token_index, "'=' not expected")),
                Some(Token::String(_)) => return Err(AstGeneratorError::syntax_issue(&context, token_index, "String not expected")),
                Some(Token::BranchNext(name)) => self.generate_branch(&context, token_index, name, BranchType::Next)?,
                Some(Token::End) => break,
                None => return Err(AstGeneratorError::InternalError)
            }
        }

        Ok(())
    }
    
    pub fn generate(&self, context: Context<'a>) -> Result<Context<'a>, AstGeneratorError> {
        match self.inline_generate(&context) {
            Ok(_) => Ok(context),
            Err(error) => {
                let tokens = context.tokens.borrow();
                let token = &tokens[self.index.get() - 1];
                print_error(&context.source, &error, token.line, token.column, token.end);
                Err(error)
            }
        }
    }
}