! Copyright (C) 2011 Risto Saarelma

USING: accessors arrays combinators combinators.short-circuit dust.com
dust.gamestate dust.hex formatting kernel locals magog.com.creature
magog.com.loc magog.com.view magog.offset math math.functions math.order
math.parser random sequences splitting ;

IN: magog.rules

: player ( -- player-uid ) "pc" ;

: name ( uid -- txt )
    dup player = [ drop "you" ]
    [ dup view-component get-facet? [ name>> nip ] when* ] if ;

: area-drawables ( -- entities ) entities
    [ [ view-component get-facet? ]
      [ loc-component get-facet? ] bi and ] filter ;

: can-walk-terrain? ( terrain -- ? )
    dup [ walk?>> ] [ drop f ] if ;

: can-fly-terrain? ( terrain -- ? )
    dup [ fly?>> ] [ drop f ] if ;

: blocks-move? ( uid -- ? ) >creature? ;

: enemy-of? ( u1 u2 -- ? )
    { [ = not ] ! They must be different things,
      [ [ >creature? ] bi@ and ] ! that are both creatures,
      [ 2array player swap index ] ! and (for now) at least one must be player.
    } 2&& ;

: find-enemy ( uid site -- uid/f )
    entities-at [ over swap enemy-of? ] find 2nip ;

CONSTANT: skill-exponent-factor 5

: skill>power ( skill -- power ) 2 swap skill-exponent-factor / ^ ;

: power>skill ( power -- skill ) log 2 1 skill-exponent-factor / ^ log / ;

: roll ( skill -- power ) [ 0 ] dip skill>power uniform-random-float ;

: (contest) ( winning-result losing-result -- outcome )
    [ 1 max ] [ 0 max ] bi* - ;

: contest ( attack-skill defense-skill -- outcome-skill )
    [ roll power>skill ] bi@
    2dup >= [ (contest) ceiling ] [ swap (contest) neg floor ] if >integer ;

: adjacent-enemy-dir? ( uid -- vec/f )
    dup >site neighbor-sites [ over swap find-enemy ] find drop nip
    [ hex-dirs nth ] [ f ] if* ;

:: current-hp ( uid -- hp )
    uid >creature? [ body>> "body" uid skill? [ * ] when* >integer ]
    [ 1 ] if* ;

: zone-title ( site -- name )
    area-uid>> "," split third string>number
    { { [ dup 0 = ] [ drop "overworld" ] }
      [ neg "underground %d" sprintf ] } cond ;