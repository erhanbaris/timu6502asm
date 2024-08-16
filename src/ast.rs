use std::{cell::Cell, fs::File, io::Read, marker::PhantomData};

use crate::{context::Context, directive::{DirectiveEnum, DirectiveType, DirectiveValue, SYSTEM_DIRECTIVES}, opcode::{ModeType, BRANCH_INSTS, INSTS_SIZE, JUMP_INSTS}, parser::{Parser, Token, TokenInfo, TokenType}, tool::{print_error, upper_case}};

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
    Branch(&'a [u8], BranchType),
    Directive(DirectiveEnum, Vec<DirectiveValue<'a>>)
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
        #[allow(dead_code)] message: String
    },
    OutOfScope,
    InternalError,
    FileNotValid
}

impl AstGeneratorError {
    pub fn syntax_issue<'a>(context: &Context<'a>, token_index: usize, message: String) -> Self {
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

    fn peek(&self)-> Result<usize, AstGeneratorError> {
        self.empty_check()?;
        Ok(self.index.get())
    }
    
    fn eat_expected(&self, context: &Context<'a>, token_type: TokenType, error: AstGeneratorError) -> Result<(), AstGeneratorError> {
        let token_index = self.eat()?;
        let token = &context.tokens.borrow()[token_index];

        if TokenType::from(&token.token) != token_type {
            return Err(error);
        }
        Ok(())
    }

    fn eat_space(&self, context: &Context<'a>) -> Result<(), AstGeneratorError> {
        let token_index= self.eat()?;
        let token = &context.tokens.borrow()[token_index];
        match token.token {
            Token::Space(_) => Ok(()),
            _ => Err(AstGeneratorError::syntax_issue(context, token_index, "Expected space".to_string()))
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
        let token_type: TokenType = TokenType::from(&token.token);
        
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

        if let Ok(mut position) = self.peek() {
            let tokens = context.tokens.borrow();
            let mut immediate = false;
            let mut mode = ModeType::ZeroPage;
            let mut number = 0_u16;
            let index = self.index.get();
            
            if let Token::Sharp = &tokens[position].token {
                let _ = self.eat();
                immediate = true;
                if let Ok(new_position) = self.peek() {
                    position = new_position;
                } else {
                    self.index.set(index);
                    return None;
                }
            }
            
            if let Token::Byte(byte) = &tokens[position].token {
                let _ = self.eat();
                mode = ModeType::ZeroPage;
                number = *byte as u16;
            }

            else if let Token::Word(word) = &tokens[position].token {
                let _ = self.eat();
                mode = ModeType::Absolute;
                number = *word;
            }

            else if let Token::Keyword(keyword) = &tokens[position].token {
                let references = context.references.borrow();
                let values = references.get(keyword)?;
                if values.len() != 1 {
                    self.index.set(index);
                    return None
                }

                let first_value = values[0];
                (number, mode) = match first_value {
                    DirectiveValue::Byte(number) => (number as u16, ModeType::ZeroPage),
                    DirectiveValue::Word(number) => (number as u16, ModeType::Absolute),
                    _ => {
                        self.index.set(index);
                        return None
                    }
                };
                let _ = self.eat();
            }

            return match immediate {
                true => Some((number, ModeType::Immediate)),
                false => match mode == ModeType::Absolute {
                    true => Some((number, ModeType::Absolute)),
                    false => Some(((number as u8) as u16, ModeType::ZeroPage)),
                },
            };
        }

        None
    }

    fn eat_number(&self, context: &Context<'a>) -> Result<(u16, ModeType), AstGeneratorError> {
        let token_index= self.eat()?;
        let token = &context.tokens.borrow()[token_index];
        match token.token {
            Token::Byte(number) => Ok((number as u16, ModeType::ZeroPage)),
            Token::Word(number) => Ok((number, ModeType::Absolute)),
            _ => Err(AstGeneratorError::syntax_issue(context, token_index, "Expected number".to_string()))
        }
    }
    
    fn eat_string(&self, context: &Context<'a>) -> Result<&'a [u8], AstGeneratorError> {
        let token_index= self.eat()?;
        let token = &context.tokens.borrow()[token_index];
        match token.token {
            Token::String(string) => Ok(string),
            _ => Err(AstGeneratorError::syntax_issue(context, token_index, "Expected string".to_string()))
        }
    }
    
    fn eat_assign(&self, context: &Context<'a>) -> Result<(), AstGeneratorError> {
        let token_index= self.eat()?;
        let token = &context.tokens.borrow()[token_index];
        match token.token {
            Token::Assign => Ok(()),
            _ => Err(AstGeneratorError::syntax_issue(context, token_index, "Expected assign".to_string()))
        }
    }

    fn eat_text(&self, context: &Context<'a>) -> Result<&'a [u8], AstGeneratorError> {
        let token_index= self.eat()?;
        let token = &context.tokens.borrow()[token_index];
        match token.token {
            Token::Keyword(text) => Ok(text),
            _ => Err(AstGeneratorError::syntax_issue(context, token_index, "Expected text".to_string()))
        }
    }

    fn parse_list(&'a self, context: &Context<'a>, validator: impl Fn(DirectiveType) -> bool) -> Result<Vec<DirectiveValue>, AstGeneratorError>  {
        let tokens = context.tokens.borrow();

        let mut token_found = false;
        let mut finish = false;

        self.cleanup_space(context)?;
        let mut values = Vec::new();

        while self.size.get() > self.index.get() {
            let value_index = self.eat()?;
            let value_token = &tokens.get(value_index).map(|item| &item.token);

            if token_found {
                /* comma, space, new line, end or comment expected */
                match value_token {
                    Some(Token::NewLine(_)) => finish = true,
                    Some(Token::Comment(_)) => finish = true,
                    Some(Token::End) => finish = true,
                    Some(Token::Space(_)) => (),
                    Some(Token::Comma) => token_found = false,
                    _ => return Err(AstGeneratorError::syntax_issue(context, value_index, "Unexpected syntax".to_string()))
                }
            }
            else {
                /* Expected parseable token */
                match value_token {
                    Some(Token::Keyword(keyword)) => { values.push(DirectiveValue::Reference(*keyword)); token_found = true; },
                    Some(Token::Word(number)) => { values.push(DirectiveValue::Word(*number)); token_found = true; },
                    Some(Token::Byte(number)) => { values.push(DirectiveValue::Byte((*number) as u8)); token_found = true; },
                    Some(Token::String(string)) => { values.push(DirectiveValue::String(*string)); token_found = true; },
                    Some(Token::BranchNext(name)) => { values.push(DirectiveValue::Reference(*name)); token_found = true; },
                    Some(Token::NewLine(_)) => finish = true,
                    Some(Token::Comment(_)) => finish = true,
                    Some(Token::End) => finish = true,
                    Some(Token::Space(_)) => (),
                    Some(Token::Comma) => return Err(AstGeneratorError::syntax_issue(&context, value_index, "',' not expected".to_string())),
                    Some(_) => return Err(AstGeneratorError::syntax_issue(context, value_index, "Unexpected syntax".to_string())),
                    None => return Err(AstGeneratorError::InternalError)
                };
            }

            if token_found {
                if !validator(DirectiveType::from(&values[values.len()-1])) {
                    return Err(AstGeneratorError::syntax_issue(context, value_index, "Unexpected syntax".to_string()))
                }
            }

            if finish {
                break;
            }
        }

        Ok(values)
    }

    fn generate_directive(&'a self, context: &Context<'a>, token_index: usize, directive_name: &'a [u8]) -> Result<(), AstGeneratorError> {
        let directive_name = upper_case(directive_name);
        if let Some(directive) = SYSTEM_DIRECTIVES.iter().find(|item| item.name == &directive_name[..]) {

            let values = self.parse_list(context, |directive_type| -> bool {
                return directive_type == DirectiveType::Reference || directive.values.iter().any(|mode| *mode == directive_type)
            })?;

            match directive.size {
                crate::directive::DirectiveVariableSize::None => {
                    if values.len() != 0 {
                        return Err(AstGeneratorError::syntax_issue(context, token_index, "No value expected".to_string()));
                    }
                },
                crate::directive::DirectiveVariableSize::Min(min) => {
                    if values.len() < min {
                        return Err(AstGeneratorError::syntax_issue(context, token_index, format!("Minimum {} value(s) expected", min)));
                    }
                },
                crate::directive::DirectiveVariableSize::Length(len) => {
                    if values.len() != len {
                        return Err(AstGeneratorError::syntax_issue(context, token_index, format!("Expected {} value(s)", len)));
                    }
                },
            }

            if directive.values.len() > 0 && values.len() == 0 {
                return Err(AstGeneratorError::syntax_issue(context, token_index, "Missing information".to_string()))
            }

            if DirectiveEnum::Include == directive.directive {
                self.process_include(context, token_index, &values[0])?;
            }

            context.add_ast(token_index, Ast::Directive(directive.directive, values));

        } else {
            return Err(AstGeneratorError::syntax_issue(context, token_index, "Unsupported compiler configuration".to_string()))
        }
        Ok(())
    }

    fn process_include(&self, context: &Context<'a>, token_index: usize, value: &DirectiveValue<'a>) -> Result<(), AstGeneratorError> {
        let file_path = match value {
            DirectiveValue::String(name) => name,
            _ => return Err(AstGeneratorError::syntax_issue(&context, token_index, "Path expected as a string".to_string()))
        };

        let file_path = match std::str::from_utf8(file_path) {
            Ok(file_path) => file_path,
            _ => return Err(AstGeneratorError::syntax_issue(&context, token_index, "Invalid text format".to_string()))
        };

        let mut file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => return Err(AstGeneratorError::FileNotValid)
        };

        let mut code = Vec::new();
        match file.read_to_end(&mut code) {
            Ok(_) => (),
            Err(_) => return Err(AstGeneratorError::FileNotValid)
        };

        let new_context = Context::new(&code);

        let mut parser = Parser::new(new_context);
        parser.parse().unwrap();

        let mut tokens = context.tokens.borrow_mut();
        let current_position = self.index.get();

        for token in new_context.tokens.borrow().iter().rev() {
            tokens.insert(current_position, token.clone());
        }

        Ok(())
    }

    fn generate_branch(&self, context: &Context<'a>, token_index: usize, name: &'a [u8], branch_type: BranchType) -> Result<(), AstGeneratorError> {
        context.add_ast(token_index,Ast::Branch(name, branch_type));
        Ok(())
    }

    fn generate_assign(&'a self, context: &Context<'a>, token_index: usize, name: &'a [u8]) -> Result<(), AstGeneratorError> {
        self.cleanup_space(context)?;
        self.eat_assign(context)?;
        self.cleanup_space(context)?;

        let values = self.parse_list(context, |_| true)?;
        context.references.borrow_mut().insert(name, values);
        Ok(())
    }

    pub(crate) fn try_parse_number(&self, context: &Context<'a>) -> Result<(u16, ModeType), AstGeneratorError> {
        self.cleanup_space(context)?;
        let tokens = context.tokens.borrow();
        let token_index = self.peek()?;
        let token = &tokens[token_index];

        if let Token::OpenParenthesis = token.token {
            let mut mode = ModeType::Indirect;
            let mut parenthesis_closed = false;
            self.eat()?;
            self.cleanup_space(context)?;
            
            let Some((number, _)) = self.eat_if_number(context) else {
                return Err(AstGeneratorError::syntax_issue(context, token_index, "Invalid numbering number format".to_string()));
            };

            self.cleanup_space(context)?;

            let token_index = self.peek()?;
            let token = &tokens[token_index];
            if let Token::OpenParenthesis = token.token {
                self.eat()?;
                parenthesis_closed = true;
            }
            
            self.cleanup_space(context)?;
            let token_index = self.peek()?;
            let token = &tokens[token_index];
            if let Token::Comma = token.token {
                self.eat()?;
                self.cleanup_space(context)?;

                let token_index = self.peek()?;
                let token = &tokens[token_index];

                mode = match &token.token {
                    Token::Keyword(&[b'x']) |Token::Keyword(&[b'X']) => ModeType::IndirectX,
                    Token::Keyword(&[b'y']) |Token::Keyword(&[b'Y']) => ModeType::IndirectY,
                    _ => return Err(AstGeneratorError::syntax_issue(context, token_index, "Expected X or Y".to_string()))
                };

                
                self.eat()?;
            }

            self.cleanup_space(context)?;

            if !parenthesis_closed {
                self.eat_expected(context, TokenType::CloseParenthesis, AstGeneratorError::syntax_issue(context, token_index, "Expected ')'".to_string()))?;
            }

            return Ok((number, mode));

        } else {
            self.cleanup_space(context)?;
            
            let Some((number, mut mode)) = self.eat_if_number(context) else {
                return Err(AstGeneratorError::syntax_issue(context, token_index, "Invalid numbering number format".to_string()));
            };

            if mode == ModeType::Immediate {
                return Ok((number, mode));
            }

            self.cleanup_space(context)?;
            let token_index = self.peek()?;
            let token = &tokens[token_index];
            if let Token::Comma = token.token {
                self.eat()?;
                self.cleanup_space(context)?;

                let token_index = self.peek()?;
                let token = &tokens[token_index];

                mode = match &token.token {
                    Token::Keyword(&[b'x']) |Token::Keyword(&[b'X']) => match mode {
                        ModeType::Absolute => ModeType::AbsoluteX,
                        ModeType::ZeroPage => ModeType::ZeroPageX,
                        _ => return Err(AstGeneratorError::syntax_issue(context, token_index, "Invalid usage".to_string()))
                    },
                    Token::Keyword(&[b'y']) |Token::Keyword(&[b'Y']) => match mode {
                        ModeType::Absolute => ModeType::AbsoluteY,
                        ModeType::ZeroPage => ModeType::ZeroPageY,
                        _ => return Err(AstGeneratorError::syntax_issue(context, token_index, "Invalid usage".to_string()))
                    },
                    _ => return Err(AstGeneratorError::syntax_issue(context, token_index, "Expected X or Y".to_string()))
                };
                self.eat()?;
            }

            return Ok((number, mode));
        }
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
            let index = self.index.get();
            if let Ok((number, mode)) = self.try_parse_number(context) {
                context.add_ast(token_index, Ast::Instr(positon, number, mode));
                return Ok(())
            }

            self.index.set(index); // Restore index

            let token_index= self.eat()?;
            let token = &context.tokens.borrow()[token_index];
            if let Token::Keyword(name) = token.token {
                context.add_ast(token_index, Ast::InstrJump(positon, name));
                return Ok(())
            }

            return Err(AstGeneratorError::syntax_issue(context, token_index, "Branch name, absolute address or indirect address expected".to_string()))
        }

        else {
            self.eat_space(context)?;
            let (number, mode) = self.try_parse_number(context)?;
            context.add_ast(token_index, Ast::Instr(positon, number, mode));
        }
        
        Ok(())
    }
    
    fn inline_generate(&'a self, context: &Context<'a>) -> Result<(), AstGeneratorError> {
        self.size.set(context.tokens.borrow().len());

        while self.size.get() > self.index.get() {
            let token_index = self.eat()?;

            match &context.tokens.borrow().get(token_index).map(|item| &item.token) {
                Some(Token::Instr(positon)) => self.generate_code_block(&context, token_index, *positon)?,
                Some(Token::Keyword(keyword)) => self.generate_assign(&context, token_index, keyword)?,
                Some(Token::Directive(option)) => self.generate_directive(&context, token_index, option)?,
                Some(Token::Comment(_)) => (),
                Some(Token::Branch(name)) => self.generate_branch(&context, token_index, name, BranchType::Generic)?,
                Some(Token::Byte(_)) => return Err(AstGeneratorError::syntax_issue(&context, token_index, "Number not expected".to_string())),
                Some(Token::Word(_)) => return Err(AstGeneratorError::syntax_issue(&context, token_index, "Number not expected".to_string())),
                Some(Token::NewLine(_)) => (),
                Some(Token::Space(_)) => (),
                Some(Token::OpenParenthesis) => return Err(AstGeneratorError::syntax_issue(&context, token_index, "'(' not expected".to_string())),
                Some(Token::CloseParenthesis) => return Err(AstGeneratorError::syntax_issue(&context, token_index, "')' not expected".to_string())),
                Some(Token::Sharp) => return Err(AstGeneratorError::syntax_issue(&context, token_index, "'#' not expected".to_string())),
                Some(Token::Assign) => return Err(AstGeneratorError::syntax_issue(&context, token_index, "'=' not expected".to_string())),
                Some(Token::Comma) => return Err(AstGeneratorError::syntax_issue(&context, token_index, "',' not expected".to_string())),
                Some(Token::String(_)) => return Err(AstGeneratorError::syntax_issue(&context, token_index, "String not expected".to_string())),
                Some(Token::BranchNext(name)) => self.generate_branch(&context, token_index, name, BranchType::Next)?,
                Some(Token::End) => break,
                None => return Err(AstGeneratorError::InternalError)
            }
        }

        Ok(())
    }
    
    pub fn generate(&'a self, context: Context<'a>) -> Result<Context<'a>, AstGeneratorError> {
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