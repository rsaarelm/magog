! Copyright (C) 2011 Risto Saarelma

USING: accessors assocs dust.gamestate fry kernel make ;

IN: dust.com

GENERIC: add-facet ( uid facet -- )

TUPLE: facet-component facets ;

: new-facet-component ( class -- facet-component ) new H{ } clone >>facets ;

: add-to-facet-component ( uid facet component-key -- )
    [ swap ] dip get-component facets>> set-at ;

: (get-facet?) ( uid facet-component -- facet? ) facets>> at ; inline

: get-facet? ( uid key -- facet? ) get-component (get-facet?) ; inline

: (get-facet) ( uid facet-component -- facet ) 2dup get-facet? dup
    [ 2nip ]
    [ drop '[ "Uid '" % _ % "' has no facet " % _ % "." ] "" make throw ] if ;

: get-facet ( uid key -- facet ) (get-facet) ;

M: facet-component remove-uid
    facets>> delete-at ;