! Copyright (C) 2011 Risto Saarelma

QUALIFIED-WITH: magog.tile tile

QUALIFIED: sets

USING: accessors arrays assocs combinators combinators.short-circuit dust.geom
fry hash-sets kernel literals locals magog.areautil magog.gen-world.chunks
magog.gen-world.spawn magog.rules math math.ranges math.vectors memoize
namespaces random sequences ;

IN: magog.gen-world

<PRIVATE

CONSTANT: north     { -1 -1  0 }
CONSTANT: northeast {  0 -1  0 }
CONSTANT: southeast {  1  0  0 }
CONSTANT: south     {  1  1  0 }
CONSTANT: southwest {  0  1  0 }
CONSTANT: northwest { -1  0  0 }
CONSTANT: up        {  0  0  1 }
CONSTANT: down      {  0  0 -1 }

CONSTANT: horizontal-6-dirs { $ north $ northeast $ southeast $ south
                              $ southwest $ northwest }

CONSTANT: dirs { $ northeast $ southeast $ southwest
                 $ northwest $ up $ down }

CONSTANT: starting-loc { 0 0 0 }

: opposite ( dir -- opposite ) -1 v*n ;

: north-edge ( chunk -- edge ) { 0 0 } swap at ;

: ne-edge ( chunk -- edge ) chunk-dim first iota
    [ 1 + 0 2array over at [ " " ] unless* ]
    map concat nip ;

: se-edge ( chunk -- edge ) chunk-dim second iota
    [ 1 + chunk-dim first 1 + swap 2array over at [ " " ] unless* ]
    map concat nip ;

: south-edge ( chunk -- edge ) chunk-dim { 1 1 } v+ swap at ;

: sw-edge ( chunk -- edge ) chunk-dim second iota
    [ 1 + chunk-dim second 1 + 2array over at [ " " ] unless* ]
    map concat nip ;

: nw-edge ( chunk -- edge ) chunk-dim second iota
    [ 1 + 0 swap 2array over at [ " " ] unless* ]
    map concat nip ;

: char-locs ( chunk char -- loc/f )
    '[ nip _ = ] assoc-filter keys ;

! Generate a loc array of the down-stairs or up-stairs locations. Matching
! chunks must have the corresponding stairs in the exact same pattern.

! XXX: Munging the patterns to same for now, since we actually want the up and
! down stairs to have an off-by-one relation, and can't think of a simple way
! to spec the equality relation right now.

