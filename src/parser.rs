use crate::opcode::{ModeType, BRANCH_INSTS, INSTS, INSTS_SIZE};


#[derive(Debug)]
pub struct Parser<'a> {
    data: &'a [u8],
    index: usize,
    size: usize,
    pub asts: Vec<Ast<'a>>
}

#[derive(Debug)]
pub enum Ast<'a> {
    InstrImplied(usize),
    InstrBranch(usize, &'a [u8]),
    Instr(usize, u16, ModeType),
    Branch(&'a [u8])
}

#[derive(Debug)]
pub enum Token<'a> {
    Instr(usize),
    Text(&'a [u8]),
    Branch(&'a [u8]),
    Number(u16, ModeType)
}

#[derive(Debug)]
pub enum ParseError {
    OutOfScope,
    UnknownToken,
    InvalidNumberFormat,
    InvalidText
}

impl<'a> Parser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            index: 0,
            size: data.len(),
            asts: Vec::new()
        }
    }

    fn add_ast(&mut self, ast: Ast<'a>) {
        self.asts.push(ast);
    }

    pub fn parse(&mut self) -> Result<(), ParseError> {
        self.eat_newlines()?;

        while self.size > self.index {
            let token = self.next()?;

            let ast = match token {
                Token::Text(_) => todo!(),
                Token::Branch(name) => Ast::Branch(name),
                Token::Number(_, _) => todo!(),
                Token::Instr(positon) => {

                    if BRANCH_INSTS.contains(&positon) {
                        // Branch inst
                        let branch_name = self.parse_keyword()?;
                        let branch_name = match branch_name {
                            Token::Text(branch_name) => branch_name,
                            _ => return Err(ParseError::UnknownToken)  
                        };

                        Ast::InstrBranch(positon, branch_name)
                    }

                     else if INSTS_SIZE[positon] == 1 {
                        Ast::InstrImplied(positon)
                    }
                    
                    else {
                        self.eat_newlines()?;
                        let number = self.next()?;
    
                        let (number, mode) = match number {
                            Token::Number(number, mode) => (number, mode),
                            _ => return Err(ParseError::UnknownToken)  
                        };
                        Ast::Instr(positon, number, mode)
                    }
                },
            };

            self.add_ast(ast);

            if self.eat_newlines().is_err() {
                break;
            }
        }

        Ok(())
    }

    fn peek(&mut self) -> Result<u8, ParseError> {
        self.empty_check()?;
        Ok(self.data[self.index])
    }

    fn eat(&mut self) -> Result<u8, ParseError> {
        self.empty_check()?;
        self.index += 1;
        Ok(self.data[self.index - 1])
    }

    fn eat_expected(&mut self, byte: u8, error: ParseError) -> Result<(), ParseError> {
        if self.eat()? != byte {
            return Err(error);
        }
        Ok(())
    }

    fn eat_newlines(&mut self) -> Result<(), ParseError> {
        loop {
            if self.peek()? == b' ' || self.peek()? == b'\t' || self.peek()? == b'\r' || self.peek()? == b'\n' {
                self.eat()?;
            } else {
                break;
            }
        }

        Ok(())
    }

    fn eat_spaces(&mut self) -> Result<(), ParseError> {
        loop {
            if self.peek()? == b' ' || self.peek()? == b'\t' {
                self.eat()?;
            } else {
                break;
            }
        }

        Ok(())
    }

    fn empty_check(&mut self) -> Result<(), ParseError> {
        match self.index >= self.size {
            true => Err(ParseError::OutOfScope),
            false => Ok(()),
        }
    }

    fn dec(&mut self) -> Result<(), ParseError> {
        if self.index > 0 {
            self.index -= 1;
            Ok(())
        } else {
            Err(ParseError::OutOfScope)
        }
    }

    fn next(&mut self) -> Result<Token<'a>, ParseError> {
        let first = self.peek()?;

        match first {
            b'$' => self.parse_absolute_hex(),
            b'%' => self.parse_absolute_binary(),
            b'0'..=b'9' => self.parse_absolute_decimal(),
            b'(' => self.parse_indirect(),
            b'#' => self.parse_immediate(),
            b'a'..=b'z' | b'A'..=b'Z' => self.parse_keyword(),
            b';' => self.parse_comment(),
            _ => Err(ParseError::UnknownToken),
        }
    }

    fn parse_absolute_mode(&mut self, number: u16) -> Result<Token<'a>, ParseError>  {
        self.eat_spaces()?;

        if self.peek()? == b',' {
            self.eat_spaces()?;

            match self.eat()? {
                b'x' | b'X' => Ok(Token::Number(number, ModeType::AbsoluteX)),
                b'y' | b'Y' => Ok(Token::Number(number, ModeType::AbsoluteY)),
                _ => Err(ParseError::InvalidNumberFormat)
            }
        } else {
            Ok(Token::Number(number, ModeType::Absolute))
        }
    }
    
    fn parse_absolute_decimal(&mut self) -> Result<Token<'a>, ParseError> {
        let number = self.eat_decimal()?;
        
        match number > 255 {
            true => self.parse_absolute_mode(number),
            false => Ok(Token::Number(number, ModeType::ZeroPage))
        }
    }

    fn parse_absolute_hex(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat_expected(b'$', ParseError::InvalidNumberFormat)?;

        let high_number = self.eat_hex()?;
        let index = self.index;

        match self.eat_hex() {
            Ok(low_number) => self.parse_absolute_mode(((high_number as u16) << 8) + low_number as u16),
            Err(ParseError::InvalidNumberFormat) => {
                self.index = index;
                Ok(Token::Number(high_number as u16, ModeType::ZeroPage))
            },
            Err(error) => Err(error)
        }
    }

    fn parse_absolute_binary(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat_expected(b'%', ParseError::InvalidNumberFormat)?;

        let high_number = self.eat_binary()?;
        let index = self.index;

        match self.eat_binary() {
            Ok(low_number) => self.parse_absolute_mode(((high_number as u16) << 8) + low_number as u16),
            Err(ParseError::InvalidNumberFormat) => {
                self.index = index;
                Ok(Token::Number(high_number as u16, ModeType::ZeroPage))
            },
            Err(error) => Err(error)
        }
    }
    
    fn parse_indirect(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat_expected(b'(', ParseError::InvalidNumberFormat)?;

        let first = self.peek()?;

        let number = match first {
            b'$' => self.parse_absolute_hex()?,
            b'%' => self.parse_absolute_binary()?,
            b'0'..=b'9' => self.parse_absolute_decimal()?,
            _ => return Err(ParseError::InvalidNumberFormat),
        };

        let number = match number {
            Token::Number(number, _) => number,
            _ => return Err(ParseError::InvalidNumberFormat)
        };

        self.eat_spaces()?;
        let next_byte = self.eat()?;
        match next_byte {
            b',' => {
                self.eat_spaces()?;
                let next_byte = self.eat()?;
                if next_byte != b'X' && next_byte != b'x' {
                    Err(ParseError::InvalidNumberFormat)
                }
                else {
                    self.eat_expected(b')', ParseError::InvalidNumberFormat)?;
                    Ok(Token::Number(number, ModeType::IndirectX))
                }
            },
            b')' => {
                self.eat_spaces()?;
                self.eat_expected(b',', ParseError::InvalidNumberFormat)?;
                
                let next_byte = self.eat()?;
                if next_byte != b'Y' && next_byte != b'y' {
                    Err(ParseError::InvalidNumberFormat)
                }
                else {
                    Ok(Token::Number(number, ModeType::IndirectY))
                }
            },
            _ => Err(ParseError::InvalidNumberFormat)
        }
    }

    fn eat_hex(&mut self) -> Result<u8, ParseError> {
        let high_byte = match self.eat()? {
            n @ b'0'..=b'9' => n - b'0',
            n @ b'A'..=b'F' => (n - b'A') + 10,
            n @ b'a'..=b'f' => (n - b'a') + 10,
            _ => return Err(ParseError::InvalidNumberFormat),
        };

        let low_byte = match self.eat()? {
            n @ b'0'..=b'9' => n - b'0',
            n @ b'A'..=b'F' => (n - b'A') + 10,
            n @ b'a'..=b'f' => (n - b'a') + 10,
            _ => return Err(ParseError::InvalidNumberFormat),
        };

        Ok(((high_byte) << 4) + low_byte)
    }

    fn eat_decimal(&mut self) -> Result<u16, ParseError> {
        let mut number: u16 = 0;

        loop {
            number = (number * 10)
                + match self.eat() {
                    Ok(byte) => match byte {
                        n @ b'0'..=b'9' => n - b'0',
                        b'\n' | b'\r' | b' ' => break,
                        _ => return Err(ParseError::InvalidNumberFormat),
                    },
                    Err(ParseError::OutOfScope) => break,
                    _ => return Err(ParseError::InvalidNumberFormat),
                } as u16;
        }

        Ok(number)
    }

    fn eat_binary(&mut self) -> Result<u8, ParseError> {
        let mut number: u8 = 0b0000_0000;

        for _ in 0..4 {
            number = number << 1
                | match self.eat()? {
                    b'0' => 0,
                    b'1' => 1,
                    _ => return Err(ParseError::InvalidNumberFormat),
                };
        }

        Ok(number)
    }

    fn parse_immediate(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat()?; //Eat # char

        let number = self.parse_number()?;
        Ok(Token::Number(number, ModeType::Immediate))
    }

    fn parse_number(&mut self) -> Result<u16, ParseError> {
        match self.eat()? {
            b'$' => self.parse_hex(),
            b'%' => self.parse_binary(),
            _ => {
                self.dec()?;
                self.parse_decimal()
            }
        }
    }

    fn parse_hex(&mut self) -> Result<u16, ParseError> {
        let high_number = self.eat_hex()?;

        let index = self.index;
        let number = match self.eat_hex() {
            Ok(low_number) => ((high_number as u16) << 8) + low_number as u16,
            Err(_) => {
                self.index = index;
                high_number as u16
            }
        };

        Ok(number)
    }

    fn parse_decimal(&mut self) -> Result<u16, ParseError> {
        let number = self.eat_decimal()?;
        Ok(number)
    }

    fn parse_binary(&mut self) -> Result<u16, ParseError> {
        let high_number = self.eat_binary()?;

        let index = self.index;
        let number = match self.eat_binary() {
            Ok(low_number) => ((high_number as u16) << 8) + low_number as u16,
            Err(_) => {
                self.index = index;
                high_number as u16
            }
        };

        Ok(number)
    }

    fn parse_keyword(&mut self) -> Result<Token<'a>, ParseError> {
        let start = self.index;

        self.eat_newlines()?;
        let mut valid = false;
        let mut branch = false;

        loop {
            match self.eat() {
                Ok(byte) => match byte {
                    b'0'..=b'9' => continue,
                    b'a'..=b'z' => valid = true,
                    b'A'..=b'Z' => valid = true,
                    b'_' => continue,
                    b'\n' | b'\r' | b' ' => break,
                    b':' => {
                        branch = true;
                        break;
                    },
                    _ => return Err(ParseError::InvalidText),
                },
                Err(ParseError::OutOfScope) => break,
                _ => return Err(ParseError::InvalidText),
            }
        }

        if !valid {
            return Err(ParseError::InvalidText);
        }

        if branch {
            return Ok(Token::Branch(&self.data[start..self.index - 1]));
        }

        if (self.index - 1) - start == 3 {
            let search_insts: [u8; 3] = [self.data[start], self.data[start+1], self.data[start+2]];
            if let Some(position) = INSTS.iter().position(|item| item == &search_insts) {
                return Ok(Token::Instr(position))
            }
        }

        Ok(Token::Text(&self.data[start..self.index - 1]))
    }

    fn parse_comment(&mut self) -> Result<Token<'a>, ParseError> {
        let start = self.index;
        
        loop {
            match self.eat() {
                Ok(byte) => match byte {
                    b'\n' | b'\r' => break,
                    _ => continue,
                },
                Err(ParseError::OutOfScope) => break,
                _ => return Err(ParseError::InvalidNumberFormat),
            };
        }

        Ok(Token::Text(&self.data[start..self.index - 1]))
    }
}