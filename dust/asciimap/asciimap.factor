! Copyright (C) 2011 Risto Saarelma

USING: arrays ascii combinators.short-circuit kernel locals multiline
sequences splitting strings ;

IN: dust.asciimap

<PRIVATE

: blank-string? ( string -- ? ) [ blank? ] all? ;

: strip-heading-blanks ( lines -- lines' )
    dup { [ empty? not ] [ first blank-string? ] } 1&&
    [ rest strip-heading-blanks ] when ;

: text>lines ( text -- lines ) "\n" split strip-heading-blanks ;

: assoc-elt ( x y c -- elt )
    dup blank? [ 3drop f ] [ 1 swap <string> [ 2array ] dip 2array ] if ;

PRIVATE>

:: parse-ascii-map ( text -- assoc )
    text text>lines
    [| line y | line >array [| c x | x y c assoc-elt ] map-index ] map-index
    concat sift ;

SYNTAX: ASCIIMAP:
    parse-here parse-ascii-map suffix! ;
