! Copyright (C) 2011 Risto Saarelma

USING: arrays assocs circular combinators dust.geom fry kernel literals locals
math math.constants math.functions math.matrices math.order math.vectors
sequences ;

IN: dust.hex

! Some of the code uses two coordinate systems, hex coordinates and cartesian
! coordinates. Hex coordinates are a nice way to operate in hex space, since
! they point to two hex edges from the center, but the coordinate axes don't
! form a right angle, so trying to operate with angles in hex space is going to
! get messed. Cartesian coordinates are the regular plane coordinates.
!
! Cartesian coordinates also use the analytic geometry convention of having an
! up-poiting y-axis while hex coordinates use the computer graphics convention
! of having a down-pointing y-axis.
!
!          A
!          |   cartesian-y axis
!         .'.
!       0' | '1
!     .'   |   '.
!    :     |     :   hex, cart { 1 0 }
!    5     o ----2---- + -->
!    :    |      :  hex-x axis and
!  -- '. |     .'   cartesian-x axis,
!       4.   .3     (they are the same)
!      |  '.'
!     | hex-y axis
!    + hex { 0 1 } ( cartesian { -1/2 -sqrt(3)/2 } )

! The four wall types in the simple fake-isometric wall graphics scheme
CONSTANT: cross-block-wall 0
CONSTANT: x-axis-wall 1
CONSTANT: y-axis-wall 2
CONSTANT: axis-diagonal-wall 3

CONSTANT: hex-dirs { { -1 -1 } { 0 -1 } { 1 0 } { 1 1 } { 0 1 } { -1 0 } }

<PRIVATE

! Bit positions of the directions.
CONSTANT: n  0
CONSTANT: ne 1
CONSTANT: se 2
CONSTANT: s  3
CONSTANT: sw 4
CONSTANT: nw 5

: power-of-2 ( n -- 2^n ) 1 swap shift ;

: anybit= ( a b -- ? ) [ 0 = ] bi@ = ;

! Pattern-assoc maps bit positions to 0 for bit having to be off and 1 for bit
! having to be on. Bits not listed in the assoc are ignored.
: check-mask ( mask pattern-assoc -- ? )
    swap '[ swap power-of-2 _ bitand anybit= ] assoc-all? ;

: top-or-bottom-corner? ( neighbor-wall-mask -- ? )
    [ { { $ nw 1 } { $ ne 1 } { $ n 0 } } check-mask ]
    [ { { $ sw 1 } { $ se 1 } { $ s 0 } } check-mask ] bi or ;

: x-axis-wall? ( neighbor-wall-mask -- ? )
    [ { { $ se 1 } { $ nw 1 } { $ ne 0 } } check-mask ]
    [ { { $ se 1 } { $ nw 1 } { $ sw 0 } } check-mask ] bi or ;

: y-axis-wall? ( neighbor-wall-mask -- ? )
    [ { { $ ne 1 } { $ sw 1 } { $ se 0 } } check-mask ]
    [ { { $ ne 1 } { $ sw 1 } { $ nw 0 } } check-mask ] bi or ;

: axis-diagonal-wall? ( neighbor-wall-mask -- ? )
    [ { { $ n 1 } { $ s 1 } { $ sw 0 } { $ nw 0 } } check-mask ]
    [ { { $ n 1 } { $ s 1 } { $ se 0 } { $ ne 0 } } check-mask ] bi or ;


PRIVATE>

: neighbors ( pos -- seq ) hex-dirs swap '[ _ v+ ] map ;

!  0  .     8  .     16  .     24  .     32  .     40  .     48  .     56  .
!   .   .    .   .     .   .     .   .     #   .     #   .     #   .     #   .
!     *        *         *         *         *         *         *         *
!   .   .    .   .     #   .     #   .     .   .     .   .     #   .     #   .
!     .        #         .         #         .         #         .         #
!
!  1  #     9  #     17  #     25  #     33  #     41  #     49  #     57  #
!   .   .    .   .     .   .     .   .     #   .     #   .     #   .     #   .
!     *        *         *         *         *         *         *         *
!   .   .    .   .     #   .     #   .     .   .     .   .     #   .     #   .
!     .        #         .         #         .         #         .         #
!
!  2  .    10  .     18  .     26  .     34  .     42  .     50  .     58  .
!   .   #    .   #     .   #     .   #     #   #     #   #     #   #     #   #
!     *        *         *         *         *         *         *         *
!   .   .    .   .     #   .     #   .     .   .     .   .     #   .     #   .
!     .        #         .         #         .         #         .         #
!
!  3  #    11  #     19  #     27  #     35  #     43  #     51  #     59  #
!   .   #    .   #     .   #     .   #     #   #     #   #     #   #     #   #
!     *        *         *         *         *         *         *         *
!   .   .    .   .     #   .     #   .     .   .     .   .     #   .     #   .
!     .        #         .         #         .         #         .         #
!
!  4  .    12  .     20  .     28  .     36  .     44  .     52  .     60  .
!   .   .    .   .     .   .     .   .     #   .     #   .     #   .     #   .
!     *        *         *         *         *         *         *         *
!   .   #    .   #     #   #     #   #     .   #     .   #     #   #     #   #
!     .        #         .         #         .         #         .         #
!
!  5  #    13  #     21  #     29  #     37  #     45  #     53  #     61  #
!   .   .    .   .     .   .     .   .     #   .     #   .     #   .     #   .
!     *        *         *         *         *         *         *         *
!   .   #    .   #     #   #     #   #     .   #     .   #     #   #     #   #
!     .        #         .         #         .         #         .         #
!
!  6  .    14  .     22  .     30  .     38  .     46  .     54  .     62  .
!   .   #    .   #     .   #     .   #     #   #     #   #     #   #     #   #
!     *        *         *         *         *         *         *         *
!   .   #    .   #     #   #     #   #     .   #     .   #     #   #     #   #
!     .        #         .         #         .         #         .         #
!
!  7  #    15  #     23  #     31  #     39  #     47  #     55  #     63  #
!   .   #    .   #     .   #     .   #     #   #     #   #     #   #     #   #
!     *        *         *         *         *         *         *         *
!   .   #    .   #     #   #     #   #     .   #     .   #     #   #     #   #
!     .        #         .         #         .         #         .         #
!
! All the wall mask values and the corresponding wall patterns

