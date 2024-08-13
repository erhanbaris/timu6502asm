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
#[case(br#"LDX #$01   ;X is $01
LDA #$aa   ;A is $aa
STA $a0,X ;Store the value of A at memory location $a1
INX        ;Increment X
STA $a0,X ;Store the value of A at memory location $a2"#, &[0xa2, 0x01, 0xa9, 0xaa, 0x95, 0xa0, 0xe8, 0x95, 0xa0])]
#[case(br#"LDA #$01
CMP #$02
BNE notequal
STA $22
notequal:
BRK"#, &[0xa9, 0x01, 0xc9, 0x02, 0xd0, 0x02, 0x85, 0x22, 0x00])]
#[case(br#"LDA #$01
STA $f0
LDA #$cc
STA $f1
JMP ($00f0) ;dereferences to $cc01"#, &[0xa9, 0x01, 0x85, 0xf0, 0xa9, 0xcc, 0x85, 0xf1, 0x6c, 0xf0, 0x00])]
#[case(br#"LDX #$00
    LDY #$00
  firstloop:
    TXA
    STA $0200,Y
    PHA
    INX
    INY
    CPY #$10
    BNE firstloop ;loop until Y is $10
  secondloop:
    PLA
    STA $0200,Y
    INY
    CPY #$20      ;loop until Y is $20
    BNE secondloop"#, &[0xa2, 0x00, 0xa0, 0x00, 0x8a, 0x99, 0x00, 0x02, 0x48, 0xe8, 0xc8, 0xc0, 0x10, 0xd0, 0xf5, 0x68, 0x99, 0x00, 0x02, 0xc8, 0xc0, 0x20, 0xd0, 0xf7])]
#[case(br#"  LDA #$03
JMP there
BRK
BRK
BRK
there:
STA $0200"#, &[0xa9, 0x03, 0x4c, 0x08, 0x00, 0x00, 0x00, 0x00, 0x8d, 0x00, 0x02])]
#[case(br#"  JSR init
JSR loop
JSR end

init:
LDX #$00
RTS

loop:
INX
CPX #$05
BNE loop
RTS

end:
BRK"#, &[0x20, 0x09, 0x00, 0x20, 0x0c, 0x00, 0x20, 0x12, 0x00, 0xa2, 0x00, 0x60, 0xe8, 0xe0, 0x05, 0xd0, 0xfb, 0x60, 0x00])]
#[case(br#"
.ORG $0600 ; change location

JSR init
JSR loop
JSR end

init:
LDX #$00
RTS

loop:
INX
CPX #$05
BNE loop
RTS

end:
BRK"#, &[0x20, 0x09, 0x06, 0x20, 0x0c, 0x06, 0x20, 0x12, 0x06, 0xa2, 0x00, 0x60, 0xe8, 0xe0, 0x05, 0xd0, 0xfb, 0x60, 0x00])]
fn check_codes(#[case] data: &'_ [u8], #[case] codes: &'_ [u8]) {
    let mut parser = Parser::new(data);
    parser.parse().unwrap();

    let ast_generator = AstGenerator::new(parser.tokens);
    ast_generator.generate().unwrap();

    let generator = CodeGenerator::new(ast_generator.asts.take());
    generator.generate().unwrap();
    assert_eq!(generator.data.take(), codes);
}
