use crate::{context::Context, opcode::INSTS, tool::{print_error, upper_case_byte}};
use log::info;
use strum_macros::EnumDiscriminants;

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
    pub index: usize,
    pub line: usize,
    pub column: usize,
    pub end: usize,
    size: usize,
    pub context: Context<'a>
}

#[derive(Debug, PartialEq, Clone)]
#[derive(EnumDiscriminants)]
#[strum_discriminants(name(TokenType))]
pub enum Token<'a> {
    Instr(usize),
    Keyword(&'a [u8]),
    String(&'a [u8]),
    Directive(&'a [u8]),
    Comment(&'a [u8]),
    Assign,
    Comma,
    OpenParenthesis,
    CloseParenthesis,
    Sharp,
    Branch(&'a [u8]),
    BranchNext(&'a [u8]),
    Byte(u8),
    Word(u16),
    NewLine(usize),
    Space(usize),
    End,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct TokenInfo<'a> {
    pub line: usize,
    pub column: usize,
    pub token: Token<'a>,
    pub end: usize,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    OutOfScope,
    UnexpectedSymbol,
    UnknownToken,
    InvalidNumberFormat,
    InvalidCommentFormat,
    InvalidKeyword,
    InvalidDirective,
    InvalidString
}

impl<'a> Parser<'a> {
    pub fn new(context: Context<'a>) -> Self {
        let size = context.source.len();

        Self {
            index: 0,
            line: 0,
            column: 0,
            end: 0,
            size,
            context
        }
    }

    fn add_token(&mut self, token: Token<'a>) {
        self.context.tokens.borrow_mut().push(TokenInfo {
            line: self.line,
            column: self.column,
            end: self.end,
            token,
        });
    }

    fn inner_parse(&mut self) -> Result<(), ParseError> {
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

    pub fn parse(&mut self) -> Result<(), ParseError> {
        match self.inner_parse() {
            Ok(_) => Ok(()),
            Err(error) => {
                println!("2{:?}", self.context.source);
                print_error(&self.context.source, &error, self.line, self.column, self.end);
                Err(error)
            }
        }
    }

    fn peek(&mut self) -> Result<u8, ParseError> {
        self.empty_check()?;
        Ok(self.context.source[self.index])
    }

    fn peek2(&mut self) -> Result<u8, ParseError> {
        self.empty_check2()?;
        Ok(self.context.source[self.index+1])
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
        Ok(self.context.source[self.index - 1])
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
            b'$' => self.parse_hex(),
            b'%' => self.parse_binary(),
            b'0'..=b'9' => self.parse_absolute_decimal(),
            b'#' => self.parse_sharp(),
            b'a'..=b'z' | b'A'..=b'Z' => self.parse_keyword(),
            b'.' => self.parse_directive(),
            b'"' => self.parse_string(),
            b';' => self.parse_comment(),
            b'=' => self.parse_assign(),
            b'(' => self.parse_open_parenthesis(),
            b')' => self.parse_close_parenthesis(),
            b',' => self.parse_comma(),
            b'\r' | b'\n' => self.parse_newline(),
            b' ' | b'\t' => self.parse_whitespace(),
            n => {
                println!("{}", n);
                Err(ParseError::UnknownToken)
            }
        }
    }

    fn parse_absolute_decimal(&mut self) -> Result<Token<'a>, ParseError> {
        
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

        let size = match decimal_number > 0xff_u16 {
            true => 2,
            false => 1
        };

        match size {
            1 => Ok(Token::Byte(decimal_number as u8)),
            2 => Ok(Token::Word(decimal_number as u16)),
            _ => Err(ParseError::InvalidNumberFormat)
        }
    }

    fn parse_hex(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat_expected(b'$', ParseError::InvalidNumberFormat)?;
    
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

        match count / 2 {
            1 => Ok(Token::Byte(hex_number as u8)),
            2 => Ok(Token::Word(hex_number as u16)),
            _ => Err(ParseError::InvalidNumberFormat)
        }
    }

    fn parse_binary(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat_expected(b'%', ParseError::InvalidNumberFormat)?;

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
        
        match count / 8 {
            1 => Ok(Token::Byte(binary_number as u8)),
            2 => Ok(Token::Word(binary_number as u16)),
            _ => Err(ParseError::InvalidNumberFormat)
        }

    }

    fn parse_open_parenthesis(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat_expected(b'(', ParseError::InvalidNumberFormat)?;
        Ok(Token::OpenParenthesis)
    }

    fn parse_close_parenthesis(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat_expected(b')', ParseError::InvalidNumberFormat)?;
        Ok(Token::CloseParenthesis)
    }

    fn parse_sharp(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat_expected(b'#', ParseError::InvalidNumberFormat)?;
        Ok(Token::Sharp)
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
                        b' ' | b',' | b')' | b'=' | b'\t' => break,
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
            return Ok(Token::Branch(&self.context.source[start..self.index - 1]));
        }

        if self.index - start == 3 {
            let search_insts: [u8; 3] = [upper_case_byte(self.context.source[start]), upper_case_byte(self.context.source[start + 1]), upper_case_byte(self.context.source[start + 2])];
            if let Some(position) = INSTS.iter().position(|item| *item == &search_insts) {
                return Ok(Token::Instr(position));
            }
        }

        Ok(Token::Keyword(&self.context.source[start..self.index]))
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
        Ok(Token::String(&self.context.source[start..self.index-1]))
    }

    fn parse_directive(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat_expected(b'.', ParseError::InvalidDirective)?;
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
                        b':' => {
                            branch = true;
                            break;
                        },
                        b' ' | b'\t' | b'\n' | b'\r' => break,
                        _ => return Err(ParseError::InvalidDirective),
                    };
                    self.eat()?;
                }
                Err(ParseError::OutOfScope) => break,
                _ => return Err(ParseError::InvalidDirective),
            };
        }

        if !valid {
            return Err(ParseError::InvalidDirective);
        }

        if branch {
            return Ok(Token::BranchNext(&self.context.source[start..self.index - 1]));
        }

        Ok(Token::Directive(&self.context.source[start..self.index]))
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
        Ok(Token::Comment(&self.context.source[start..self.index - 1]))
    }

    fn parse_assign(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat_expected(b'=', ParseError::UnexpectedSymbol)?;
        Ok(Token::Assign)
    }

    fn parse_comma(&mut self) -> Result<Token<'a>, ParseError> {
        self.eat_expected(b',', ParseError::UnexpectedSymbol)?;
        Ok(Token::Comma)
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

        info!("Tokens");
        print!("{:>5}. ", line);
        for ast in self.context.tokens.borrow().iter() {
            let type_name = match ast.token {
                Token::Instr(_) => "INSTR",
                Token::Keyword(_) => "KEYWORD",
                Token::Directive(_) => "DIRECTIVE",
                Token::Comment(_) => "COMMENT",
                Token::Branch(_) => "BRANCH",
                Token::Byte(_) => "BYTE",
                Token::Word(_) => "WORD",
                Token::OpenParenthesis => "(",
                Token::CloseParenthesis => ")",
                Token::Sharp => "#",
                Token::NewLine(_) => "NEWLINE",
                Token::Space(_) => "SPACE",
                Token::End => "END",
                Token::String(_) => "STRING",
                Token::BranchNext(_) => "BRANCHNEXT",
                Token::Assign => "ASSIGN",
                Token::Comma => "COMMA",
            };

            if ast.line != line {
                println!();
                line = ast.line;
                print!("{:>5}. ", line);
            }

            print!("[{:>2}:{:<2} {:^10}] ", ast.column, ast.end, type_name);
        }
        println!();
    }
}
