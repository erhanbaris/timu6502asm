mod opcode;
mod parser;
mod code_gen;
mod ast;
mod options;
#[cfg(test)]
mod tests;

use ast::AstGenerator;
use code_gen::CodeGenerator;
use parser::Parser;


fn main() {
    let data = br#".INCBIN "test""#;
    let mut parser = Parser::new(data);
    parser.parse().unwrap();
    parser.friendly_dump();

    let ast_generator = AstGenerator::new(parser.tokens);
    ast_generator.generate().unwrap();

    let generator = CodeGenerator::new(ast_generator.asts.take());
    generator.generate().unwrap();
    generator.dump();
}


