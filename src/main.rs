mod opcode;
mod parser;
mod code_gen;
mod ast;
mod directive;
mod tool;
mod context;
#[cfg(test)]
mod tests;

use std::{fs::File, io::{Read, Write}, path::PathBuf};

use log::{error, info, LevelFilter};
use simplelog::*;

use ast::{AstGenerator, AstGeneratorError};
use code_gen::{CodeGenerator, CodeGeneratorError};
use context::Context;
use parser::{ParseError, Parser};

use clap::{arg, command, Parser as ClapParser};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StarterError {
    #[error("IO Error: ({0})")]
    IOError(#[from] std::io::Error),
    
    #[error("{0}")]
    Generation(#[from] CodeGeneratorError),
    
    #[error("{0}")]
    Parser(#[from] ParseError),
    
    #[error("{0}")]
    Ast(#[from] AstGeneratorError),

    #[error("Please specify on of the argument [--target, --binary_dump, --token_dump]")]
    InvalidArgument
}

#[derive(ClapParser)]
#[command(version, about, long_about = None)]
struct Cli {

    /// Source .asm file
    #[arg(value_name = "SOURCE-FILE")]
    source: PathBuf,

    /// Target binary
    #[arg(long, value_name = "TARGET-FILE")]
    target: Option<PathBuf>,

    /// Dump binary
    #[clap(long, short='b', action)]
    binary_dump: bool,

    /// Dump tokens
    #[clap(long, short, action)]
    token_dump: bool,

    /// Silent mode
    #[clap(long, short, action)]
    silent: bool,
}


fn read_file(path: PathBuf) -> Result<Vec<u8>, StarterError> {
    let mut file = File::open(&path)?;    
    let mut code = Vec::new();
    file.read_to_end(&mut code)?;
    Ok(code)
}

fn execute(cli: &Cli) -> Result<(), StarterError> {
    if !cli.binary_dump && !cli.token_dump && cli.target.is_none() {
        return Err(StarterError::InvalidArgument);
    }

    if !cli.silent {
        info!("timu6502asm Compiler");
    }

    let mut context = Context::default();
    context.silent = cli.silent;
    
    if !cli.silent {
        info!("Compiling {:?}", &cli.source.as_os_str());
    }

    let data = read_file(cli.source.clone())?;

    context.add_file(0, cli.source.clone());
    context.code_files.borrow_mut()[0].data = data.clone();

    let mut parser = Parser::new(0, &data, context);
    parser.parse()?;
    if cli.token_dump {
        parser.friendly_dump();
    }

    let context = parser.context;

    let ast_generator = AstGenerator::new();
    let context = ast_generator.generate(context)?;

    let mut generator = CodeGenerator::new();
    generator.silent = cli.silent;

    let context = generator.generate(context)?;

    if cli.binary_dump {
        generator.dump(&context); 
    }

    if let Some(target) = &cli.target {
        let mut file = File::create(target)?;
        file.write_all(&context.target)?;
    }

    if !cli.silent {
        info!("Compilation successfully finished. ");
    }

    Ok(())
}

fn main() {
    let _ = CombinedLogger::init(vec![TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)]);

    let cli: Cli = Cli::parse();

    if let Err(error) = execute(&cli) {
        if !cli.silent {
            error!("Compilation failed.");
            error!("Reason: {}", error);
        }
        
        std::process::exit(1);
    }
}
