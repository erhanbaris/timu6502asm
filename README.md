# timu6502asm

[![codecov](https://codecov.io/gh/erhanbaris/timu6502asm/graph/badge.svg?token=GWS9VMW347)](https://codecov.io/gh/erhanbaris/timu6502asm)
![Build](https://github.com/erhanbaris/timu6502asm/actions/workflows/rust.yml/badge.svg)

Yet another 6502 Asm compiler project. The goal is make a multi platform (include web) compiler generator. Project is still in very early stage and there is no easy way to use it. You can check the code or wait to get more usable version.

Example code what compiler can compile now.
```assembly
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
    BRK
```

Expected output:
```
0600: 20 09 06 20 0c 06 20 12 06 a2 00 60 e8 e0 05 d0
0610: fb 60 00
```

## Data types
Compiler works with primative date types. 

### Byte
It takes up one byte of space. It is written in three different ways depending on the number type.
Examples:
```assembly
$01       ; in decimal format
$CC       ; in decimal format

%00000000 ; in binary format
%01010011 ; in binary format

128       ; in decimal format
2         ; in decimal format
```

### Word
It takes up two bytes of space. It is written in three different ways depending on the number type.
Examples:
```assembly
$0122             ; in decimal format
$CC33             ; in decimal format

%0000000000000000 ; in binary format
%0101001100000000 ; in binary format

123456            ; in decimal format
888888            ; in decimal format
```

### Ascii
It takes up different sizes of space depending on the definition. The text must be written between double quotes.
```assembly
"Hello world" ; in decimal format
```

## Available directives

### .org
Change reference locations. It is not changing where the codes are stored, it is changing jump and branch references.
```assembly
.ORG $0600
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
.byte $1122
.byte $3344, $5566
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
.warning "timu6502asm compiler works partial"
```
```
22:05:16 [WARN] timu6502asm compiler works partial
```

### .include
Import another file.
```assembly
.include "header.asm"
.include "body.asm"
.include "footer.asm"
```
```
22:05:16 [WARN] timu6502asm compiler works partial
```

There are many things to do. Here are the some todos:
 - [ ] Case insensitivity
 - [ ] Rom file generation
 - [ ] Decompiler
 - [ ] Human friendly prints
 - [ ] Import different asm files
 - [ ] Performance measurement
 - [ ] Documentation
 - [ ] Deploy on real hardware
 - [ ] (stretch goal) Basic high-level programming language
 - [ ] (stretch goal) Basic emulator
