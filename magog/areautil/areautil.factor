! Copyright (C) 2011 Risto Saarelma

USING: accessors arrays assocs combinators dust.com dust.gamestate dust.hex
kernel locals magog.com.area magog.com.loc magog.offset make math math.parser
math.vectors namespaces sequences ;

IN: magog.areautil

<PRIVATE

! Dim should always be even, given how we operate with it halved a lot here.

: hex-edge-offsets ( dim -- offsets )
    { { 1 1/2 } { 1/2 1 } { -1/2 1/2 } { -1 -1/2 } { -1/2 -1 } { 1/2 -1/2 } }
    [ over v*n ] map nip ;

!    o 1 1 2    n = 4   o 1 1 2
!   0 . . . 2           0 . . . 2
!  0 . . . . 2          0 . . . . 2
!   5 . . . 3             5 . . . 3
!    5 . . 3                5 . . 3
!     4 4 4    o: origin      4 4 4
!
!   hex view          same chunk, ortho view

:: hex-edge-points ( dim facet -- seq )
    dim 2 / facet even? 1 0 ? + iota
    facet {
        { 0 [ [| i | 0 i 2array ] map ] }
        { 1 [ [| i | i 1 + 0 2array ] map ] }
        { 2 [ [| i | dim 2 / 1 + i + i 2array ] map ] }
        { 3 [ [| i | dim 1 + dim 2 / 1 + i + 2array ] map ] }
        { 4 [ [| i | dim 2 / 1 + i + dim 1 + 2array ] map ] }
        { 5 [ [| i | 1 i + dim 2 / 1 + i + 2array ] map ] }
        [ 2drop { } ]
    } case ;

!           Rect chunk layout example
!
!          o 1 1 1 1   n = 4
!         5 . . . . 2  o: origin
!        5 . . . . 2   0-edge portal
!       5 . . . . 2    at origin
!      5 . . . . 2
!       4 4 4 4 3

! Rect chunks still have six neighbors, since two of the corner points have a
! diagonal connection.

: rect-edge-offsets ( dim -- offsets )
    { { 1 1 } { 0 1 } { -1 0 } { -1 -1 } { 0 -1 } { 1 0 } }
    [ over v*n ] map nip ;

:: rect-edge-points ( dim facet -- seq )
    dim iota
    facet {
        { 0 [ drop { { 0 0 } } ] }
        { 1 [ [| i | i 1 + 0 2array ] map ] }
        { 2 [ [| i | dim 1 + i 1 + 2array ] map ] }
        { 3 [ drop dim 1 + dup 2array 1array ] }
        { 4 [ [| i | i 1 + dim 1 + 2array ] map ] }
        { 5 [ [| i | 0 i 1 + 2array ] map ] }
        [ 2drop { } ]
    } case ;

PRIVATE>

: edge-portals ( area-facet dim edge-seq -- ) 3drop ; ! XXX Deprecated

! XXX: Repeated code.

:: rect-edge-portals ( area-facet dim adjacent-area-seq -- )
    dim rect-edge-offsets adjacent-area-seq zip
    [ first2 dup [ <site> ] [ 2drop f ] if ] map :> portals
    6 iota [ dim swap rect-edge-points ] map :> edges
    portals edges zip
    [ dup first
      [ first2 [ over swap { 1 1 } v- area-facet set-portal ] each drop ]
      [ drop ] if ] each ;

:: hex-edge-portals ( area-facet dim adjacent-area-seq -- )
    dim hex-edge-offsets adjacent-area-seq zip
    [ first2 dup [ <site> ] [ 2drop f ] if ] map :> portals
    6 iota [ dim swap hex-edge-points ] map :> edges
    portals edges zip
    [ dup first
      [ first2 [ over swap { 1 1 } v- area-facet set-portal ] each drop ]
      [ drop ] if ] each ;

<PRIVATE

SYMBOL: +chunk-loc+

SYMBOL: +area-uid+

: init-current-area ( -- )
    +area-uid+ get uid-in-use?
    [ +area-uid+ get dup register-uid <area-facet> add-facet ] unless ;

PRIVATE>

: area-name ( chunk-loc -- uid )
    reverse first3 [ "area:" % # "," % # "," % # ] "" make ;

: current-area ( -- area ) +area-uid+ get >area ;

: chunk-loc ( -- chunk-loc ) +chunk-loc+ get ;

: area-below ( -- uid ) chunk-loc { 0 0 -1 } v+ area-name ;

: area-above ( -- uid ) chunk-loc { 0 0 1 } v+ area-name ;

: neighbor-chunk-loc ( chunk-loc side -- chunk-loc' )
    { { -1 -1 0 } { 0 -1 0 } { 1 0 0 } { 1 1 0 } { 0 1 0 } { -1 0 0 } }
    nth v+ ;

: neighbor-chunk-uids ( chunk-loc -- neighbor-seq )
    6 iota [ over swap neighbor-chunk-loc area-name ] map nip ;

: current-rect-edge-portals ( dim -- )
    current-area swap chunk-loc neighbor-chunk-uids rect-edge-portals ;

: current-hex-edge-portals ( dim -- )
    current-area swap chunk-loc neighbor-chunk-uids hex-edge-portals ;

: terrain ( terrain loc -- )
    current-area terrain>> set-at ;

: terrain-at ( loc -- terrain )
    current-area terrain>> at ;

: portal ( delta-loc target-area loc -- )
    [ <site> ] dip current-area set-portal ;

: current-site ( loc -- site )
    +area-uid+ get <site> ;

: spawn ( mob-uid loc -- )
    current-site add-facet ;

: make-area ( chunk-loc quot -- )
    [
        [ +chunk-loc+ set ] dip
        chunk-loc area-name +area-uid+ set
        init-current-area
        call
    ] with-scope ; inline

: wall-mask ( site -- mask )
    neighbor-sites [ >terrain spread?>> 1 0 ? ] map
    [ 6 iota ] dip zip 0 [ first2 1 rot shift * + ] reduce ;

: wall-type ( site -- type )
    wall-mask hex-wall-type ;

: wallify-icon ( site -- icon )
    dup >terrain spread?>>
    [
        [ >terrain char>> ]
        [ wall-type ] bi +
    ]
    [
        >terrain char>>
    ] if ;