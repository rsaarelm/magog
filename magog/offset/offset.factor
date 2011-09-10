! Copyright (C) 2011 Risto Saarelma

USING: assocs dust.hex kernel math.vectors memoize namespaces sequences
magog.com.loc ;

IN: magog.offset

: atomic-offset ( vec site -- site' )
    translate-site dup get-portal
    [ portal-translate ] when* ;

<PRIVATE

SYMBOL: +center-site+
SYMBOL: +site-cache+

MEMO: hexline-previous ( vec -- previous-vec difference )
    dup { 0 0 } =
    [ f swap ]
    [ dup dup hex-line last v- swap dupd swap v- ] if ;

: cache ( vec site -- site ) [ swap +site-cache+ get set-at ] keep ;

: cached ( vec -- site/f )
    dup { 0 0 } = [ drop +center-site+ get ]
    [ +site-cache+ get at ] if ;

: start-offset ( center-site -- )
    +center-site+ set
    H{ } clone +site-cache+ set ;

PRIVATE>

! Called from within with-offset-center, cache using the specific site.
: offset ( vec -- site )
    dup cached [ nip ]
    [ dup hex-line +center-site+ get [ swap atomic-offset ] reduce
      cache ] if* ;

: with-center ( center-site quot -- )
    [ swap start-offset call ] with-scope ; inline

: offset-site ( vec site -- site' )
    swap [ offset ] curry with-center ;

: neighbor-sites ( site -- seq ) hex-dirs [ over atomic-offset ] map nip ;