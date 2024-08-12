mod opcode;
mod parser;
mod code_gen;
mod ast;
#[cfg(test)]
mod tests;

use ast::AstGenerator;
use code_gen::CodeGenerator;
use parser::Parser;


fn main() {
    let data = br#"LDX #$08
decrement:
DEX
STX $0200
CPX #$03
BNE decrement
BNE decrement2
STX $0201
decrement2:
STX $0201
BRK"#;
    let mut parser = Parser::new(data);
    parser.parse().unwrap();
    parser.friendly_dump();

    let ast_generator = AstGenerator::new(parser.tokens);
    ast_generator.generate().unwrap();

    let generator = CodeGenerator::new(ast_generator.asts.take());
    generator.generate().unwrap();
    generator.dump();
}


