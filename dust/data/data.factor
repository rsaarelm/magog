! Copyright (C) 2011 Risto Saarelma

USING: colors colors.constants combinators continuations grouping kernel math
math.parser regexp sequences unicode.data ;

IN: dust.data

<PRIVATE

CONSTANT: r-#rrggbb R/ #\p{xdigit}{6}/
CONSTANT: r-#rgb R/ #\p{xdigit}{3}/
CONSTANT: r-0xcc R/ 0x\p{xdigit}{2,4}/

: parse-rgb ( rgb-string -- color )
    {
        { [ dup r-#rrggbb matches? ]
          [ 1 tail 2 group [ hex> ] map ] }
        { [ dup r-#rgb matches? ]
          [ 1 tail 1 group [ dup first prefix hex> ] map ] }
        [ no-such-color ]
    } cond [ 255.0 / ] map first3 1.0 <rgba> ;

PRIVATE>

! Parse a named color, a #rgb hex color or a #rrggbb hex color into a rgba
! tuple.
: parse-color ( string -- color )
    [ named-color ] [ drop parse-rgb ] recover ;

! Parse a literal or a 0xcc or a 0xcccc hex encoded char into an ascii or
! unicode number.
: parse-char ( string -- char )
    {
        { [ dup length 1 = ] [ first ] }
        { [ dup r-0xcc matches? ] [ 2 tail hex> ] }
        { [ name>char dup ] [ ] }
        [ "Invalid char string" throw ]
    } cond ;