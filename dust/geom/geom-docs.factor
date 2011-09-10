USING: help.markup help.syntax ;
IN: dust.geom

HELP: overlap-rect
{ $description "Creates a copy of the original rectangle with the minimal translation needed to completely overlap the target rectangle." } ;

HELP: resize-rect
{ $description "Creates a new rect with the size of offset-rect and the original rect's location translated by offset rect's location." } ;