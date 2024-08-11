mod opcode;
mod parser;
mod code_gen;

use code_gen::Generator;
use parser::Parser;


fn main() {
    let data = br#"
LDA #$c0  
TAX       
INX       
ADC #$c4  
BRK       
"#;
    let mut parser = Parser::new(data);
    parser.parse().unwrap();

    println!("Asts: {:#?}", parser.asts);

    let mut generator = Generator::new();
    generator.generate(parser.asts);
    generator.dump();
}
