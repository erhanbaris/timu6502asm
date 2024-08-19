.include "sub-file.asm"

.fillvalue $00
.pad $0020

.fillvalue $11
.pad $0040

.include "sub-2-file.asm"

.fillvalue $22
.pad $0060

.warning "test warning"