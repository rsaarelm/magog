! Copyright (C) 2011 Risto Saarelma

USING: accessors colors.constants combinators dust.com dust.gamestate kernel
magog.com.creature magog.com.map-memory magog.com.view ;

IN: magog.gen-world.spawn

! Player starting position.
! Has fixed uid, can only be used once per world.
: pc ( -- uid )
    "pc" dup dup register-uid {
        [ "player" 48 COLOR: LightBlue <view> add-facet ]
        [ <creature> H{
            { "body" 10 }
            { "attack" 7 }
          } clone >>skills
          add-facet ]
        [ <map-memory> add-facet ]
    } cleave ;

: dreg ( -- uid )
    new-uid dup {
        [ "a dreg" 51 COLOR: tan4 <view> add-facet ]
        [ <creature> H{
            { "body" 2 }
            { "attack" 2 }
          } clone >>skills
          add-facet ]
    } cleave ;

: thrall ( -- uid )
    new-uid dup {
        [ "a thrall" 54 COLOR: cyan4 <view> add-facet ]
        [ <creature> H{
            { "body" 6 }
            { "attack" 5 }
          } clone >>skills
          add-facet ]
    } cleave ;
