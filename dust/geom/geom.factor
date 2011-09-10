! Copyright (C) 2010 Risto Saarelma

USING: arrays combinators fry kernel locals math math.compare math.constants
math.functions math.order math.rectangles math.vectors sequences ;

IN: dust.geom

<PRIVATE
: point-pairs ( p1 p2 -- x1 x2 y1 y2 ) [ first2 ] bi@ swapd ;

: range-subset? ( r1 r2 -- ? )
    point-pairs [ >= ] [ <= ] 2bi* and ;

: ranges-overlap? ( r1 r2 -- ? )
    [ range-subset? ] [ swap range-subset? ] 2bi or ;

: overlap-offsets ( r target-range -- a b )
    [ [ first ] bi@ swap - ] [ [ second ] bi@ swap - ] 2bi ;

: overlap-offset ( r target-range -- x )
    2dup ranges-overlap? [ 2drop 0 ] [ overlap-offsets absmin ] if ;

: overlap-range ( r target-range -- r2 ) dupd overlap-offset v+n ;

: axis-decompose ( rect -- x-range y-range )
    rect-extent point-pairs [ 2array ] 2bi@ ;

: axis-compose ( x-range y-range -- rect )
    point-pairs [ 2array ] 2bi@ <extent-rect> ;
PRIVATE>

: overlap-rect ( rect target-rect -- newrect )
    [ axis-decompose ] bi@ swapd [ overlap-range ] 2bi@ axis-compose ;

: resize-rect ( rect offset-rect -- newrect )
    [ rect-bounds ] bi@ swapd [ v+ ] [ nip ] 2bi* <rect> ;

: vec-scale-rect ( rect scale-vec -- newrect )
    [ rect-bounds ] dip dup swapd [ v- ] [ 2 v*n v+ ] 2bi* <rect> ;

<PRIVATE
: floor-above-1 ( x -- y ) dup 1.0 < [ floor ] unless ;

: (fit-rect) ( canvas-dim rect-dim scale-quot -- dim )
    '[ dup -rot v/ first2 min _ call v*n ] call ; inline

: center-rect ( container-dim rect-dim -- pos )
    v- 2 v/n ;

: (scale-to-fit) ( screen-dim rect-dim scale-quot -- dim )
    '[ over swap v/ first2 min _ call [ 2.0 swap n/v ] dip v*n ] call ; inline
PRIVATE>

: scale-rect-into ( container-dim rect-dim -- dim )
    [ ] (fit-rect) ;

: scale-rect-center ( container-dim rect-dim -- loc dim )
    dupd scale-rect-into dup [ center-rect ] dip ;

: integer-scale-rect-into ( container-dim rect-dim -- dim )
    [ floor-above-1 ] (fit-rect) ;

: integer-scale-rect-center ( container-dim rect-dim -- loc dim )
    dupd integer-scale-rect-into dup [ center-rect ] dip ;

! Do a scaling that ensures that the given rect, when centered, will fit the
!  screen. Will try to scale the rect up by integer multiples if possible.
! Preserves aspect ratio of rect.

: integer-scale ( container-dim rect-dim -- dim )
    [ floor-above-1 ] (scale-to-fit) ;

<PRIVATE
: (rect-iota) ( loc dim i -- loc dim i-loc )
    [ dup first ] dip swap /mod swap 2array pick v+ ;
PRIVATE>

: rect-iota ( loc dim -- seq )
    dup product iota [ (rect-iota) ] map [ 2drop ] dip ;

: v-90 ( v -- v+90 ) first2 neg swap 2array ; inline

<PRIVATE
! Return scalar along q1-q2 on the line segment intersection. Return -1 if it
! looks like there would be a divide by zero otherwise, caller should
! interpret this as merely "not intersecting", not as a valid extrapolation
! point.
:: (line-segment-intersect) ( p1 p2 q1 q2 -- qv h )
    q2 q1 v- :> qv
    p2 p1 v- v-90 :> p
    qv p v. :> denom
    qv denom 0 = [ -1 ] [ p1 q1 v- p v. denom / ] if ;
PRIVATE>

:: line-segment-intersect ( p1 p2 q1 q2 -- loc? )
    p1 p2 q1 q2 (line-segment-intersect) :> ( qv h )
    q1 q2 p1 p2 (line-segment-intersect) :> ( pv j )
    h 0 1 between? j 0 1 between? and [ q1 qv h v*n v+ ] [ f ] if ;

: v-zero? ( v -- ? ) norm-sq 0 number= ;

! Replace null vector with x-axis unit vector. For geometry code that breaks
! with zero vectors, such as getting a vector's angle.
: unzero-v ( v -- u ) dup v-zero? [ length { 1 } resize-array ] when ;

: largest-elt ( v -- n ) -1/0. [ max ] accumulate drop ;

! Normalize a vector so that its largest component is unit length. In other
! words, scale the vector onto the surface of an axis-aligned cube instead of
! on an unit sphere.
: cube-normalize ( u -- v ) unzero-v dup vabs largest-elt v/n ;

:: line-points ( p1 p2 -- seq )
    p2 p1 v- :> dir
    dir cube-normalize :> step
    dir vabs largest-elt floor 1 + :> n
    n iota p1 [ drop step v+ ] accumulate nip ;

: dir4>vec ( dir4 -- vec )
    {
        { 0 [ {  0 -1 } ] }
        { 1 [ {  1  0 } ] }
        { 2 [ {  0  1 } ] }
        { 3 [ { -1  0 } ] }
    } case ;

:: rect-edge ( loc dim dir4 -- p1 p2 )
    { { 0 0 } { 1 0 } { 1 1 } { 0 1 } { 0 0 } } :> corners
    dir4 corners nth dir4 1 + corners nth [ dim v* loc v+ ] bi@ ;

: vec2-angle ( vec -- rad ) first2 rect> arg ;

: rotate-vec ( vec angle -- vec' )
    [ first2 rect> ] [ 0 swap rect> exp ] bi* *
    >rect 2array ;

: rad>deg ( rad -- deg ) 180 * pi / ;

: point-set-bounding-box ( vec-seq -- loc dim )
    dup empty? [ drop { 0 0 } { 0 0 } ]
    [ [ unclip [ vmin ] reduce ] [ unclip [ vmax ] reduce ] bi
      dupd swap v- ] if ;