! Copyright (C) 2011 Risto Saarelma

USING: accessors assocs dust.com kernel ;

QUALIFIED-WITH: magog.tile tile

IN: magog.com.area

TUPLE: area-component < facet-component ;

: <area-component> ( -- area-component ) area-component new-facet-component ;

TUPLE: area-facet terrain portals ;

: <area-facet> ( -- area-facet )
    area-facet new
    ! TODO use arrays for terrain instead of hashtables for faster access
    H{ } clone >>terrain
    H{ } clone >>portals ;

: >area ( uid -- area-facet ) area-component get-facet ;

: get-terrain ( pos area-facet -- tile )
    dup [ terrain>> at [ tile:ethereal-void ] unless* ]
    [ 2drop tile:ethereal-void ] if ;

: area-get-portal ( loc area-facet -- portal? ) portals>> at ;

: set-portal ( portal loc area-facet -- )
    portals>> set-at ;

M: area-facet add-facet area-component add-to-facet-component ;
