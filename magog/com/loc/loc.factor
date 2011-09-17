! Copyright (C) 2011 Risto Saarelma

USING: accessors arrays assocs combinators dust.com dust.gamestate
dust.relation kernel magog.com.area math.vectors sequences strings ;

IN: magog.com.loc

TUPLE: site { loc array read-only } { area-uid string read-only } ;

: <site> ( loc area-uid -- site ) site boa ;

TUPLE: loc-component < facet-component site-index ;

<PRIVATE
TUPLE: entity-site site uid ;

: <site-index> ( -- relation ) { 0 } 1 <relation> ;

: register-site ( uid site -- )
    swap entity-site boa
    loc-component get-component site-index>>
    insert-tuple ;

M: entity-site obj>row
    { [ site>> ] [ uid>> ] } cleave 2array ;

PRIVATE>

M: loc-component remove-uid
    [ facets>> delete-at ]
    [ [ f swap entity-site boa ] dip site-index>> delete-tuples ] 2bi ;

: <loc-component> ( -- loc-component )
    loc-component new-facet-component <site-index> >>site-index ;

M: site add-facet
    [ loc-component add-to-facet-component ]
    [ register-site ] 2bi ;

: >site ( uid -- site ) loc-component get-facet ;

: site>loc ( site -- loc ) loc>> ;

: site>area ( site -- area ) area-uid>> area-component get-facet? ;

: decompose-site ( site -- loc area-facet ) [ site>loc ] [ site>area ] bi ;

: get-area ( uid -- area-uid ) >site site>area ;

: get-loc ( uid -- loc ) >site loc>> ;

: >terrain ( site -- terrain )
    [ site>loc ] [ site>area ] bi get-terrain ;

: translate-site ( vec site -- site' )
    [ loc>> v+ ]
    [ nip area-uid>> ] 2bi <site> ;

: set-site ( uid site -- ) add-facet ;

: portal-translate ( site portal-site -- site' )
    [ [ [ loc>> ] bi@ v+ ]
      [ area-uid>> nip ] 2bi <site> ] when* ;

: get-portal ( site -- portal? ) decompose-site
    [ area-get-portal ] [ drop f ] if* ;

: entities-at ( site -- entities )
    f entity-site boa loc-component get-component site-index>>
    select-tuples [ uid>> ] map ;

: blocks-sight? ( site -- ? ) >terrain see?>> not ;