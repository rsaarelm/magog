! Copyright (C) 2011 Risto Saarelma

USING: accessors arrays assocs byte-arrays colors colors.constants dust.geom
fry grouping images images.atlas images.normalization kernel locals math
math.bitwise math.functions math.rectangles math.vectors sequences
sequences.deep ;

IN: dust.img

: inverse-color ( color -- color' )
    >rgba-components [ [ 1.0 swap - ] tri@ ] dip <rgba> ;

<PRIVATE

: atlas>locdim-coords ( atlas-coord -- loc dim )
    first4 -rot swap 4array 2 group first2 over v- ;

:: intersect-blit-rect ( dst-bounds dst-loc src-bounds rect -- intersect-rect )
    rect src-bounds rect-intersect
    dst-loc offset-rect dst-bounds rect-intersect
    dst-loc vneg offset-rect ;

:: flip-y? ( y src-img dst-img -- y' )
    src-img upside-down?>> dst-img upside-down?>> not and
    [ src-img dim>> second y - 1 - ]
    [ y ] if ;

:: (blit-img) ( src-img src-loc src-dim dst-img dst-loc -- )
    src-dim [ 0 = ] any? [
        src-dim first2 :> ( blit-w blit-h )
        src-loc first2 :> ( src-x src-y )
        dst-img dim>> first2 :> ( w h )
        dst-loc first2 :> ( dst-x dst-y )
        blit-h iota [| row |
            src-x src-y row + src-img dst-img flip-y?
            blit-w src-img pixel-row-slice-at

            dst-x dst-y row + dst-img src-img flip-y?
            blit-w dst-img set-pixel-row-at
        ] each
    ] unless ;
PRIVATE>

:: blit-img ( src-img src-loc src-dim dst-img dst-loc -- )
    src-loc src-dim <rect> :> src-rect
    { 0 0 } src-img dim>> <rect> :> src-bounds
    { 0 0 } dst-img dim>> <rect> :> dst-bounds
    dst-bounds dst-loc src-loc v- src-bounds src-rect intersect-blit-rect
    rect-bounds :> ( src-loc src-dim )
    src-img src-loc src-dim dst-img dst-loc (blit-img) ;

:: clone-empty-image ( dim exemplar-image -- image )
   <image>
        dim >>dim
        exemplar-image component-order>> >>component-order
        exemplar-image component-type>> >>component-type
        exemplar-image upside-down?>> >>upside-down?
        dim product exemplar-image bytes-per-pixel * <byte-array> >>bitmap ;

:: sub-img ( loc dim src-img -- image )
    dim src-img clone-empty-image :> dst-img
    src-img loc dim dst-img { 0 0 } blit-img
    dst-img ;

:: tile-images ( src-image tile-dim -- images )
    { 0 0 } src-image dim>> tile-dim v/ [ floor ] map rect-iota
    [ tile-dim v* tile-dim src-image sub-img ] map ;

: loc-dim-atlas ( images -- loc-dim-coords atlas-image )
    dup make-atlas
    [ swap [ over at atlas>locdim-coords 2array ] map nip ] dip ;

:: flip-if-upside-down ( image -- image )
    image upside-down?>> [
        image dim>> image clone-empty-image :> result
        image { 0 0 } image dim>> result { 0 0 } blit-img
        result ]
        [ image ] if ;

<PRIVATE

: power-of-2? ( x -- ? ) bit-count 2 < ;

:: (clone-pow2-dim-image) ( image -- pow2-dim-image )
    image normalize-image :> image
    image dim>> [ next-power-of-2 ] map image clone-empty-image :> dst-img
    image { 0 0 } image dim>> dst-img { 0 0 } blit-img
    dst-img ;

PRIVATE>

: clone-pow2-dim-image ( image -- pow2-dim-image )
    dup dim>> [ power-of-2? ] all?
    [ normalize-image ]
    [ (clone-pow2-dim-image) ] if ;

<PRIVATE

: >rgb24 ( col -- rgb24 )
    >rgba-components drop [ 255 * >integer ] tri@ 3byte-array ;

: map-alpha ( rgb-image rgb->rgba-quot -- rgba-image )
    swap
    [ 4 group swap map flatten >byte-array ] change-bitmap ; inline

: color-key-mapping ( color-key -- rgb->rgba-quot )
    >rgb24 '[ first3 3byte-array dup _ = [ 0 ] [ 255 ] if suffix ] ; inline

PRIVATE>

: color-key-alpha ( image color-key -- image' )
    [ normalize-image ] dip color-key-mapping map-alpha ;

<PRIVATE

:: make-rgba-image ( dim -- image )
    <image>
        dim >>dim
        RGBA >>component-order
        ubyte-components >>component-type
        dim product 4 * <byte-array> >>bitmap ;

: pixels-rect-iota ( image -- seq ) [ { 0 0 } ] dip dim>> rect-iota ;

: >pixel ( color -- pixel )
    >rgba-components 4array 255 v*n [ >integer ] map >byte-array ;

: >color ( pixel -- color )
    >array [ 255 / ] map first4 <rgba> ;

PRIVATE>

:: filter-image ( image pixel-quot: ( loc color -- color ) -- )
    image pixels-rect-iota
    [ first2 :> ( x y )
      x y 2array x y image pixel-at >color pixel-quot call
      >pixel x y image set-pixel-at ] each ; inline

: gen-image ( dim pixel-quot: ( loc color -- color ) -- image )
    [ make-rgba-image dup ] dip filter-image ; inline

: halftone-filter ( loc color -- color )
    drop first2 + 2 mod 0 = [ "white" ] [ "black" ] if named-color ;

: color-intensity ( color -- a )
    >rgba-components drop 3array { 0.2989 0.5870 0.1140 } v. ;

: image-white-to-alpha ( image -- image' )
    normalize-image dup
    [ nip color-intensity dup dup dup <rgba> ] filter-image ;

: (clerp) ( t -- v_t ) [ 4 ] dip [ ] curry replicate ;

: clerp ( a b t -- a_t )
    [ [ >rgba-components 4array ] bi@ ] dip (clerp)
    vlerp first4 <rgba> ;