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

There are many things to do. Here are the some todos:
 - [ ] Case insensitivity
 - [ ] Generate binary
 - [ ] Decompile binaries
 - [ ] Human friendly prints
 - [ ] Import different asm files
 - [ ] Performance measurement
 - [ ] Documentation
 - [ ] Deploy on real hardware
 - [ ] (stretch goal) Basic high-level programming language
 - [ ] (stretch goal) Basic emulator
