! Copyright (C) 2011 Risto Saarelma

USING: accessors assocs combinators combinators.short-circuit dust.gamestate
dust.hex fry kernel locals make math math.order math.parser math.vectors
random sequences magog.com.area magog.com.creature magog.com.loc
magog.com.map-memory magog.com.view magog.effects magog.fov magog.gen-world
magog.offset magog.rules ;

IN: magog.logic

: init-components ( -- )
    <creature-component> register-component
    <area-component> register-component
    <view-component> register-component
    <map-memory-component> register-component
    <loc-component> register-component ;

: init-magog ( -- )
    init-components
    init-world ;

: full-heal ( uid -- )
    [ 1.0 >>body drop ] when-creature ;

: kill ( uid -- ) dup player =
    [ full-heal "Your body regenerates." msg ]
    [ delete-entity ] if ;

:: attack ( attacker-uid target-uid -- )
    "attack" attacker-uid skill? "body" target-uid skill? contest :> damage
    damage 0 > [
        damage "body" target-uid skill? dup 0 = [ drop 1 ] when / :> scaled
        target-uid [ body>> scaled - ] [ -1 ] if-creature :> new-body
        new-body 0 <
        [ "Kill!" msg target-uid kill ]
        [ [ "Hit for " % damage # "." % ] make-msg
          target-uid >creature? [ new-body >>body drop ] when* ] if ] when ;

:: attempt-move ( uid vec -- actually-moved-vec )
    vec uid >site offset-site :> target
    uid target find-enemy
    [| enemy-uid | uid enemy-uid attack { 0 0 } ]
    [
        { [ target >terrain can-walk-terrain? ]
          [ target entities-at [ blocks-move? ] any? not ] } 0&&
        [ uid target set-site vec ] [ { 0 0 } ] if
    ] if* ;

:: fight-adjacent? ( uid -- ? )
    uid adjacent-enemy-dir? [ uid swap attempt-move ]
    [ f ] if* ;

: give-target ( uid vec -- )
    [ >creature ] dip '[ _ >>target-vec drop ] when* ;

: forget-target ( uid -- ) >creature f >>target-vec drop ;

: adjust-target-vec ( step uid -- )
    >creature swap [ v- ] curry change-target-vec drop ;

:: approach-target? ( uid -- ? )
    uid >creature target-vec>> [| target-vec |
        uid target-vec vec>hex-dir attempt-move :> step
        step { 0 0 } = [ uid forget-target f ]
        [ step uid adjust-target-vec t ] if
    ] [ f ] if* ;

: wander ( uid -- )
    ! TODO: Try to move to open places, don't punch your buddies.
    hex-dirs random attempt-move drop ;

CONSTANT: threat-level 5

CONSTANT: regeneration-rate 1/5

: activate-threat ( uid -- )
    [ threat-level >>threat drop ] when-creature ;

: decrement-threat ( uid -- )
    [ [ 1 - 0 max ] change-threat drop ] when-creature ;

: threatened? ( uid -- ? )
    [ threat>> 0 > ] [ f ] if-creature ;

: regenerate ( uid -- )
    [ [ regeneration-rate + 1 min ] change-body drop ] when-creature ;

<PRIVATE

:: ping-entity ( uid viewer loc -- )
    ! Wake up enemies when they come into player's fov

    ! TODO: Stealth check, player can sneak up on unsuspecting enemies.
    loc hex-dist 4 <= [
        uid wake-creature
        uid loc vneg give-target ] when
    ! Re-threaten viewer whenever an awake hostile is encountered
    viewer uid enemy-of? uid [ awake>> ] [ f ] if-creature and
    [ viewer activate-threat ] when ;

PRIVATE>

:: do-fov ( radius uid -- )
    ! Threat is reactivated during fov process if enemies are found.
    uid decrement-threat

    uid >map-memory :> map-memory
    uid >site radius fov :> fov
    map-memory fov >>fov drop
    fov [ swap map-memory see-map ] assoc-each
    fov values [ entities-at ] map concat :> entities

    fov [| loc site |
        site entities-at [ uid loc ping-entity ] each
    ] assoc-each
    ;

! TODO: Less hacky way to tell we don't want to run an AI on the player mob.
:: heartbeat ( uid -- )
    { [ uid >creature? ] [ uid player = not ] } 0&&
    [
        {
            { [ uid fight-adjacent? ] [ [ uid % " attacks!" % ] make-msg ] }
            { [ uid approach-target? ] [ ] }
            [ uid wander ]
        } cond
    ] when
    uid player =
    [
        uid threatened?
        [ uid regenerate ] unless
    ] when ;

: run-heartbeats ( -- )
    awake-creatures [ heartbeat ] each ;