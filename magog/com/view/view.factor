! Copyright (C) 2011 Risto Saarelma

USING: accessors dust.com kernel ;

IN: magog.com.view

TUPLE: view-component < facet-component ;

: <view-component> ( -- view-component ) view-component new-facet-component ;

TUPLE: view name symbol color ;

: <view> ( name symbol color -- view ) view boa ;

M: view add-facet view-component add-to-facet-component ;

: >view ( uid -- view ) view-component get-facet ;