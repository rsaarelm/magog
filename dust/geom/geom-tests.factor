! Copyright (C) 2010 Risto Saarelma
! See http://factorcode.org/license.txt for BSD license.
USING: literals math.rectangles dust.geom tools.test ;
IN: dust.geom.tests

[ RECT: { 2 3 } { 4 4 } ]
[
    RECT: { 2 3 } { 4 4 }
    RECT: { 0 0 } { 10 10 }
    overlap-rect
] unit-test

[ RECT: { 2 3 } { 4 4 } ]
[
    RECT: { 2 3 } { 4 4 }
    RECT: { 3 3 } { 1 1 }
    overlap-rect
] unit-test

[ RECT: { 2 3 } { 4 4 } ]
[
    RECT: { 2 3 } { 4 4 }
    RECT: { 2 3 } { 4 4 }
    overlap-rect
] unit-test

[ RECT: { 10 10 } { 3 2 } ]
[
    RECT: { 2 3 } { 3 2 }
    RECT: { 10 10 } { 4 4 }
    overlap-rect
] unit-test

[ RECT: { -9 -8 } { 3 2 } ]
[
    RECT: { 2 3 } { 3 2 }
    RECT: { -10 -10 } { 4 4 }
    overlap-rect
] unit-test

[ RECT: { 2 -8 } { 3 2 } ]
[
    RECT: { 2 3 } { 3 2 }
    RECT: { 1 -10 } { 4 4 }
    overlap-rect
] unit-test

[ RECT: { -9 3 } { 3 2 } ]
[
    RECT: { 2 3 } { 3 2 }
    RECT: { -10 2 } { 4 4 }
    overlap-rect
] unit-test

[ RECT: { 0 0 } { 320 240 } ]
[
    RECT: { 32 24 } { 288 216 } RECT: { -32 -24 } { 320 240 } resize-rect
] unit-test

[ RECT: { 32 24 } { 288 216 } ]
[
    RECT: { 0 0 } { 320 240 } RECT: { 32 24 } { 288 216 } resize-rect
] unit-test