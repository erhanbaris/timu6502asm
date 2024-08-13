use crate::opcode::{ModeType, INSTS};

/*
Address Modes
A	    Accumulator	        OPC A	    operand is AC (implied single byte instruction)
abs	    absolute	        OPC $LLHH	operand is address $HHLL *
abs,X	absolute, X-indexed	OPC $LLHH,X	operand is address; effective address is address incremented by X with carry **
abs,Y	absolute, Y-indexed	OPC $LLHH,Y	operand is address; effective address is address incremented by Y with carry **
#	    immediate	        OPC #$BB	operand is byte BB
impl	implied	            OPC	        operand implied
ind	    indirect	        OPC ($LLHH)	operand is address; effective address is contents of word at address: C.w($HHLL)
X,ind	X-indexed, indirect	OPC ($LL,X)	operand is zeropage address; effective address is word in (LL + X, LL + X + 1), inc. without carry: C.w($00LL + X)
ind,Y	indirect, Y-indexed	OPC ($LL),Y	operand is zeropage address; effective address is word in (LL, LL + 1) incremented by Y with carry: C.w($00LL) + Y
rel	    relative	        OPC $BB	    branch target is PC + signed offset BB ***
zpg	    zeropage	        OPC $LL	    operand is zeropage address (hi-byte is zero, address = $00LL)
zpg,X	zeropage, X-indexed	OPC $LL,X	operand is zeropage address; effective address is address incremented by X without carry **
zpg,Y	zeropage, Y-indexed	OPC $LL,Y	operand is zeropage address; effective address is address incremented by Y without carry **
*/


