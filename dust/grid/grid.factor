! Copyright (C) 2010 Risto Saarelma

USING: arrays kernel math.functions math.ranges math.rectangles math.vectors
sequences sequences.product ;

IN: dust.grid

: 2nth ( loc grid -- elt ) [ first2 ] dip nth nth ;

: grid>world ( grid-loc tile-dim -- world-loc ) v* ;

: world>grid ( world-loc tile-dim -- tile-loc ) v/ ;

<PRIVATE

: expand-snap-rect ( rect -- newrect )
    rect-extent [ [ floor ] map ] [ [ ceiling ] map ] bi* <extent-rect> ;

: rect-ranges ( rect -- x-range y-range )
    rect-extent [ first2 ] bi@ swapd [ [a,b) ] 2bi@ ;

: scale-rect ( rect dim -- newrect )
    [ rect-bounds ] dip [ v* ] curry bi@ <rect> ;

: descale-rect ( rect dim -- newrect )
    [ rect-bounds ] dip [ v/ ] curry bi@ <rect> ;

PRIVATE>

: rect-points ( rect -- seq )
    expand-snap-rect rect-ranges 2array <product-sequence> ;

: grid-intersections ( world-rect tile-dim -- seq )
    descale-rect rect-points ;

: tile-rect ( loc tile-dim -- rect )
    [ { 1 1 } <rect> ] dip scale-rect ;