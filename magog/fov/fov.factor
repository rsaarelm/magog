! Copyright (C) 2011 Risto Saarelma

USING: arrays assocs dust.hex fry kernel locals math math.functions math.order
math.ranges namespaces sequences magog.com.loc ;

IN: magog.fov

<PRIVATE
SYMBOL: +visible-set+
SYMBOL: +current-center+
SYMBOL: +range+

! Offsetted site in un-portaled space of the current center location.
: site ( loc -- site )
    +current-center+ get translate-site ;

: portal ( loc -- portal/f ) site get-portal ;

: mark-seen ( loc -- )
    dup [ [ site ] [ portal ] bi portal-translate ] dip
    +visible-set+ get set-at ;

: blocks? ( loc -- ? ) site dup get-portal portal-translate
    blocks-sight? ;

: similar? ( loc1 loc2 -- ? )
    [ [ blocks? ] [ portal ] bi 2array ] bi@ = ;

: enter-portal ( portal -- )
    +current-center+ get swap portal-translate
    +current-center+ set ;


: circumference ( u -- n ) 6 * ;

: index>angle ( i u -- angle ) [ 1/2 - ] dip circumference / ;

: start-angle>index ( angle u -- i ) circumference * 1/2 + floor >integer ;

: end-angle>index ( angle u -- i ) circumference * 1/2 + ceiling >integer ;


: (arc-points) ( u angle-range -- i1 i2 )
    [ first2 pick swap [ start-angle>index ] [ end-angle>index ] 2bi* ]
    [ 0 swap circumference ] if* ;

: arc-points ( u angle-range -- seq )
    over [ (arc-points) [a,b) ] dip hex-circle
    swap [ over nth ] map nip ;


: range-clamp ( x angle-range -- y ) [ first2 clamp ] when* ;


: begin-end-index-pairs ( start-index seq-seq -- pair-seq )
    [ length 2dup dupd + 2array [ + ] dip ] map nip ;

: index-pairs-assoc ( start-index seq-seq -- pairs-assoc )
    [ begin-end-index-pairs ] keep zip ;


: similar-slice ( loc-seq -- similar-slice tail-slice )
    dup empty? [ f ]
    [ dup dup first [ similar? not ] curry find drop
      [ cut-slice ] [ f ] if* ] if ;

: clump-similar ( loc-seq -- loc-seq-seq )
    [ dup empty? not ] [ similar-slice swap ] produce nip ;

: indexes-to-angles ( idx-assoc u angle-range -- angles-assoc )
    '[ [ [ _ index>angle _ range-clamp ] map ] dip ] assoc-map ;

:: segments ( u angle-range -- angles-assoc )
    angle-range [ first u start-angle>index ] [ 0 ] if*
    u angle-range [ arc-points ] [ hex-circle ] if* clump-similar
    index-pairs-assoc u angle-range indexes-to-angles ;


:: process-arc ( u angle-range -- )
    u +range+ get < [
        u angle-range segments [| sector locs |
            locs first blocks? [
                locs first portal
                [ [ enter-portal
                    u 1 + sector process-arc ] with-scope
                ]
                [ u 1 + sector process-arc ] if*
            ] unless
            locs [ mark-seen ] each
        ] assoc-each
    ] when ;

PRIVATE>

:: fov ( center range -- visibles )
    [ range +range+ set
      center +current-center+ set
      H{ } clone +visible-set+ set
      { 0 0 } mark-seen
      1 f process-arc
      +visible-set+ get ] with-scope ;
