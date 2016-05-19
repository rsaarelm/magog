# Vitral

A backend-agnostic immediate mode GUI library.

## Working with raw font files

Using ImageMagick, convert raw font to png:

    convert -depth 8 -size 96x72 a:src/profont9.raw profont9.png

Convert png back to raw:

    convert profont9.png a:src/profont9.raw

(The size value depends on the dimensions of your font sheet.)
