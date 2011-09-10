! Copyright (C) 2011 Risto Saarelma

USING: make namespaces ;

IN: magog.effects

SYMBOL: +fx-receiver+

: register-fx-receiver ( receiver -- ) +fx-receiver+ set ;

GENERIC: fx-print ( txt receiver -- )

: msg-nl ( -- ) "\n" +fx-receiver+ get fx-print ;

: msg ( txt -- )
    +fx-receiver+ get fx-print msg-nl ;

: make-msg ( quot -- ) "" make msg ; inline
