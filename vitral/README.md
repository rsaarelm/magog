# Vitral

A backend-agnostic immediate mode GUI library.

## Working with raw font files

Using ImageMagick, convert raw font to png:

    convert -depth 8 -size 96x48 a:src/font-96x48.raw font.png

Convert png back to raw:

    convert font.png a:src/font-96x48.raw

(The size value depends on the dimensions of your font sheet.)
