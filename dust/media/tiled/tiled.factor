! Copyright (C) 2010 Risto Saarelma

USING: accessors arrays ascii assocs base64 combinators compression.inflate
fry grouping kernel make math math.parser pack sequences xml xml.traversal ;
IN: dust.media.tiled

<PRIVATE
: attr-error ( attr -- ) '[ "Attribute '" % % "' not found." % ] "" make throw ;

: get-attr ( xml-tag attr -- value ) swap attrs>> ?at [ attr-error f ] unless ;

: get-attr? ( xml-tag attr -- value/f ) swap attrs>> at ;

: layer-dim ( xml-layer -- dim )
    [ "width" get-attr ] [ "height" get-attr ] bi 2array [ string>number ] map ;

: correct-dim? ( dim tile-list -- ? ) [ 1 [ * ] reduce ] dip length = ;

: data-bytes ( xml-data -- bytes )
    dup "encoding" get-attr? "base64" =
    [ "Non-base64 encoding not supported." throw ] unless
    first [ blank? ] trim base64> ;

: decompress-data ( data compression -- data )
    { { "zlib" [ zlib-inflate ] }
      { f [ ] }
      [ "Unsupported compression" throw ] } case ;

: unpack-data ( data -- ints ) 4 group [ "i" unpack-le first ] map ;

: list-tiles ( xml-data -- tile-list )
    [ data-bytes ] [ "compression" get-attr? ] bi
    decompress-data unpack-data ;

: >tile-matrix ( dim tile-list -- tile-matrix )
    [ correct-dim? [ "Wrong number of tiles read" throw ] unless ] 2keep
    swap first group ;

: read-layer ( xml-layer -- name tile-matrix )
    [ "name" get-attr ] [ layer-dim ] [ "data" tag-named list-tiles ]
    tri >tile-matrix ;

: (read-tiled) ( xml -- mapdata ) "layer" tags-named [ read-layer 2array ] map ;

PRIVATE>

: read-tiled ( stream -- mapdata ) read-xml (read-tiled) ;

: file>tiled ( path -- mapdata ) file>xml (read-tiled) ;