use rstest::*;

use crate::{ast::AstGenerator, code_gen::CodeGenerator, parser::Parser};

#[rstest]
#[case(br#"LDX #$08
decrement2:
STX $0201
decrement:
DEX
STX $0200
CPX #$03
BNE decrement
BNE decrement2
STX $0201
BRK"#)]
#[case(br#"LDA #$01
STA $0200
LDA #$05
STA $0201
LDA #$08
STA $0202"#)]
#[case(br#"LDA #$c0  ;Load the hex value $c0 into the A register
TAX       ;Transfer the value in the A register to X
INX       ;Increment the value in the X register
ADC #$c4  ;Add the hex value $c4 to the A register
BRK       ;Break - we're done"#)]
#[case(br#"
LDA #$80
STA $01
ADC $01
"#)]
#[case(br#"LDX #$08
decrement:
DEX
STX $0200
CPX #$03
BNE decrement
BNE decrement2
STX $0201
decrement2:
STX $0201
BRK"#)]
fn compile_test(#[case] data: &'_ [u8]) {
    let mut parser = Parser::new(data);
    parser.parse().unwrap();

    let ast_generator = AstGenerator::new(parser.tokens);
    ast_generator.generate().unwrap();

    let generator = CodeGenerator::new(ast_generator.asts.take());
    generator.generate().unwrap();
}

/*
  */
#[rstest]
#[case(br#"LDX #$08
decrement:
DEX
STX $0200
CPX #$03
BNE decrement
BNE decrement2
STX $0201
decrement2:
STX $0201
BRK"#, &[0xa2, 0x08, 0xca, 0x8e, 0x00, 0x02, 0xe0, 0x03, 0xd0, 0xf8, 0xd0, 0x03, 0x8e, 0x01, 0x02, 0x8e, 0x01, 0x02, 0x00])]
#[case(br#"LDX #$08
decrement2:
STX $0201
decrement:
DEX
STX $0200
CPX #$03
BNE decrement
BNE decrement2
STX $0201
BRK"#, &[0xa2, 0x08, 0x8e, 0x01, 0x02, 0xca, 0x8e, 0x00, 0x02, 0xe0, 0x03, 0xd0, 0xf8, 0xd0, 0xf3, 0x8e, 0x01, 0x02, 0x00])]
fn check_codes(#[case] data: &'_ [u8], #[case] codes: &'_ [u8]) {
    let mut parser = Parser::new(data);
    parser.parse().unwrap();

    let ast_generator = AstGenerator::new(parser.tokens);
    ast_generator.generate().unwrap();

    let generator = CodeGenerator::new(ast_generator.asts.take());
    generator.generate().unwrap();
    assert_eq!(generator.data.take(), codes);
}
