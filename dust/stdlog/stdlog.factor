! Copyright (C) 2011 Risto Saarelma

USING: io io.encodings.utf8 io.files kernel namespaces prettyprint ;

IN: dust.stdlog

<PRIVATE
CONSTANT: log-path "/tmp/stdout.txt"

SYMBOL: +inited+

: init ( -- )
    +inited+ get
    [ "" log-path utf8 set-file-contents
      +inited+ on ] unless ;
PRIVATE>

: logmsg ( seq -- ) init log-path utf8 [ write ] with-file-appender ;

: p-logmsg ( obj -- ) unparse-short logmsg ;
