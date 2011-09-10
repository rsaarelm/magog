! Copyright (C) 2011 Risto Saarelma

USING: arrays ascii assocs combinators.short-circuit grouping kernel math
multiline parser quotations sequences splitting words ;

IN: dust.hexmap

<PRIVATE

CONSTANT: element-width 2

: blank-string? ( string -- ? ) [ blank? ] all? ;

: find-nonblank-index ( line -- i? ) [ blank? not ] find drop ;

: strip-heading-blanks ( lines -- lines' )
    dup { [ empty? not ] [ first blank-string? ] } 1&&
    [ rest strip-heading-blanks ] when ;

: text>lines ( text -- lines ) "\n" split strip-heading-blanks ;

: adjust-line ( line x -- line' ) odd? over empty? not and [ rest ] when ;

: line>cells ( line -- cells ) dup empty?
    [ drop { } ] [ element-width group ] if ;

: cell-rows ( lines -- x0 cell-rows )
    dup first find-nonblank-index
    swap [ pick + adjust-line line>cells ] map-index ;

: preprocess ( text -- x0 cell-rows )
    text>lines
    dup empty? [ drop 0 { } ] [ cell-rows ] if ;

: make-x1 ( x0 _ -- x0 _ x1 )
    over dup 0 < [ 1 - ] when element-width /i neg ;

: format-cell ( elt -- elt' )
    dup blank-string?
    [ drop f ]
    [ dup first blank? [ "Misaligned hex map" throw ] when
      [ blank? ] trim ] if ;

: add-cell ( y x1 assoc elt -- y x1 assoc )
    [ 2dup swap 2array ] 2dip swapd
    pick swapd set-at ;

: advance ( x1 assoc -- x1' assoc ) [ 1 + ] dip ;

: next-row ( x0 y x1 assoc -- x0' y' x1 assoc )
    [ [ 1 - ] [ 1 + ] [ drop make-x1 ] tri* ] dip ;

PRIVATE>

: parse-hex-map ( text -- assoc )
    preprocess [ 0 make-x1 H{ } clone ] dip
    [ [ format-cell [ add-cell ] when* advance ] each
      next-row ] each [ 3drop ] dip ;

SYNTAX: HEXMAP:
    CREATE-WORD parse-here parse-hex-map
    1quotation (( -- assoc )) define-inline ;