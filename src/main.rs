mod opcode;
mod parser;
mod code_gen;
mod ast;
mod directive;
mod tool;
mod context;
#[cfg(test)]
mod tests;

use log::{info, LevelFilter};
use simplelog::*;

use ast::AstGenerator;
use code_gen::CodeGenerator;
use context::Context;
use parser::Parser;

fn main() {
    let _ = CombinedLogger::init(vec![TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)]);
    info!("timu6502asm Compiler");

    let data = br#"
    .include "src/tests/asms/tables.asm"
    .include "test2.asm"
    ADC TEST
    "#;

    let context = Context::default();
    context.add_file(0, "<MEMORY>".to_string());

    let mut parser = Parser::new(0, data, context);
    parser.parse().unwrap();
    parser.friendly_dump();

    let context = parser.context;

    let ast_generator = AstGenerator::new();
    let context = ast_generator.generate(context).unwrap();

    let mut generator = CodeGenerator::new();
    let context = generator.generate(context).unwrap();
    //generator.dump(&context); 
}
