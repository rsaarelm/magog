! Copyright (C) 2011 Risto Saarelma

USING: accessors arrays assocs dust.com dust.gamestate dust.relation kernel ;

IN: magog.com.creature

TUPLE: creature-component < facet-component
    awake-index ;

: <creature-component> ( -- creature-component )
    creature-component new-facet-component
    { 0 } 0 <relation> >>awake-index ;

TUPLE: creature
    skills
    body
    awake
    threat
    target-vec
    ;

<PRIVATE

: awake-index ( -- awake-index )
    creature-component get-component awake-index>> ;

: awake-row ( uid -- awake-row ) 1array ;

: awake-query ( -- awake-query ) f 1array ;

: index-awake ( uid -- )
    dup awake-row awake-index insert-row ;

: index-asleep ( uid -- )
    awake-row awake-index delete-rows ;

PRIVATE>

M: creature-component remove-uid
    [ facets>> delete-at ]
    [ drop index-asleep ] 2bi ;

: awake-creatures ( -- seq )
    awake-query awake-index select-values ;

: <creature> ( -- creature )
    creature new
    H{ } clone >>skills
    1.0 >>body
    1 >>threat
    f >>awake ;

M: creature add-facet creature-component add-to-facet-component ;

: >creature ( uid -- creature ) creature-component get-facet ;

: >creature? ( uid -- creature/f ) creature-component get-facet? ;

: when-creature ( uid quot -- ) [ >creature? ] dip when* ; inline

: if-creature ( uid true false -- ) [ >creature? ] 2dip if* ; inline

: wake-creature ( uid -- )
    dup >creature? [ [ t >>awake drop ] [ over index-awake drop ] bi ] when* drop ;

: sleep-creature ( uid -- )
    dup >creature? [ [ f >>awake drop ] [ over index-asleep drop ] bi ] when* drop ;

: skill? ( skill uid -- skill/0 )
    [ skills>> at ] [ drop f ] if-creature
    [ 0 ] unless* ;
