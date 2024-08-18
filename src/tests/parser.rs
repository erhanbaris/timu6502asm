use std::path::PathBuf;

use rstest::*;

use crate::{ast::AstGenerator, context::Context, parser::{Parser, Token}};

#[rstest]
// Hex numbers
#[case(b"$a0", 0xa0)]
#[case(b"$a000", 0xa000)]

// Binary numbers
#[case(b"%10100000", 0xa0)]
#[case(b"%1010000000000000", 40960)]

// Decimal numbers
#[case(b"160", 0xa0)]
fn number_check(#[case] data: &'_ [u8], #[case] expected: u16) {
    let context = Context::default();
    let path = PathBuf::from("main.asm");
    context.add_file(0, path);
  
    let mut parser = Parser::new(0, data, context);
    parser.parse().unwrap();
    assert_eq!(parser.context.tokens.borrow().len(), 2);
    match parser.context.tokens.borrow()[0].token {
        Token::Byte(current) => assert_eq!(current, expected as u8),
        Token::Word(current) => assert_eq!(current, expected),
        _ => panic!("Unexpected token")
    }
    assert_eq!(parser.context.tokens.borrow()[1].token, Token::End);
}

#[rstest]
#[case(b"#$a00")]
#[case(b"#%123")]
#[case(b"#%001")]
#[case(b"#%00111")]
#[case(b"#% 00111")]
#[case(b"#%a00111")]
#[case(b"#$")]
#[case(b"#$1")]
#[case(b"$a01")]
#[case(b"$a0111")]
#[case(b"$a")]
#[case(b"$ta000")]
#[case(b"$a000-,X")]
#[case(b"($a0,X")]
#[case(b"$a0),Y")]
#[case(b"$a0 , Y)")]
#[case(b"$a0  Y)")]
#[case(b"($a0)")]
#[case(b"($a000")]
#[case(b"$a000)")]
fn invalid_number_check(#[case] data: &'_ [u8]) {
    let context = Context::default();
    let path = PathBuf::from("main.asm");
    context.add_file(0, path);
    context.code_files.borrow_mut()[0].data = data.to_vec();
  
    let mut parser = Parser::new(0, data, context);

    if let Ok(_) = parser.parse() {
        let ast_generator = AstGenerator::new();
        ast_generator.generate(parser.context).unwrap_err();
    }
}

#[rstest]
#[case(b";")]
#[case(b";hello world")]
#[case(b";\"test")]
#[case(b";''''''")]
#[case(b";;;;;;;;;;;;;")]
fn check_comment(#[case] data: &'_ [u8]) {
    let context = Context::default();
    let path = PathBuf::from("main.asm");
    context.add_file(0, path);
  
    let mut parser = Parser::new(0, data, context);
    parser.parse().unwrap();
    assert_eq!(parser.context.tokens.borrow().len(), 2);
    if let Token::Comment(_) = parser.context.tokens.borrow()[0].token {
        return
    }

    panic!("Comment not parsed")
}
