! Copyright (C) 2011 Risto Saarelma

USING: accessors calendar kernel literals math math.functions ;

IN: dust.fps

TUPLE: fps dampening timestamp interval-seconds ;

<PRIVATE

CONSTANT: initial-interval $[ 1 seconds ]

: timestamp-and-get-elapsed ( fps -- seconds ) now
    [ swap timestamp>> time- duration>seconds ]
    [ >>timestamp drop ] 2bi ;

! : new

PRIVATE>

: <fps> ( dampening -- fps )
    initial-interval [ ago ] [ duration>seconds ] bi fps boa ;

: update-fps ( fps -- ) dup
    [ timestamp-and-get-elapsed ]
    [ interval-seconds>> ]
    [ dampening>> ] tri lerp >>interval-seconds drop ;

: get-fps ( fps -- 1/s )
    interval-seconds>> 1 swap / >integer ;

: fps-reset ( fps -- )
    now >>timestamp drop ;