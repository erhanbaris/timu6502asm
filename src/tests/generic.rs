use std::{fs::File, io::Read, path::PathBuf};

use rstest::*;

use crate::{
    ast::{AstGenerator, InstrInfo, InstrValue, InstrInfoRegister},
    code_gen::{CodeGenerator, CodeGeneratorError},
    context::Context,
    parser::Parser,
};

#[rstest]
#[case(
    br#"
VAR = "\""    
LDX #$08
decrement2:
STX $0201
decrement:
DEX
STX $0200
CPX #$03
BNE decrement
BNE decrement2
STX $0201
BRK"#
)]
#[case(
    br#"LDA #$01
STA $0200
LDA #$05
STA $0201
LDA #$08
STA $0202"#
)]
#[case(
    br#"LDA #$c0  ;Load the hex value $c0 into the A register
TAX       ;Transfer the value in the A register to X
INX       ;Increment the value in the X register
ADC #$c4  ;Add the hex value $c4 to the A register
BRK       ;Break - we're done"#
)]
#[case(
    br#"
LDA #$80
STA $01
ADC $01
"#
)]
#[case(
    br#"LDX #$08
decrement:
DEX
STX $0200
CPX #$03
BNE decrement
BNE decrement2
STX $0201
decrement2:
STX $0201
BRK"#
)]
fn compile_test(#[case] data: &'_ [u8]) {
    let context = Context::default();
    let path = PathBuf::from("main.asm");
    context.add_file(0, path);
    context.code_files.borrow_mut()[0].data = data.to_vec();

    let mut parser = Parser::new(0, data, context);
    parser.parse().unwrap();
    parser.friendly_dump();

    let context = parser.context;

    let ast_generator = AstGenerator::new();
    let context = ast_generator.generate(context).unwrap();

    let mut generator = CodeGenerator::new();
    let context = generator.generate(context).unwrap();
    generator.dump(&context);
}

