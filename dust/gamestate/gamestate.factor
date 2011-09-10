! Copyright (C) 2011 Risto Saarelma

USING: accessors assocs classes fry hash-sets hashtables kernel make math
math.parser namespaces sequences ;

QUALIFIED: sets

IN: dust.gamestate

TUPLE: gamestate
    { entities sets:set }
    { components hashtable }
    { next-uid-seed integer } ;

<PRIVATE

SYMBOL: +world+

: assert-world-exists ( world -- )
    [ "World not initialized." throw ] unless ; inline

: <gamestate> ( -- gamestate )
    gamestate new
    { } <hash-set> >>entities
    H{ } clone >>components
    0 >>next-uid-seed ;

PRIVATE>

: start-world ( -- )
    <gamestate> +world+ set ;

: stop-world ( -- )
    f +world+ set ;

: with-world ( quot -- )
    [ <gamestate> +world+ set call ] with-scope ; inline

: world ( -- gamestate )
    +world+ get dup assert-world-exists ; inline

: uid-in-use? ( uid -- ? ) world entities>> sets:in? ;

<PRIVATE

: increment-uid-seed ( -- ) world [ 1 + ] change-next-uid-seed drop ;

: gen-uid ( -- uid )
    increment-uid-seed world next-uid-seed>> '[ "entity#" % _ # ] "" make ;

: next-uid ( -- uid )
    [ gen-uid dup uid-in-use? ] [ drop ] while ;

: assert-unused-uid ( uid -- )
    dup uid-in-use?
    [ '[ "Uid '" % _ % "' already in use" ] "" make throw ]
    [ drop ] if ;

PRIVATE>

: register-uid ( uid -- )
    dup assert-unused-uid
    world entities>> sets:adjoin ;

: new-uid ( -- uid ) next-uid dup register-uid ;

: register-component ( component -- )
    dup class world components>> set-at ;

: get-component ( key -- component? )
    world components>> at* drop ;

: entities ( -- entities ) world entities>> sets:members ;

! Components need to implement this.
GENERIC: remove-uid ( uid component -- )

: delete-entity ( uid -- )
    [ world components>> [ nip over swap remove-uid ] assoc-each drop ]
    [ world entities>> sets:delete ] bi ;