#[derive(Debug)]
pub struct Parser<'a> {
    data: &'a [u8],
    pub index: usize,
    pub line: usize,
    pub column: usize,
    pub end: usize,
    size: usize,
    pub tokens: Vec<TokenInfo<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Instr(usize),
    Keyword(&'a [u8]),
    String(&'a [u8]),
    CompilerOption(&'a [u8]),
    Comment(&'a [u8]),
    Branch(&'a [u8]),
    Number(u16, ModeType),
    NewLine(usize),
    Space(usize),
    End,
}

#[derive(Debug)]
pub struct TokenInfo<'a> {
    pub line: usize,
    pub column: usize,
    pub token: Token<'a>,
    pub end: usize,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    OutOfScope,
    UnknownToken,
    InvalidNumberFormat,
    InvalidCommentFormat,
    InvalidKeyword,
    InvalidCompilerOption,
    InvalidString
}

impl<'a> Parser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            index: 0,
            line: 0,
            column: 0,
            end: 0,
            size: data.len(),
            tokens: Vec::new(),
        }
    }

    fn add_token(&mut self, token: Token<'a>) {
        self.tokens.push(TokenInfo {
            line: self.line,
            column: self.column,
            end: self.end,
            token,
        });
    }

    pub fn parse(&mut self) -> Result<(), ParseError> {
        while self.size > self.index {
            let mut total_lines = 0;
            let token = self.next()?;

            if let Token::NewLine(lines) = token {
                total_lines = lines;
            }

            self.add_token(token);

            if total_lines > 0 {
                self.end = 0;
                self.line += total_lines;
            }

            self.column = self.end;
        }

        self.add_token(Token::End);
        Ok(())
    }

    fn peek(&mut self) -> Result<u8, ParseError> {
        self.empty_check()?;
        Ok(self.data[self.index])
    }

    fn peek2(&mut self) -> Result<u8, ParseError> {
        self.empty_check2()?;
        Ok(self.data[self.index+1])
    }

    fn peek_expected(&mut self, byte: u8, error: ParseError) -> Result<(), ParseError> {
        if self.peek()? != byte {
            return Err(error);
        }
        Ok(())
    }

    fn eat(&mut self) -> Result<u8, ParseError> {
        self.empty_check()?;
        self.index += 1;
        self.end += 1;
        Ok(self.data[self.index - 1])
    }

    fn eat_expected(&mut self, byte: u8, error: ParseError) -> Result<(), ParseError> {
        if self.eat()? != byte {
            return Err(error);
        }
        Ok(())
    }

    fn eat_spaces(&mut self) -> Result<(), ParseError> {
        loop {
            if self.peek() == Ok(b' ') || self.peek() == Ok(b'\t') {
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

    fn empty_check2(&mut self) -> Result<(), ParseError> {
        match self.index + 1 >= self.size {
            true => Err(ParseError::OutOfScope),
            false => Ok(()),
        }
    }

    fn dec(&mut self) -> Result<(), ParseError> {
        if self.index > 0 {
            self.index -= 1;
            self.end -= 1;
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
            b'.' => self.parse_compiler_options(),
            b'"' => self.parse_string(),
            b';' => self.parse_comment(),
            b'\r' | b'\n' => self.parse_newline(),
            b' ' | b'\t' => self.parse_whitespace(),
            n => {
                println!("{}", n);
                Err(ParseError::UnknownToken)
            }
        }
    }

    fn parse_absolute_mode(&mut self, number: u16, is_absolute: bool) -> Result<Token<'a>, ParseError> {
        self.eat_spaces()?;

        if self.peek() == Ok(b',') {
            self.eat()?; // Eat ,
            self.eat_spaces()?;

            match self.eat()? {
                b'x' | b'X' => Ok(Token::Number(number, match is_absolute {
                    true => ModeType::AbsoluteX,
                    false => ModeType::ZeroPageX
                })),
                b'y' | b'Y' => Ok(Token::Number(number, match is_absolute {
                    true => ModeType::AbsoluteY,
                    false => ModeType::ZeroPageY
                })),
                _ => Err(ParseError::InvalidNumberFormat),
            }
        } else {
            Ok(Token::Number(number, match is_absolute {
                true => ModeType::Absolute,
                false => ModeType::ZeroPage
            }))
        }
    }

    fn parse_absolute_decimal(&mut self) -> Result<Token<'a>, ParseError> {
        let (size, number) = self.parse_decimal()?;

        self.parse_absolute_mode(number, size == 2)
    }

    fn parse_absolute_hex(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat_expected(b'$', ParseError::InvalidNumberFormat)?;
     
        let (size, number) = self.parse_hex()?;
        self.parse_absolute_mode(number, size == 2)
    }

    fn parse_absolute_binary(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat_expected(b'%', ParseError::InvalidNumberFormat)?;

        let (size, number) = self.parse_binary()?;
        self.parse_absolute_mode(number, size == 2)
    }

    fn parse_indirect(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat_expected(b'(', ParseError::InvalidNumberFormat)?;
        self.eat_spaces()?;

        let first = self.eat();

        let (size, number) = match first {
            Ok(b'$') => self.parse_hex()?,
            Ok(b'%') => self.parse_binary()?,
            Ok(b'0'..=b'9') => {
                let _ = self.dec(); // Give back what you eat
                self.parse_decimal()?
            },
            _ => return Err(ParseError::InvalidNumberFormat),
        };

        if size == 2 { // For ($0x0000) to ($0xffff) numbers
            self.eat_spaces()?;
            self.eat_expected(b')', ParseError::InvalidNumberFormat)?;
            return Ok(Token::Number(number, ModeType::Indirect));
        }

        self.eat_spaces()?;
        let next_byte = self.eat()?;
        match next_byte {
            b',' => {
                self.eat_spaces()?;
                self.peek_expected(b'X', ParseError::InvalidNumberFormat).or(self.peek_expected(b'x', ParseError::InvalidNumberFormat))?;
                let _ = self.eat(); // Eat x or X

                self.eat_expected(b')', ParseError::InvalidNumberFormat)?;
                Ok(Token::Number(number, ModeType::IndirectX))
            }
            b')' => {
                self.eat_spaces()?;
                self.eat_expected(b',', ParseError::InvalidNumberFormat)?;
                self.eat_spaces()?;

                self.peek_expected(b'Y', ParseError::InvalidNumberFormat).or(self.peek_expected(b'y', ParseError::InvalidNumberFormat))?;
                let _ = self.eat(); // Eat y or Y
                Ok(Token::Number(number, ModeType::IndirectY))
            }
            _ => Err(ParseError::InvalidNumberFormat),
        }
    }

    fn parse_immediate(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat()?; //Eat # char

        let number = self.parse_number()?;
        Ok(Token::Number(number, ModeType::Immediate))
    }

    fn parse_number(&mut self) -> Result<u16, ParseError> {
        match self.eat()? {
            b'$' => self.parse_hex().map(|(_, number)| number),
            b'%' => self.parse_binary().map(|(_, number)| number),
            _ => {
                self.dec()?;
                self.parse_decimal().map(|(_, number)| number)
            }
        }
    }

    fn parse_hex(&mut self) -> Result<(u8, u16), ParseError> {
        let mut hex_number: u16 = 0;
        let mut count: u8 = 0;
        
        while let Ok(n) = self.peek() {
            let number = match n {
                b'0'..=b'9' => n - b'0',
                b'A'..=b'F' => (n - b'A') + 10,
                b'a'..=b'f' => (n - b'a') + 10,
                b' ' | b'\r' | b'\t' | b'\n' | b',' | b')' => break,
                _ => return Err(ParseError::InvalidNumberFormat),
            };

            hex_number = hex_number << 4 | number as u16;
            count += 1;
            let _ = self.eat();
        }
        
        if count != 2 && count != 4 {
            return Err(ParseError::InvalidNumberFormat);
        }

        Ok((count / 2, hex_number))
    }

    fn parse_binary(&mut self) -> Result<(u8, u16), ParseError> {
        let mut binary_number: u16 = 0b0000_0000_0000_0000;
        let mut count: u8 = 0;
        
        while let Ok(n) = self.peek() {
            let number: u16 = match n {
                b'0' => 0,
                b'1' => 1,
                b' ' | b'\r' | b'\t' | b'\n' | b',' | b')' => break,
                _ => return Err(ParseError::InvalidNumberFormat),
            };

            binary_number = binary_number << 1 | number;
            count += 1;
            let _ = self.eat();
        }
        
        if count != 8 && count != 16 {
            return Err(ParseError::InvalidNumberFormat);
        }

        Ok((count / 8, binary_number))
    }

    fn parse_decimal(&mut self) -> Result<(u8, u16), ParseError> {
        let mut decimal_number: u16 = 0;
        
        while let Ok(n) = self.peek() {
            let number = match n {
                n @ b'0'..=b'9' => n - b'0',
                b' ' | b'\r' | b'\t' | b'\n' | b',' | b')' => break,
                _ => return Err(ParseError::InvalidNumberFormat),
            };

            decimal_number = (decimal_number * 10) + number as u16;
            let _ = self.eat();
        }

        Ok((match decimal_number > 0xff_u16 {
            true => 2,
            false => 1
        }, decimal_number))
    }

    fn parse_keyword(&mut self) -> Result<Token<'a>, ParseError> {
        let start = self.index;

        let mut valid = false;
        let mut branch = false;

        loop {
            match self.peek() {
                Ok(byte) => {
                    match byte {
                        b'0'..=b'9' => (),
                        b'a'..=b'z' => valid = true,
                        b'A'..=b'Z' => valid = true,
                        b'_' => (),
                        b' ' | b'\t' => break,
                        b'\n' | b'\r' => break,
                        b':' => {
                            branch = true;
                            self.eat()?;
                            break;
                        }
                        _ => return Err(ParseError::InvalidKeyword),
                    };
                    self.eat()?;
                }
                Err(ParseError::OutOfScope) => break,
                _ => return Err(ParseError::InvalidKeyword),
            };
        }

        if !valid {
            return Err(ParseError::InvalidKeyword);
        }

        if branch {
            return Ok(Token::Branch(&self.data[start..self.index - 1]));
        }

        if self.index - start == 3 {
            let search_insts: [u8; 3] = [self.data[start], self.data[start + 1], self.data[start + 2]];
            if let Some(position) = INSTS.iter().position(|item| *item == &search_insts) {
                return Ok(Token::Instr(position));
            }
        }

        Ok(Token::Keyword(&self.data[start..self.index]))
    }

    fn parse_string(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat_expected(b'"', ParseError::InvalidString)?;
        let start = self.index;

        loop {
            match self.peek() {
                Ok(byte) => {
                    match byte {
                        b'"' => break,
                        b'\\' => {
                            if self.peek2()? == b'"' { // It is inline \"
                                self.eat()?;
                            }
                        },
                        _ => ()
                    };
                    self.eat()?;
                }
                _ => return Err(ParseError::InvalidString),
            };
        }

        self.eat_expected(b'"', ParseError::InvalidString)?;
        Ok(Token::String(&self.data[start..self.index]))
    }

    fn parse_compiler_options(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat_expected(b'.', ParseError::InvalidCompilerOption)?;
        let start = self.index;

        let mut valid = false;

        loop {
            match self.peek() {
                Ok(byte) => {
                    match byte {
                        b'0'..=b'9' => (),
                        b'a'..=b'z' => valid = true,
                        b'A'..=b'Z' => valid = true,
                        b'_' => (),
                        b' ' | b'\t' | b'\n' | b'\r' => break,
                        _ => return Err(ParseError::InvalidCompilerOption),
                    };
                    self.eat()?;
                }
                Err(ParseError::OutOfScope) => break,
                _ => return Err(ParseError::InvalidCompilerOption),
            };
        }

        if !valid {
            return Err(ParseError::InvalidCompilerOption);
        }

        Ok(Token::CompilerOption(&self.data[start..self.index]))
    }

    fn parse_comment(&mut self) -> Result<Token<'a>, ParseError> {
        let start = self.index;

        loop {
            match self.eat() {
                Ok(byte) => match byte {
                    b'\n' | b'\r' => {
                        self.dec()?;
                        break;
                    }
                    _ => continue,
                },
                Err(ParseError::OutOfScope) => break,
                _ => return Err(ParseError::InvalidCommentFormat),
            };
        }
        Ok(Token::Comment(&self.data[start..self.index - 1]))
    }

    fn parse_newline(&mut self) -> Result<Token<'a>, ParseError> {
        let mut total_lines = 0;

        loop {
            match self.peek() {
                Ok(b'\r') => (),
                Ok(b'\n') => total_lines += 1,
                _ => break,
            };
            self.eat()?;
        }
        Ok(Token::NewLine(total_lines))
    }

    fn parse_whitespace(&mut self) -> Result<Token<'a>, ParseError> {
        let mut total_whitespaces = 0;

        while let Ok(b' ') | Ok(b'\t') = self.peek() {
            total_whitespaces += 1;
            self.eat()?;
        }

        Ok(Token::Space(total_whitespaces))
    }

    pub fn friendly_dump(&self) {
        let mut line = 0;

        print!("{}. ", line);
        for ast in self.tokens.iter() {
            let type_name = match ast.token {
                Token::Instr(_) => "INS",
                Token::Keyword(_) => "KEY",
                Token::CompilerOption(_) => "OPT",
                Token::Comment(_) => "COM",
                Token::Branch(_) => "BRN",
                Token::Number(_, _) => "NUM",
                Token::NewLine(_) => "NLN",
                Token::Space(_) => "SPA",
                Token::End => "END",
                Token::String(_) => "STR"
            };

            if ast.line != line {
                println!();
                line = ast.line;
                print!("{}. ", line);
            }

            print!("[{}:{}]{} ", ast.column, ast.end, type_name);
        }
        println!();
    }
}
