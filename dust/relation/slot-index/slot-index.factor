! Copyright (C) 2011 Risto Saarelma

USING: accessors arrays assocs hashtables kernel locals sequences ;

IN: dust.relation.slot-index

TUPLE: slot-index slot-n index ;

: <slot-index> ( slot-n -- slot-index )
    H{ } clone slot-index boa ;

<PRIVATE
: index-key ( row slot-index -- key ) slot-n>> swap nth ;

: key-and-index ( row slot-index -- key index )
    [ index-key ] [ nip index>> ] 2bi ;

: get-bucket ( row slot-index -- bucket )
     key-and-index at ;

: set-bucket ( bucket row slot-index -- )
    key-and-index
    pick [ set-at ] [ delete-at drop ] if ;

: bucket-add ( value row old-bucket -- new-bucket )
    [ 2dup key? [ 2nip ] [ [ set-at ] keep ] if ]
    [ swap 2array 1array >hashtable ] if* ;

: bucket-remove ( tuple old-bucket -- new-bucket )
    [ delete-at ] keep dup assoc-empty? [ drop f ] when ;
PRIVATE>

:: index-add ( value row slot-index -- )
    row slot-index get-bucket [ value row ] dip bucket-add
    row slot-index set-bucket ;

:: index-remove ( row slot-index -- )
    row slot-index get-bucket [ row ] dip bucket-remove
    row slot-index set-bucket ;

! If the query tuple has a non-f value in the indexed slot, it will be
! indexed.
: index-matches? ( query/row slot-index -- ? )
    slot-n>> swap nth ;

: index-query ( row slot-index -- rows-assoc ) key-and-index at ;