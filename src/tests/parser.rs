use rstest::*;

use crate::{opcode::ModeType, parser::{Parser, Token}};

#[rstest]
#[case(b"#$a0", Token::Number(0xA0, ModeType::Immediate))]
#[case(b"$a0", Token::Number(0xA0, ModeType::ZeroPage))]
#[case(b"$a0,X", Token::Number(0xA0, ModeType::ZeroPageX))]
#[case(b"$a0,Y", Token::Number(0xA0, ModeType::ZeroPageY))]
#[case(b"$a0, x", Token::Number(0xA0, ModeType::ZeroPageX))]
#[case(b"$a0, y", Token::Number(0xA0, ModeType::ZeroPageY))]
#[case(b"$a000", Token::Number(0xA000, ModeType::Absolute))]
#[case(b"$a000,X", Token::Number(0xA000, ModeType::AbsoluteX))]
#[case(b"$a000,Y", Token::Number(0xA000, ModeType::AbsoluteY))]
#[case(b"($a0,X)", Token::Number(0xA0, ModeType::IndirectX))]
#[case(b"($a0),Y", Token::Number(0xA0, ModeType::IndirectY))]
#[case(b"($a0, x)", Token::Number(0xA0, ModeType::IndirectX))]
#[case(b"($a0), y", Token::Number(0xA0, ModeType::IndirectY))]
fn number_check(#[case] data: &'_ [u8], #[case] token: Token<'_>) {
    let mut parser = Parser::new(data);
    parser.parse().unwrap();
    assert_eq!(parser.tokens.len(), 2);
    assert_eq!(parser.tokens[0].token, token);
    assert_eq!(parser.tokens[1].token, Token::End);
}

#[rstest]
#[case(b"#$a00", 3)]
#[case(b"#$", 0)]
#[case(b"#$1", 0)]
#[case(b"$a01", 3)]
#[case(b"$a0111", 3)]
#[case(b"$a", 0)]
#[case(b"$a0,b", 0)]
#[case(b"$a0,", 0)]
#[case(b"$ta000", 0)]
#[case(b"$a000-,X", 0)]
#[case(b"($a0,X", 0)]
#[case(b"$a0),Y", 0)]
#[case(b"$a0 , Y)", 0)]
#[case(b"$a0  Y)", 0)]
fn invalid_number_check(#[case] data: &'_ [u8], #[case] count: usize) {
    let mut parser = Parser::new(data);
    if let Ok(_) = parser.parse() {
        assert_eq!(parser.tokens.len(), count);
    }
}
