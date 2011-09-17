! Copyright (C) 2011 Risto Saarelma

USE: images.png

QUALIFIED: threads

USING: accessors arrays assocs calendar colors.constants combinators
dust.bitfont dust.com dust.fps dust.gamestate dust.geom dust.sdl formatting
images.loader kernel locals magog.areautil magog.com.creature magog.com.loc
magog.com.map-memory magog.com.view magog.effects magog.logic magog.rules
magog.tiledata math math.matrices math.vectors memoize namespaces sequences
splitting ;

IN: magog

MEMO: tiledata ( -- seq )
    "vocab:magog/tiles.png" load-image bitify-image ;

MEMO:: tile-bitmap ( n -- tile )
    tiledata
    32
    n 16 mod 2 *
    n 16 /i 16 *
    2
    16
    <bitmap8> ;

CONSTANT: screen-w 80
CONSTANT: screen-h 25

SYMBOL: fore-color
SYMBOL: back-color

COLOR: black back-color set
COLOR: white fore-color set

SYMBOL: +fps+

: memory-move ( vec -- )
    player >map-memory perceived-move ;

: remembered-map ( offset -- terrain )
    player >map-memory recall-map ;

: fps-tick ( -- )
    +fps+ get update-fps ;

: fps ( -- fps ) +fps+ get get-fps ;

: put-char ( loc char -- )
    [ fore-color get back-color get ] 2dip
    [ first2 ] dip sdl-put-char ;

: put-string ( loc str -- )
    dup empty?
    [ 2drop ]
    [ 2dup first put-char
      [ { 1 0 } v+ ] [ rest ] bi* put-string ] if ;

: putstr ( loc str -- )
    dup empty?
    [ 2drop ]
    [ 2dup first put-char
      [ { 1 0 } v+ ] [ rest ] bi* putstr
    ] if ;

SYMBOL: +msg-buffer+

SINGLETON: sdl-fx

M: sdl-fx fx-print
    drop +msg-buffer+ get swap append +msg-buffer+ set ;

: clear-msgs ( -- )
    f +msg-buffer+ set ;

: print-msgs ( -- )
    +msg-buffer+ get "\n" split
    [ 1 + 0 swap 2array swap putstr ] each-index ;

CONSTANT: P { { 1/4 -1/4 } { 1/2 1/2 } }

: screen-to-map ( screen-loc -- map-loc )
    P v.m ;

: draw-terrain ( site terrain loc -- )
    [ [ char>> over wallify-icon tile-bitmap ]
      [ fore>> ]
      [ back>> ] tri ] dip
    first2 draw-bitmap8 2drop ;

CONSTANT: fov-radius 12

: valid-pos? ( map-pos -- ? ) [ fixnum? ] all? ;

: clock-ms ( -- ms ) now timestamp>millis ;

: cycle-anim ( n salt -- k )
    hashcode clock-ms + 400 / >integer swap rem ;

:: view-frame ( uid -- frame )
    { { [ uid player = ] [ uid >view symbol>> ] } ! Player won't animate
      { [ uid >creature? ]
        [ uid >creature awake>> [ 3 uid cycle-anim ] [ 0 ] if
          uid >view symbol>> + ] }
      [ uid >view symbol>> ]
    } cond ;

! XXX: Bunch of fov-tied game logic here, needs to be moved elsewhere.
:: draw-map ( -- )
    player >map-memory :> map-memory
    { -32 -22 } { 63 45 } rect-iota
    [| screen-pos |
      screen-pos screen-to-map :> map-pos
      map-pos valid-pos?
      [ map-pos map-memory fov>> at :> site
        site [ f ] [ map-pos remembered-map ] if :> mem-site
        screen-pos { 33 23 } v+ { 8 8 } v* :> draw-pos
        site
        [ site dup >terrain draw-pos draw-terrain
          site entities-at :> entities
          entities [ view-component get-facet? ] filter
          [ last :> uid
            uid view-frame tile-bitmap
            uid >view color>> COLOR: black draw-pos first2 draw-bitmap8
          ] unless-empty
        ]
        [ mem-site
          [ dup >terrain fog-of-war-terrain draw-pos draw-terrain ] when*
        ]
        if
      ] when
    ] each
    ;

