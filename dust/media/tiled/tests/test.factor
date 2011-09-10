! Copyright (C) 2010 Risto Saarelma
! See http://factorcode.org/license.txt for BSD license.
USING: combinators kernel dust.media.tiled sequences tools.test ;
IN: dust.media.tiled.tests

: map-ok? ( mapdata -- ? )
    {
        [ length 3 = ]
        [ [ first ] map { "Background" "Terrain" "Objects" } = and ]
        [ first second first second 97 = and ]
        [ second second first 5 swap nth 119 = and ]
    } cleave ;

[ t ] [ "vocab:dust/media/tiled/tests/map.tmx" file>tiled map-ok? ] unit-test
