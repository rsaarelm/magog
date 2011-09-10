! Copyright (C) 2011 Risto Saarelma

USING: magog.tiledata ;

IN: magog.tile

!           char  foreground   background  walk? fly? see? spread?
TILE: grass   0x05    #0a0           #000            t t t f
TILE: floor   0x05    #aaa           #000            t t t f
TILE: sand    0x05    #ff5           #000            t t t f
TILE: forest  0x07    #2a0           #000            t t f f
TILE: wall    0x01    #888           #000            f f f t
TILE: rock    0x09    #840           #000            f f f t
TILE: water   0x06    #3af           #058            f t t f
TILE: pit     0x08    #aa0           #000            f t t f
TILE: tree    0x0d    #080           #000            f t t f
TILE: ethereal-void 0x08 #a0a        #000            f f f f
TILE: slope0  0x23    #aaa           #000            t t t f
TILE: slope1  0x24    #aaa           #000            t t t f
TILE: slope2  0x25    #aaa           #000            t t t f
TILE: slope3  0x26    #aaa           #000            t t t f
TILE: slope4  0x27    #aaa           #000            t t t f
TILE: slope5  0x28    #aaa           #000            t t t f