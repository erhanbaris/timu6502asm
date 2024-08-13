use rstest::*;

use crate::{opcode::ModeType, parser::{Parser, Token}};

#[rstest]
// Hex numbers
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
#[case(b"($a000)", Token::Number(0xa000, ModeType::Indirect))]
#[case(b"( $a000 )", Token::Number(0xA000, ModeType::Indirect))]

// Binary numbers
#[case(b"#%10100000", Token::Number(0xA0, ModeType::Immediate))]
#[case(b"%10100000", Token::Number(0xA0, ModeType::ZeroPage))]
#[case(b"%10100000,X", Token::Number(0xA0, ModeType::ZeroPageX))]
#[case(b"%10100000,Y", Token::Number(0xA0, ModeType::ZeroPageY))]
#[case(b"%10100000, x", Token::Number(0xA0, ModeType::ZeroPageX))]
#[case(b"%10100000, y", Token::Number(0xA0, ModeType::ZeroPageY))]
#[case(b"%1010000000000000", Token::Number(0xA000, ModeType::Absolute))]
#[case(b"%1010000000000000,X", Token::Number(0xA000, ModeType::AbsoluteX))]
#[case(b"%1010000000000000,Y", Token::Number(0xA000, ModeType::AbsoluteY))]
#[case(b"(%10100000,X)", Token::Number(0xA0, ModeType::IndirectX))]
#[case(b"(%10100000),Y", Token::Number(0xA0, ModeType::IndirectY))]
#[case(b"(%10100000, x)", Token::Number(0xA0, ModeType::IndirectX))]
#[case(b"(%10100000), y", Token::Number(0xA0, ModeType::IndirectY))]
#[case(b"(%1010000000000000)", Token::Number(0xa000, ModeType::Indirect))]
#[case(b"( %1010000000000000 )", Token::Number(0xA000, ModeType::Indirect))]

// Decimal numbers
#[case(b"#160", Token::Number(0xA0, ModeType::Immediate))]
#[case(b"160", Token::Number(0xA0, ModeType::ZeroPage))]
#[case(b"160,X", Token::Number(0xA0, ModeType::ZeroPageX))]
#[case(b"160,Y", Token::Number(0xA0, ModeType::ZeroPageY))]
#[case(b"160, x", Token::Number(0xA0, ModeType::ZeroPageX))]
#[case(b"160, y", Token::Number(0xA0, ModeType::ZeroPageY))]
#[case(b"40960", Token::Number(0xA000, ModeType::Absolute))]
#[case(b"40960,X", Token::Number(0xA000, ModeType::AbsoluteX))]
#[case(b"40960,Y", Token::Number(0xA000, ModeType::AbsoluteY))]
#[case(b"(160,X)", Token::Number(0xA0, ModeType::IndirectX))]
#[case(b"(160),Y", Token::Number(0xA0, ModeType::IndirectY))]
#[case(b"(160, x)", Token::Number(0xA0, ModeType::IndirectX))]
#[case(b"(160), y", Token::Number(0xA0, ModeType::IndirectY))]
#[case(b"(40960)", Token::Number(0xa000, ModeType::Indirect))]
#[case(b"( 40960 )", Token::Number(0xA000, ModeType::Indirect))]
fn number_check(#[case] data: &'_ [u8], #[case] token: Token<'_>) {
    let mut parser = Parser::new(data);
    parser.parse().unwrap();
    assert_eq!(parser.tokens.len(), 2);
    assert_eq!(parser.tokens[0].token, token);
    assert_eq!(parser.tokens[1].token, Token::End);
}

#[rstest]
#[case(b"#$a00", 3)]
#[case(b"#%123", 3)]
#[case(b"#%001", 3)]
#[case(b"#%00111", 3)]
#[case(b"#% 00111", 3)]
#[case(b"#%a00111", 3)]
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
#[case(b"($a0)", 0)]
#[case(b"($a000", 0)]
#[case(b"$a000)", 0)]
fn invalid_number_check(#[case] data: &'_ [u8], #[case] count: usize) {
    let mut parser = Parser::new(data);
    if let Ok(_) = parser.parse() {
        assert_eq!(parser.tokens.len(), count);
    }
}

#[rstest]
#[case(b";")]
#[case(b";hello world")]
#[case(b";\"test")]
#[case(b";''''''")]
#[case(b";;;;;;;;;;;;;")]
fn check_comment(#[case] data: &'_ [u8]) {
    let mut parser = Parser::new(data);
    parser.parse().unwrap();
    assert_eq!(parser.tokens.len(), 2);
    if let Token::Comment(_) = parser.tokens[0].token {
        return
    }

    panic!("Comment not parsed")
}