:: hex-wall-type ( neighbor-wall-mask -- wall-type )
    cross-block-wall x-axis-wall y-axis-wall axis-diagonal-wall :> ( o x y z )
    neighbor-wall-mask {
        { 2 [ y ] }
        { 3 [ y ] }
        { 4 [ x ] }
        { 8 [ z ] }
        { 9 [ z ] }
        { 10 [ y ] }
        { 11 [ z ] }
        { 12 [ x ] }
        { 13 [ z ] }
        { 15 [ z ] }
        { 16 [ y ] }
        { 18 [ y ] }
        { 19 [ y ] }
        { 22 [ y ] }
        { 24 [ y ] }
        { 25 [ z ] }
        { 26 [ y ] }
        { 30 [ y ] }
        { 31 [ y ] }
        { 32 [ x ] }
        { 33 [ x ] }
        { 36 [ x ] }
        { 37 [ x ] }
        { 38 [ x ] }
        { 39 [ x ] }
        { 40 [ x ] }
        { 44 [ x ] }
        { 47 [ x ] }
        { 50 [ y ] }
        { 51 [ y ] }
        { 52 [ x ] }
        { 57 [ z ] }
        { 59 [ y ] }
        { 60 [ x ] }
        { 61 [ x ] }
        [ drop o ]
    } case ;
!    {
!        { [ dup top-or-bottom-corner? ] [ cross-block-wall ] }
!        { [ dup x-axis-wall? ] [ x-axis-wall ] }
!        { [ dup y-axis-wall? ] [ y-axis-wall ] }
!        { [ dup axis-diagonal-wall? ] [ axis-diagonal-wall ] }
!        [ cross-block-wall ]
!    } cond nip ;

: dir6>vec ( dir -- hex-vec ) hex-dirs nth ;

: 7-cells ( pos -- seq ) dup neighbors swap prefix ;

: hex-dist ( pos1 pos2 -- dist )
    v- [ vabs first2 ] [ [ sgn ] map first2 = ] bi
    [ max ] [ + ] if ;

CONSTANT: hex-projection
{ { 1   0               }
  { -1/2 $[ 3 sqrt -2 / ] } }

: hex>cartesian ( hex-vec -- cartesian-vec )
    hex-projection v.m ;

: hex-angle ( hex-vec -- radian )
    hex>cartesian vec2-angle ;

: vec>hex-dir ( hex-vec -- hex-dir )
    hex-angle 2/6 pi * / 5/2 - neg >integer 6 rem hex-dirs nth ;

: angle>hex-sector ( radian -- hex-sector )
    3 * pi / 5/2 swap - 6 rem ;

: exit-sector ( hex-vec -- hex-sector )
    hex-angle angle>hex-sector ;

: sector-normal ( hex-sector -- cartesian-vec )
    { { -1/2 $[ 3 sqrt 2 / ] }
      { 1/2 $[ 3 sqrt 2 / ] }
      { 1 0 }
      { 1/2 $[ 3 sqrt -2 / ] }
      { -1/2 $[ 3 sqrt -2 / ] }
      { -1 0 } } nth ;

: hexline-scale ( vec -- vec' ) 1/2 3 sqrt 2 / 2array v/ ;

: rot-to-sector ( cartesian-vec sector -- cartesian-vec' )
    sector-normal vec2-angle neg rotate-vec ;

:: hex-line ( hex-vec -- step-seq )
    hex-vec exit-sector >fixnum :> sector
    hex-vec hex>cartesian sector rot-to-sector
    hexline-scale first2 :> ( dx dy )
    dx dy abs 1 + / :> y-jump
    sector dir6>vec :> ahead-step
    sector dy sgn - 6 rem dir6>vec :> side-step
    y-jump 1 - { 0 0 }
    ! if the algorithm is broken and we miss hex-vec, hello infinite loop.
    [ dup hex-vec = not ]
    [ over 0 <=
      [ [ y-jump + 1 - ] [ side-step v+ ] bi* side-step ]
      [ [ 2 - ] [ ahead-step v+ ] bi* ahead-step ] if ] produce 2nip ;

: test-hex-line ( hex-vec -- )
    dup hex-line { 0 0 } [ v+ ] reduce swap assert= ;

<PRIVATE
: sector-edge ( n -- vec )
    { { -1 -1 } { 0 -1 } { 1 0 } { 1 1 } { 0 1 } { -1 0 } } nth ;

: sector-scan ( n -- vec )
    { { 1 0 } { 1 1 } { 0 1 } { -1 0 } { -1 -1 } { 0 -1 } } nth ;
PRIVATE>

MEMO:: hex-circle ( r -- seq )
    r 0 = [ { { 0 0 } } ]
    [ 6 iota [| sector |
        sector sector-edge r v*n
        r iota [ sector sector-scan n*v over v+ ] map nip ]
    map concat ] if <circular> ;