: munge ( vert-edge -- vert-edge' ) dup empty? [ drop { 1 } ] unless ;

: up-edge ( chunk -- edge ) "y" char-locs munge ;

: down-edge ( chunk -- edge ) "z" char-locs munge ;

MEMO: edge ( chunk dir -- edge ) {
    { north [ north-edge ] }
    { northeast [ ne-edge ] }
    { southeast [ se-edge ] }
    { south [ south-edge ] }
    { southwest [ sw-edge ] }
    { northwest [ nw-edge ] }
    { up [ up-edge ] }
    { down [ down-edge ] }
  } case ;

: in-chunk? ( loc -- ? ) {
    [ { 0 0 } v>= [ ] all? ]
    [ chunk-dim v< [ ] all? ] } 1&& ;

: strip-annotations ( chunk -- chunk' )
    [ [ { 1 1 } v- ] dip ] assoc-map
    [ drop in-chunk? ] assoc-filter ;

! Vertical edges are represented by arrays of stair points instead of strings.
: open-vertical-edge? ( edge -- ? ) [ array? ] [ empty? not ] bi and ;

: open-horizontal-edge? ( edge -- ? ) [ CHAR: 1 = ] any? ;

! Edges that are up-down are always
: open? ( edge -- ? )
    { [ open-vertical-edge? ]
      [ open-horizontal-edge? ] } 1|| ;

: vertical-edge-open? ( chunk -- ? )
    { [ up edge open? ] [ down edge open? ] } 1|| ;

TUPLE: slot edges ;

: <slot> ( -- slot ) H{ } clone slot boa ;

TUPLE: mapgen placed-nodes slots ;

: <mapgen> ( -- mapgen )
    H{ } clone H{ } clone mapgen boa
    <slot> starting-loc pick slots>> set-at ;

SYMBOL: +mapgen+

: get-mapgen ( -- mapgen ) +mapgen+ get ;

: clear? ( loc -- ? ) get-mapgen placed-nodes>> key? not ;

:: set-initial-edges ( slot loc -- )
    dirs [| dir |
        loc dir v+ get-mapgen placed-nodes>> at
        [ dir opposite edge dir slot edges>> set-at ] when*
    ] each ;

! If there is a slot at loc, return it. If loc is empty, generate one and
! return it. If loc has already a chunk in it, return f.
:: gen-slot ( loc -- slot/f )
    loc clear?
    [ loc get-mapgen slots>> at
      [ <slot> dup loc get-mapgen slots>> set-at
        dup loc set-initial-edges ] unless*
    ] [ f ] if ;

:: add-slots ( chunk loc -- )
    dirs [| dir |
        chunk dir edge :> edge
        edge open?
        [ loc dir v+ gen-slot :> slot
          slot [ edge dir opposite slot edges>> set-at ] when
        ] when
    ] each ;

: slots ( -- slots ) get-mapgen slots>> ;

: level-slots ( z -- slots ) slots [ drop third over = ] assoc-filter nip ;

PRIVATE>

: with-mapgen ( quot -- )
    [ <mapgen> +mapgen+ set call ] with-scope ; inline

: new-chunk-sites ( -- locs ) slots keys ;

:: fits-in-slot? ( chunk slot -- ? )
    slot [
        slot edges>> >alist [
            first2 :> ( dir e )
            chunk dir edge e = ] all?
    ] [ f ] if ;

:: paint-chunk ( chunk loc -- )
    loc third :> z
    loc [ chunk strip-annotations [| loc char |
        char {
            { "." [ z 0 = tile:grass tile:floor ? loc terrain ] }
            { "#" [ z 0 = tile:tree tile:wall ? loc terrain ] }
            { "*" [ z 0 = tile:rock tile:wall ? loc terrain ] }
            { "~" [ tile:water loc terrain ] }
            { "<" [ tile:slope0 loc terrain ] }
            { "P" [ tile:slope1 loc terrain ] }
            { "A" [ tile:slope2 loc terrain ] }
            { ">" [ tile:slope3 loc terrain ] }
            { "J" [ tile:slope4 loc terrain ] }
            { "V" [ tile:slope5 loc terrain ] }
            { "z" [ tile:floor loc terrain
                    { 0 0 } area-below loc portal ] }
            { "y" [ tile:floor loc terrain
                    { 0 0 } area-above loc portal ] }
            [ drop ]
        } case
    ] assoc-each
    chunk-dim first current-rect-edge-portals
    ] make-area ;

: place-chunk ( chunk loc -- ) {
    [ get-mapgen placed-nodes>> set-at ]
    [ get-mapgen slots>> delete-at drop ]
    [ add-slots ]
    [ paint-chunk ] } 2cleave ;

: starting-chunk ( -- chunk ) 0 chunks nth ;

: upstairs-chunk ( -- chunk ) 7 chunks nth ;

: downstairs-chunk ( -- chunk ) 8 chunks nth ;

: random-slot ( z -- loc slot ) level-slots >alist
    [ f f ] [ random first2 ] if-empty ;

: level-chunks ( z -- chunk-assoc )
    get-mapgen placed-nodes>> [ drop third over = ] assoc-filter nip ;

:: level-border ( z -- locs )
    z level-chunks :> chunks
    chunks keys
    [ horizontal-6-dirs [ over v+ ] map nip ] map concat <hash-set>
    chunks keys sets:diff >array ;

: open-cells ( loc -- seq )
    [ { 0 0 } chunk-dim rect-iota [ terrain-at can-walk-terrain? ] filter ]
    make-area ;

: 1-chance-in ( n -- ? ) random 0 = ;

: on-cells ( loc n quot -- ) [ open-cells ] 2dip [ sample ] dip each ; inline

:: spawn-mobs ( loc -- )
    loc [
        loc third :> z
        {
            { [ z -6 < 8 1-chance-in and ]
              [ loc 2 random [ totem-guardian swap spawn ] on-cells ] }
            { [ z -4 < 6 1-chance-in and ]
              [ loc 3 random [ golem swap spawn ] on-cells ] }
            { [ z -1 < 5 1-chance-in and ]
              [ loc 5 random [ thrall swap spawn ] on-cells ] }
            { [ 4 1-chance-in ]
              [ loc 6 random [ dreg swap spawn ] on-cells ] }
            [ ]
        } cond
    ] make-area ;

:: ground-chunk ( z -- )
    z random-slot :> ( loc slot )
    chunks [ slot fits-in-slot? ] filter
    [ vertical-edge-open? not ] filter
    [ random loc place-chunk loc spawn-mobs ] unless-empty ;

:: stairwell ( z -- )
    z random-slot drop :> upstairs-loc
    upstairs-chunk upstairs-loc place-chunk
    downstairs-chunk upstairs-loc { 0 0 -1 } v+ place-chunk
    ;

:: cover-edges ( z -- )
    z level-border :> border
    border [ edge-chunk swap place-chunk ] each ;

CONSTANT: chunks-per-level 32

:: generate-level ( z -- )
    ! Generate ground chunks
    chunks-per-level [ z ground-chunk ] times
    z -8 > [ z stairwell ] when
    z cover-edges ;

:: init-world ( -- )
    [
        starting-chunk starting-loc place-chunk
        0 -8 [a,b] [ generate-level ] each
    ] with-mapgen
    ! Player in center
    starting-loc [ pc { 2 2 } spawn ] make-area ;