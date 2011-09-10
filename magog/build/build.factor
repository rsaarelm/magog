! Copyright (C) 2011 Risto Saarelma

USING: io.pathnames memory vocabs.parser ;

IN: magog.build

: save-magog-image ( -- )
    "magog" use-vocab
    "magog.image" resource-path save-image ;

MAIN: save-magog-image