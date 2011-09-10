! Copyright (C) 2011 Risto Saarelma

USING: accessors arrays assocs dust.relation.slot-index kernel locals
sequences ;

IN: dust.relation

! XXX: Uses f as a tuple slot value to denote a query wildcard. This will make
! with tuples that either actually want to have meaningful f values in their
! slots or that are type-specified to not accept them in all slots
! problematic.

TUPLE: relation rows indexes key/f ;

<PRIVATE
: wildcard-slots ( row kept-slot-indexes -- query )
    [ drop f ]
    [ swap
      [ pick index [ drop f ] unless ] map-index
      nip ] if-empty ;

: add-to-rows ( value row relation -- ) rows>> set-at ;

: remove-from-rows ( row relation -- ) rows>> delete-at ;

: add-to-indexes ( value row relation -- )
    indexes>> [ 2over rot index-add ] each 2drop ;

: remove-from-indexes ( row relation -- )
    indexes>> [ over swap index-remove ] each drop ;

: index-rows ( query/row relation -- rows/f )
    indexes>> [ dupd index-matches? ] find nip
    [ index-query keys ] [ drop f ] if* ;

: index-values ( query/row relation -- rows/f )
    indexes>> [ dupd index-matches? ] find nip
    [ index-query values ] [ drop f ] if* ;
PRIVATE>

:: <relation> ( index-slots key/f -- relation )
    relation new
    H{ } clone >>rows
    index-slots [ <slot-index> ] map >>indexes
    key/f >>key/f ;

<PRIVATE
: matches? ( row query-row -- ? )
    t [ [ = ] [ nip not ] 2bi or and ] 2reduce ;

: (select-rows) ( query/row relation -- rows )
    rows>> [ drop over matches? ] assoc-filter nip keys ;

: (select-values) ( query/row relation -- values )
    rows>> [ drop over matches? ] assoc-filter nip values ;

: row>value ( row relation -- value )
    rows>> at ;
PRIVATE>

: select-rows ( query/row relation -- rows )
    over [
        2dup index-rows
        [ 2nip ] [ (select-rows) ] if*
    ] [ 2drop f ] if ;

: select-values ( query/row relation -- values )
    over [
        2dup index-values
        [ 2nip ] [ (select-values) ] if*
    ] [ 2drop f ] if ;

: delete-rows ( query/row relation -- )
    [ select-rows ] keep swap
    [ [ over remove-from-indexes ]
      [ over remove-from-rows ] bi ] each drop ;

: select-row ( query/row relation -- row/f )
    ! TODO: Optimize to stop at first matching tuple.
    select-rows [ f ] [ first ] if-empty ;

: select-value ( query/row relation -- row/f )
    ! TODO: Optimize to stop at first matching tuple.
    select-values [ f ] [ first ] if-empty ;

<PRIVATE
: key-query ( array relation -- query/f )
    key/f>> [ 1array wildcard-slots ] [ drop f ] if* ;

: delete-matching-key ( row relation -- )
    2dup key-query
    swap delete-rows drop ;
PRIVATE>

! The tuples used here should be immutable.
: insert-row ( value row relation -- )
    [ delete-matching-key ] 2keep
    [ add-to-indexes ]
    [ add-to-rows ] 3bi ;

! Basically the same as tuple>array, but that's not enabled by default since
! it requires reflection support and game deploys might not want to have that
! enabled and bloating things. up.
GENERIC: obj>row ( obj -- seq )

: select-tuples ( query/tuple relation -- tuples )
    [ obj>row ] dip select-values ;

: delete-tuples ( query/tuple relation -- )
    [ obj>row ] dip delete-rows ;

: select-tuple ( query/tuple relation -- tuple/f )
    [ obj>row ] dip select-value ;

: insert-tuple ( tuple relation -- )
    [ dup obj>row ] dip insert-row ;