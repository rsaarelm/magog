! Copyright (C) 2011 Risto Saarelma

USING: combinators kernel locals magog.com.view magog.rules make math
sequences unicode.case ;

IN: magog.text

: article ( str -- article? )
    { { [ dup "a " head? ] [ "a" ] }
      { [ dup "an " head? ] [ "an" ] }
      [ f ]
    } cond nip ;

: without-article ( str -- str' )
    dup article [ length 1 + tail ] when* ;

: the-name ( str -- the-str )
    dup article [ [ "the " % without-article % ] "" make ] when ;

: The-name ( str -- the-str ) the-name capitalize ;

: es ( verb -- verbs ) dup "s" tail? "es" "s" ? append ;

:: verbs-msg ( subject-uid verb object-uid -- txt )
    subject-uid name object-uid name :> ( sub obj )
    subject-uid player = :> player?
    [ sub The-name % " " %
      verb player? [ es ] unless %
      " " % obj the-name % "." % ]
    "" make ;