! Copyright (C) 2011 Risto Saarelma

USING: alien alien.c-types arrays combinators compiler.units cpu.x86.assembler
cpu.x86.assembler.operands fry kernel make math math.parser quotations
sequences vocabs.parser words ;

IN: dust.hline8

! Generate word scanline-switch ( col1 col2 ptr offset int -- ), which can
! draw any 8-pixel (specified by setting "int" to the 8-bit integer
! corresponding to the bit pattern of the line) two-color horizontal line
! into a memory buffer.

! XXX: Only implemented for 32-bit X86 assembly, other platforms will need a
! different implementation. A Factor version will be much slower.

! One complete scanline assembly routine, EAX and EBX vary in MOV instructions
! according to the bit pattern.

! void { int int void* int } cdecl [
!     EAX ESP     [] MOV ! color1
!     EBX ESP 4  [+] MOV ! color2
!     ECX ESP 8  [+] MOV ! pixel data pointer
!     ECX ESP 12 [+] ADD ! offset
!     ECX     [] EAX MOV
!     ECX 4  [+] EAX MOV
!     ECX 8  [+] EAX MOV
!     ECX 12 [+] EBX MOV
!     ECX 16 [+] EBX MOV
!     ECX 20 [+] EAX MOV
!     ECX 24 [+] EAX MOV
!     ECX 28 [+] EAX MOV
! ] alien-assembly ;

CONSTANT: hline8-bytes 32

<<

<PRIVATE

: scanline-asm-prefix ( -- quot )
    [ EAX ESP     [] MOV ! color1
      EBX ESP 4  [+] MOV ! color2
      ECX ESP 8  [+] MOV ! pixel data pointer
      ECX ESP 12 [+] ADD ! offset
    ] ;

: scanline-asm-pixelop ( bitmask n -- quot )
    [ nip 4 * ] [ bit? [ EAX ] [ EBX ] ? ] 2bi '[ ECX _ [+] @ MOV ] ;

: (scanline-asm) ( bitmask -- quot )
      [ scanline-asm-prefix %
        8 iota [ dupd scanline-asm-pixelop % ] each
      ] [ ] make nip ;

: scanline-asm ( bitmask -- quot )
    (scanline-asm) '[ void { int int void* int } cdecl _ alien-assembly ] ;

: scanline-name ( bitmask -- str ) [ "asm-scanline-" % # ] "" make ;

: name-define ( bitmask -- sym ) scanline-name current-vocab create ;

: name-symbol ( bitmask -- sym ) scanline-name current-vocab lookup ;

: define-scanline-word ( bitmask -- )
    dup name-define swap scanline-asm
    (( col1 col2 ptr offset -- )) define-declared ;

PRIVATE>

[   ! Define the 256 scanline words.
    256 iota [ define-scanline-word ] each

    ! Build a bit case-switch word for choosing the correct procedure for a
    ! bitmask.
    SYMBOL: scanline-switch
    scanline-switch
    [ 256 iota [ dup name-symbol 1array >quotation 2array ] map ,
      \ case , ] [ ] make
    (( col1 col2 ptr offset int -- )) define-declared
] with-compilation-unit
>>