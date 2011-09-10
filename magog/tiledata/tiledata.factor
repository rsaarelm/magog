! Copyright (C) 2011 Risto Saarelma

USING: accessors colors colors.constants combinators combinators.short-circuit
dust.data dust.img kernel lexer math memoize parser strings words
words.constant ;

IN: magog.tiledata

TUPLE: tiledata
    { name string read-only }
    { char fixnum read-only }
    { fore color read-only }
    { back color read-only }
    { walk? boolean read-only }
    { fly? boolean read-only }
    { see? boolean read-only }
    { spread? boolean read-only }
    ;

SYNTAX: TILE:
    scan dup  ! name, dub for tile constant
    scan parse-char ! char
    scan parse-color ! fore-color
    scan parse-color ! back-color
    scan-object ! tile flags
    scan-object
    scan-object
    scan-object
    tiledata boa
    [ create-in dup reset-generic ] dip
    define-constant ;

: tiles-merge? ( t1 t2 -- char/f )
    [ drop char>> ] 2keep
    { [ [ char>> ] bi@ = ] [ [ spread?>> ] bi@ and ] } 2&& swap and ;

: between-background ( t1 t2 -- col )
    [ back>> ] bi@ 1/2 clerp ;

: between-foreground ( t1 t2 -- col )
    [ fore>> ] bi@ 1/2 clerp ;

! XXX: Really ugly way to make a slightly modified clone of an immutable
! tuple...
MEMO: fog-of-war-terrain ( terrain -- terrain' )
    [ { [ name>> ]
        [ char>> ]
        [ fore>> COLOR: black 1/2 clerp ]
        [ back>> COLOR: black 1/2 clerp ]
        [ walk?>> ]
        [ fly?>> ]
        [ see?>> ]
        [ spread?>> ] } cleave tiledata boa
    ] [ f ] if* ;