:: draw-screen ( -- )
    +fps+ get [ 0.05 <fps> +fps+ set ] unless
    ! Reset fps timestamp so that the time spent waiting for keypresses won't
    ! get factored in.
    +fps+ get fps-reset
    COLOR: black sdl-clear
    draw-map
    COLOR: white fore-color set COLOR: black back-color set
    { 0 0 }
    player current-hp "body" player skill?
    "HP: %3d/%3d  " sprintf put-string

    player >site zone-title :> title
    ! TODO: Nice, non-hacky way to do right-justified text
    80 1 - title length - 0 2array title put-string

    { 66 23 } "Move: u i o" put-string
    { 66 24 } "keys  j k l" put-string
    print-msgs
    sdl-flip
    fps-tick ;

CONSTANT: north { -1 -1 }
CONSTANT: northeast { 0 -1 }
CONSTANT: southeast { 1 0 }
CONSTANT: south { 1 1 }
CONSTANT: southwest { 0 1 }
CONSTANT: northwest { -1 0 }

: pseudo-east ( -- dir )
    player >map-memory perceived-loc>> first2 + odd? northeast southeast ? ;

: pseudo-west ( -- dir )
    player >map-memory perceived-loc>> first2 + odd? northwest southwest ? ;

: cmd-move ( dir -- ? ) player swap attempt-move memory-move t ;

: process-input ( key -- running? )
    {
        { SDLK-ESCAPE [ f ] }

        { CHAR: u [ northwest cmd-move ] }
        { SDLK-KP7 [ northwest cmd-move ] }

        { CHAR: i [ north cmd-move ] }
        { SDLK-KP8 [ north cmd-move ] }
        { SDLK-UP [ north cmd-move ] }

        { CHAR: o [ northeast cmd-move ] }
        { SDLK-KP9 [ northeast cmd-move ] }

        { CHAR: j [ southwest cmd-move ] }
        { SDLK-KP1 [ southwest cmd-move ] }

        { CHAR: k [ south cmd-move ] }
        { SDLK-KP2 [ south cmd-move ] }
        { SDLK-DOWN [ south cmd-move ] }

        { CHAR: l [ southeast cmd-move ] }
        { SDLK-KP3 [ southeast cmd-move ] }

        { SDLK-LEFT [ pseudo-west cmd-move ] }
        { SDLK-KP4 [ pseudo-west cmd-move ] }

        { SDLK-RIGHT [ pseudo-east cmd-move ] }
        { SDLK-KP6 [ pseudo-east cmd-move ] }
        [ drop t ]
    } case ;

: cycle ( -- ? )
    wait-focus
    get-key
    [ clear-msgs process-input
      fov-radius player do-fov
      run-heartbeats ]
      [ 50 milliseconds threads:sleep t ] if*
    draw-screen ;

: game-loop ( -- )
    init-magog
    fov-radius player do-fov
    draw-screen
    [ cycle sdl-quit-received? not and ] [ ] while ;

SYMBOL: +fullscreen+

: main ( -- )
    +fullscreen+ get 640 400 "Magog" [
        "vocab:magog/font.png" load-bitfont
        sdl-fx register-fx-receiver
        enable-key-repeat
        start-world game-loop stop-world
        \ tiledata reset-memoized
        \ tile-bitmap reset-memoized
    ] with-sdl ;

: spawn-magog ( -- thread ) [ main ] "magog" threads:spawn ;

: fullscreen-magog ( -- ) [ +fullscreen+ on main ] with-scope ;

: scoped-main ( -- ) [ main ] with-scope ;

MAIN: scoped-main
