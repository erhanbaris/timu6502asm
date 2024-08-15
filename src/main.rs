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

use std::fs::File;
use std::io::prelude::*;

fn main() {
    let _ = CombinedLogger::init(vec![TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)]);
    info!("timu6502asm Compiler");

    let data = br#"
PRG_COUNT       = 1 
.byte PRG_COUNT, PRG_COUNT, PRG_COUNT"#;

    let context = Context::new(data);

    let mut parser = Parser::new(context);
    parser.parse().unwrap();
    parser.friendly_dump();

    let context = parser.context;

    let ast_generator = AstGenerator::new();
    let context = ast_generator.generate(context).unwrap();

    let mut generator = CodeGenerator::new();
    let context = generator.generate(context).unwrap();
    generator.dump(&context);

    let mut file = File::create("tables.bin").unwrap();
    file.write_all(&context.target).unwrap();
 
}
