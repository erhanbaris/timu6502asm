mod opcode;
mod parser;
mod code_gen;
mod ast;
mod options;
mod tool;
mod context;
#[cfg(test)]
mod tests;

use ast::AstGenerator;
use code_gen::CodeGenerator;
use context::Context;
use parser::Parser;


fn main() {
    let data = br#"; to assemble: vasm6502_oldstyle -Fbin -dotdir clock.s -o clock.out

    .org    $6000

; kernel routines
IOSAVE          = $FF4A ; save the A, X, and Y registers
IOREST          = $FF3F ; restore the A, X, and Y registers

; kernal addresses
KYBD            = $C000 ; keyboard
KBSTROBE        = $C010 ; keyboard strobe to clear the keyboard register
IRQ_VECTOR_L    = $03FE
IRQ_VECTOR_H    = $03FF
INT_ENABLE      = $C05C ; sets annuciater 2 low

; constants
LEFT_ARROW      = $88   ; keyboard left arrow
RIGHT_ARROW     = $95   ; keyboard right arrow
CLOCK_X_OFFSET  = 1     ; clock offset on x-axis
CLOCK_Y_OFFSET  = 8     ; clock offset on y-axis
TICKS_PER_MIN   = 95    ; ticks per minute from pendulum clock

; zero page addresses
tmp             = $1D   ; general purpose for storing temporary address (2 bytes)
row_ptr         = $1F   ; pointer to a row address in screen memory (2 bytes)
char_x          = $21   ; x position of the number to draw (1 byte)
char_y          = $22   ; y position of the number to draw (1 byte)
stor_x          = $23   ; (1 byte)
stor_y          = $24   ; (1 byte)
ticks           = $25   ; counter for pendulum clock ticks (1 byte)
blink           = $26   ; on/off toggle for hours/minutes separator (1 byte)
hours           = $27   ; the hours part of the current time (1 byte)
minutes         = $28   ; the minutes part of the current time (1 byte)

;=======================================================================
; Wait for key press
; M: increases minutes
; H: increases hours
; Any other key exits
;=======================================================================
main_loop:      bidt     KYBD                ; wait for a key press to adjust time
                bpl     main_loop
                lda     KYBD"#;

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
}


