# timu6502asm

[![codecov](https://codecov.io/gh/erhanbaris/timu6502asm/graph/badge.svg?token=GWS9VMW347)](https://codecov.io/gh/erhanbaris/timu6502asm)
![Build](https://github.com/erhanbaris/timu6502asm/actions/workflows/rust.yml/badge.svg)

Yet another 6502 assembler project. The goal is make a multi platform (include web) compiler generator. Project is still in very early stage and there is no easy way to use it. You can check the code or wait to get more usable version.

## Building
timu6502 builded with latest Rust Language. You have to install Rust Language. After installetion execute ```cargo build --release``` command. The executable will be located under _target/release/_ folder.
Compiler tested under Windows and MacOS operating system. It should work under Linux OS but not yet tested.


## Usage
timu6502 is terminal based compiler. So, basic usage is:
```bash
timu6502asm test.asm --target test.bin
timu6502asm test.asm --binary-dump
timu6502asm test.asm --token-dump
timu6502asm test.asm --token-dump --slient
timu6502asm --help
```
If the compilation operation failed, process exit code will be **1** and print error descriptions if silent mode is off.

## Branches
Basically, branches is referencing the location at the execution code. If you want to jump location, it is hard to calculate and remember the address, but, with branches you just need to remember branch name and the compiler will be assign address automatically.

Example:
```assembly
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
    BRK
```
As you can see in the example there are **init**, **loop** and **end** branches defined and used with the instruction code.
Also, compiler has a support for local branches.
```assembly
branch1:
    @local1:
        INX

    @local2:
        INY
    jump @local1

branch2:
    @local1:
        DEX

    @local2:
        DEY
    jump @local1
```

## Const
You can define consts and use it with instruction.

Example:
```assembly
const1 = $10
const2 = 22
const3 = %11001100

CPX #const1
```

## Data types
Compiler works with primative data types.

### Byte
It takes up one byte of space. It is written in three different ways depending on the number type.
Examples:
```assembly
$01       ; in hexadecimal format
$CC       ; in hexadecimal format

%00000000 ; in binary format
%01010011 ; in binary format

128       ; in decimal format
2         ; in decimal format
```

### Word
It takes up two bytes of space. It is written in three different ways depending on the number type.
Examples:
```assembly
$0122             ; in hexadecimal format
$CC33             ; in hexadecimal format

%0000000000000000 ; in binary format
%0101001100000000 ; in binary format

123456            ; in decimal format
888888            ; in decimal format
```

### Ascii
It takes up different sizes of space depending on the definition. The text must be written between double quotes.
```assembly
"Hello world"
```

## Available directives

### .org
Change reference locations. It is not changing where the codes are stored, it is changing jump and branch references.
```assembly
.org $0600
.byte $11
```
```
0600: 11
```

### .byte
Define byte sized data. Must be followed by a sequence of (byte ranged) expressions or strings.

```assembly
.byte $11
.byte $22, $33
.byte "Hello"
```
```
0000: 11 22 33 48 65 6C 6C 6F
```

### .word
Write 1 or many word information into memory
```assembly
.word $1122
.word $3344, $5566
```
```
0000: 22 11 44 33 66 55
```

### .ascii
Write ascii information into memory. Also, byte directive can be used.
```assembly
.ascii "hello world"
```
```
0000: 68 65 6C 6C 6F 20 77 6F
0008: 72 6C 64
```

### .asciiz
Write ascii information into memory. If there is no 0x00 at end of the string, compiler will add 0x00.
```assembly
.asciiz "hello world"
```
```
0000: 68 65 6C 6C 6F 20 77 6F
0008: 72 6C 64 00
```

### .incbin
Include a file as binary data.
```assembly
.incbin "src/tests/bins/test1.bin"
```
```
0000: 00 01 02 03
```

### .warning
Print warning message on compilation time.
```assembly
.warning "timu6502asm compiler works"
```
```
22:05:16 [WARN] timu6502asm compiler works
```

### .fail
The compilation process stops with an error message.
```assembly
.fail "Unsupported platform"
```

### .include
Import another assembly file. All variable defitions will be imported and could be accessible from other files.
```assembly
.include "header.asm"
.include "body.asm"
.include "footer.asm"
```

### .pad
Fill memory from the current address to a specified address.  A fill value may also be specified.
```assembly
.pad $0600
```

### .fillvalue
Change the default filler for **.pad**.
```assembly
.fillvalue $ff
```

### .dsb
 Define storage bytes. The size argument may be followed by a fill value (default filler is 0).
```assembly
.dsb $05 ; same as .byte $00, $00, $00, $00, $00
.dsb $05, $11 ; same as .byte $11, $11, $11, $11, $11
```

### .dsw
 Define storage words. The size argument may be followed by a fill value (default filler is 0).
```assembly
.dsw $05 ; same as .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
.dsw $05, $1122 ; same as .byte $22, $11, $22, $11, $22, $11, $22, $11, $22, $11
```

There are many things to do. Here are the some todos:
 - [X] Case insensitivity
 - [X] Binary file generation
 - [ ] Decompiler
 - [X] Human friendly prints
 - [X] Import different asm files
 - [ ] Performance measurement
 - [ ] Documentation
 - [ ] Deploy on real hardware/emulator
 - [ ] (stretch goal) Basic high-level programming language
 - [ ] (stretch goal) Basic emulator
