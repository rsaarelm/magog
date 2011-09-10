! Copyright (C) 2011 Risto Saarelma

USING: kernel math math.order math.ranges memoize sequences ;

IN: dust.keyboard

<PRIVATE

: reverse-keymap ( keymap -- rev-keymap ) 32 126 [a,b] swap [ index 32 + ] curry map ;

PRIVATE>

CONSTANT: colemak>qwerty-map
" !\"#$%&'()*+,-./0123456789Pp<=>?@ABCGKETHLYNUMJ:RQSDFIVWXOZ[\\]^_`abcgkethlynumj;rqsdfivwxoz{|}~"

MEMO: qwerty>colemak-map ( -- map ) colemak>qwerty-map reverse-keymap ;

CONSTANT: dvorak>qwerty-map
" !Q#$%&q()*}w'e[0123456789ZzW]E{@ANIHDYUJGCVPMLSRXO:KF><BT?/\\=^\"`anihdyujgcvpmlsrxo;kf.,bt/_|+~"

MEMO: qwerty>dvorak-map ( -- map ) dvorak>qwerty-map reverse-keymap ;

: ascii-remap ( key keymap -- key' )
    over 32 127 between? [ [ 32 - ] dip nth ] [ drop ] if ;

: colemak-key ( keystring -- keystring' )
    [ qwerty>colemak-map ascii-remap ] map ;