#[rstest]
#[case(br#"#$08"#, InstrInfo { value: InstrValue::Byte(0x08), is_immediate: true, in_parenthesis: false, register: InstrInfoRegister::None })]
#[case(br#"#$0008"#, InstrInfo { value: InstrValue::Byte(0x08), is_immediate: true, in_parenthesis: false, register: InstrInfoRegister::None })]
#[case(br#"$08"#,  InstrInfo { value: InstrValue::Byte(0x08), is_immediate: false, in_parenthesis: false, register: InstrInfoRegister::None })]
#[case(br#"$0800"#,  InstrInfo { value: InstrValue::Word(0x0800), is_immediate: false, in_parenthesis: false, register: InstrInfoRegister::None })]
#[case(br#"($0800)"#, InstrInfo { value: InstrValue::Word(0x0800), is_immediate: false, in_parenthesis: true, register: InstrInfoRegister::None })]
#[case(br#"($0008, X)"#, InstrInfo { value: InstrValue::Byte(0x08), is_immediate: false, in_parenthesis: true, register: InstrInfoRegister::X })]
#[case(br#"($0008) , Y"#, InstrInfo { value: InstrValue::Byte(0x08), is_immediate: false, in_parenthesis: true, register: InstrInfoRegister::Y })]
#[case(br#"( $08, X ) "#, InstrInfo { value: InstrValue::Byte(0x08), is_immediate: false, in_parenthesis: true, register: InstrInfoRegister::X })]
#[case(br#"( $08 ) , Y "#, InstrInfo { value: InstrValue::Byte(0x08), is_immediate: false, in_parenthesis: true, register: InstrInfoRegister::Y })]
#[case(br#"( test ) , Y "#, InstrInfo { value: InstrValue::Reference("test".to_string()), is_immediate: false, in_parenthesis: true, register: InstrInfoRegister::Y })]
#[case(br#"(test),Y"#, InstrInfo { value: InstrValue::Reference("test".to_string()), is_immediate: false, in_parenthesis: true, register: InstrInfoRegister::Y })]
#[case(br#"#test"#, InstrInfo { value: InstrValue::Reference("test".to_string()), is_immediate: true, in_parenthesis: false, register: InstrInfoRegister::None })]
#[case(br#"(test)"#, InstrInfo { value: InstrValue::Reference("test".to_string()), is_immediate: false, in_parenthesis: true, register: InstrInfoRegister::None })]
#[case(br#"test"#, InstrInfo { value: InstrValue::Reference("test".to_string()), is_immediate: false, in_parenthesis: false, register: InstrInfoRegister::None })]
fn number_parsing_test(#[case] data: &'_ [u8], #[case] expected: InstrInfo) {
    let context = Context::default();
    let path = PathBuf::from("main.asm");
    context.add_file(0, path);
    context.code_files.borrow_mut()[0].data = data.to_vec();

    let mut parser = Parser::new(0, data, context);
    parser.parse().unwrap();
    parser.friendly_dump();

    let context = parser.context;

    let ast_generator = AstGenerator::new();
    ast_generator.size.set(context.tokens.borrow().len());
    let info = ast_generator.parse_instr_value(&context).unwrap();
    assert_eq!(info, expected);
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
#[case(br#"IOSAVE          = $FF4A ; save the A, X, and Y registers
IOREST          = $FF3F ; restore the A, X, and Y registers

lda IOSAVE
LDx IOREST"#, &[0xad, 0x4a, 0xff, 0xae, 0x3f, 0xff])]
#[case(br#".ascii "abcd""#, &[0x61, 0x62, 0x63, 0x64])]
#[case(br#".asciiz "abcd""#, &[0x61, 0x62, 0x63, 0x64, 0x00])]
#[case(br#".byte $ff"#, &[0xFF])]
#[case(br#".byte $11, $22,$33,$44"#, &[0x11, 0x22, 0x33, 0x44])]
#[case(br#".word $2211, $4433,$6655, $8877"#, &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88])]
#[case(br#".byte $ff
.asciiz "abcd""#, &[0xFF, 0x61, 0x62, 0x63, 0x64, 0x00])]
#[case(br#".word $ccff"#, &[0xff, 0xcc])]
#[case(br#".word $ff"#, &[0xff, 0x00])]
#[case(br#"AND #$dd"#, &[0x29, 0xdd])]
#[case(br#"AND #$ffdd"#, &[0x29, 0xdd])]
#[case(br#"AND $dd"#, &[0x25, 0xdd])]
#[case(br#"AND $ffdd"#, &[0x2d, 0xdd, 0xff])]
#[case(br#"AND ($ff, x)"#, &[0x21, 0xff])]
#[case(br#"AND ($00ff, x)"#, &[0x21, 0xff])]
#[case(br#"AND ($ff),Y"#, &[0x31, 0xff])]
#[case(br#"LDX $ff,Y"#, &[0xb6, 0xff])]
#[case(br#"AND $ff,x"#, &[0x35, 0xff])]
#[case(br#"AND $ffdd , x"#, &[0x3d, 0xdd, 0xff])]
#[case(br#"LDX $ffdd , y"#, &[0xBE, 0xdd, 0xff])]
#[case(br#"JMP ($ffdd)"#, &[0x6c, 0xdd, 0xff])] // Only jump has indirect mode
#[case(br#"LDX #$08
decrement2:
    STX $0201
@decrement:
    DEX
    STX $0200
    CPX #$03
    BNE @decrement
    BNE decrement2
    STX $0201
    BRK"#, &[0xA2, 0x08, 0x8E, 0x01, 0x02, 0xCA, 0x8E, 0x00, 0x02, 0xE0, 0x03, 0xD0, 0xF8, 0xD0, 0xF3, 0x8E, 0x01, 0x02, 0x00])]
#[case(br#"var1 = $10
var2 = 22
var3 = %11001100

CPX #var1"#, &[0xe0, 0x10])]
#[case(br#".dsb 5"#, &[0x00, 0x00, 0x00, 0x00, 0x00])]
#[case(br#".dsb 5 , $10"#, &[0x10, 0x10, 0x10, 0x10, 0x10])]
#[case(br#".dsw 5"#, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])]
#[case(br#".dsw 5 , $1122"#, &[0x22, 0x11, 0x22, 0x11, 0x22, 0x11, 0x22, 0x11, 0x22, 0x11])]
fn check_codes(#[case] data: &'_ [u8], #[case] codes: &'_ [u8]) {
    let context = Context::default();
    let path = PathBuf::from("main.asm");
    context.add_file(0, path);
    context.code_files.borrow_mut()[0].data = data.to_vec();

    let mut parser = Parser::new(0, data, context);
    parser.parse().unwrap();
    parser.friendly_dump();

    let context = parser.context;

    let ast_generator = AstGenerator::new();
    let context = ast_generator.generate(context).unwrap();

    let mut generator = CodeGenerator::new();
    let context = generator.generate(context).unwrap();
    generator.dump(&context);
    assert_eq!(context.target, codes);
}

#[rstest]
#[case(br#".INCBIN "src/tests/bins/test1.bin""#, &[0x00, 0x01, 0x02, 0x03])]
fn binary_read(#[case] data: &'_ [u8], #[case] binary: &'_ [u8]) {
    let context = Context::default();
        let path = PathBuf::from("main.asm");
    context.add_file(0, path);
    context.code_files.borrow_mut()[0].data = data.to_vec();

    let mut parser = Parser::new(0, data, context);
    parser.parse().unwrap();
    parser.friendly_dump();

    let context = parser.context;

    let ast_generator = AstGenerator::new();
    let context = ast_generator.generate(context).unwrap();

    let mut generator = CodeGenerator::new();
    let context = generator.generate(context).unwrap();
    generator.dump(&context);
    assert_eq!(context.target, binary);
}

#[rstest]
#[case(br#"init :"#)]
#[case(br#"1-1 :"#)]
#[case(br#"- :"#)]
#[case(br#"= :"#)]
#[case(br#"? :"#)]
fn parser_fail(#[case] data: &'_ [u8]) {
    let context = Context::default();
        let path = PathBuf::from("main.asm");
    context.add_file(0, path);
    context.code_files.borrow_mut()[0].data = data.to_vec();

    let mut parser = Parser::new(0, data, context);
    assert!(parser.parse().is_err());
}

#[rstest]
#[case(br#".INCBIN"#)]
#[case(br#"BNE"#)]
#[case(br#"BNE BNE"#)]
#[case(br#"BNE 11111"#)]
#[case(br#"BNE "Hello""#)]
#[case(br#"BNE  = "Hello""#)]
#[case(br#".fBNE  = "Hello""#)]
#[case(br#"AND ($0008) , x"#)]
#[case(br#"AND ($0008 , Y)"#)]
#[case(br#"
VAR = 1
VAR = 1
"#)]
fn ast_generator_fail(#[case] data: &'_ [u8]) {
    let context = Context::default();
        let path = PathBuf::from("main.asm");
    context.add_file(0, path);
    context.code_files.borrow_mut()[0].data = data.to_vec();

    let mut parser = Parser::new(0, data, context);
    parser.parse().unwrap();
    parser.friendly_dump();

    let context = parser.context;

    let ast_generator = AstGenerator::new();
    assert!(ast_generator.generate(context).is_err());
}
#[rstest]
#[case(br#"AND ($ffdd)"#)]
fn compile_failure(#[case] data: &'_ [u8]) {
    let context = Context::default();
        let path = PathBuf::from("main.asm");
    context.add_file(0, path);
    context.code_files.borrow_mut()[0].data = data.to_vec();

    let mut parser = Parser::new(0, data, context);
    parser.parse().unwrap();
    parser.friendly_dump();

    let context = parser.context;

    let ast_generator = AstGenerator::new();
    let context = ast_generator.generate(context).unwrap();

    let mut generator = CodeGenerator::new();
    match generator.generate(context).unwrap_err() {
        CodeGeneratorError::IllegalOpcode => (),
        _ => {
            panic!("Invalid error code");
        }
    };
}

#[rstest]
#[case("src/tests/asms/tables.asm", "src/tests/bins/tables.bin")]
#[case("src/tests/asms/import-test.asm", "src/tests/bins/import-test.bin")]
fn test_file(#[case] code_filename: &str, #[case] expected_filename: &str) {
    let mut code = Vec::new();
    let mut file = File::open(code_filename).unwrap();
    file.read_to_end(&mut code).unwrap();

    let mut binary = Vec::new();
    let mut file = File::open(expected_filename).unwrap();
    file.read_to_end(&mut binary).unwrap();

    let context = Context::default();
    let path = PathBuf::from(code_filename);
    context.add_file(0, path);
    context.code_files.borrow_mut()[0].data = code.clone();

    let mut parser = Parser::new(0, &code, context);
    parser.parse().unwrap();

    let context = parser.context;

    let ast_generator = AstGenerator::new();
    let context = ast_generator.generate(context).unwrap();

    let mut generator = CodeGenerator::new();
    let context = generator.generate(context).unwrap();
    assert_eq!(context.target, binary);
}

#[rstest]
#[case("src/tests/asms/fail-test.asm")]
fn fail_test(#[case] code_filename: &str) {
    let mut code = Vec::new();
    let mut file = File::open(code_filename).unwrap();
    file.read_to_end(&mut code).unwrap();

    let context = Context::default();
    let path = PathBuf::from(code_filename);
    context.add_file(0, path);
    context.code_files.borrow_mut()[0].data = code.clone();

    let mut parser = Parser::new(0, &code, context);
    parser.parse().unwrap();

    let context = parser.context;

    let ast_generator = AstGenerator::new();
    let context = ast_generator.generate(context).unwrap();

    let mut generator = CodeGenerator::new();
    assert!(generator.generate(context).is_err());
}

#[rstest]
#[case(br#"@decrement:"#)]
#[case(br#"LDX #$08
decrement2:
    STX $0201
@decrement:
    DEX
    STX $0200
    CPX #$03
    BNE @decrement
    BNE decrement2
    STX $0201
    BRK"#)]
fn local_branch_test(#[case] data: &'_ [u8]) {
    let context = Context::default();
    let path = PathBuf::from("main.asm");
    context.add_file(0, path);
    context.code_files.borrow_mut()[0].data = data.to_vec();

    let mut parser = Parser::new(0, data, context);
    parser.parse().unwrap();
    parser.friendly_dump();

    let context = parser.context;

    let ast_generator = AstGenerator::new();
    let context = ast_generator.generate(context).unwrap();

    let mut generator = CodeGenerator::new();
    let context = generator.generate(context).unwrap();
    generator.dump(&context);
}