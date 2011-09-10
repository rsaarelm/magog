! Copyright (C) 2011 Risto Saarelma

USING: accessors assocs dust.com kernel locals math.vectors sequences
magog.com.creature magog.com.loc magog.fov magog.logic magog.rules ;

IN: magog.com.map-memory

TUPLE: map-memory-component < facet-component ;

: <map-memory-component> ( -- map-memory-component )
    map-memory-component new-facet-component ;

TUPLE: map-memory
    fov
    memory
    perceived-loc ;

: <map-memory> ( -- map-memory )
    H{ } clone H{ } clone { 0 0 } map-memory boa ;

M: map-memory add-facet
    map-memory-component add-to-facet-component ;

: >map-memory ( uid -- map-memory ) map-memory-component get-facet ;

: perceived-move ( vec map-memory -- )
    swap [ v+ ] curry change-perceived-loc drop ;

:: recall-map ( offset map-memory -- site/f )
    map-memory perceived-loc>> offset v+ map-memory memory>> at ;

:: see-map ( site offset map-memory -- )
    site map-memory perceived-loc>> offset v+ map-memory memory>> set-at ;
