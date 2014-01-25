use std::libc::*;
use std::unstable::intrinsics;
use std::vec;


struct stbtt_fontinfo {
    userdata: *c_void,
    data: *c_uchar,
    fontstart: c_int,
    numGlyphs: c_int,
    loca: c_int,
    head: c_int,
    glyf: c_int,
    hhea: c_int,
    hmtx: c_int,
    kern: c_int,
    index_map: c_int,
    indexToLocFormat: c_int
}

extern {
    fn stbtt_InitFont(
        info: *stbtt_fontinfo, data: *c_uchar, offset: c_int) -> c_int;
    fn stbtt_ScaleForPixelHeight(
        info: *stbtt_fontinfo, pixels: c_float) -> c_float;
    fn stbtt_FindGlyphIndex(
        info: *stbtt_fontinfo, unicode_codepoint: c_int) -> c_int;
    fn stbtt_GetGlyphHMetrics(
        info: *stbtt_fontinfo, glyph_index: c_int,
        advanceWidth: *c_int, leftSideBearing: *c_int);
    fn stbtt_GetGlyphBitmapBox(
        info: *stbtt_fontinfo, glyph: c_int,
        scale_x: c_float, scale_y: c_float,
        ix0: *c_int, iy0: *c_int, ix1: *c_int, iy1: *c_int);
    fn stbtt_MakeGlyphBitmap(info: *stbtt_fontinfo, output: *c_uchar,
        out_w: c_int, out_h: c_int, out_stride: c_int,
        scale_x: c_float, scale_y: c_float, glyph: c_int);
}

pub struct Font {
    priv info: stbtt_fontinfo
}

pub struct Glyph {
    width: int,
    height: int,
    xOffset: f64,
    yOffset: f64,
    xAdvance: f64,
    pixels: ~[u8]
}

impl Font {
    pub fn new(data: &[u8]) -> Option<Font> {
        unsafe {
            let ret = Font {
                info: intrinsics::uninit(),
            };
            let status = stbtt_InitFont(
                &ret.info, data.as_ptr() as *c_uchar, 0 as c_int);
            if status == 0 {
                return None
            }
            return Some(ret);
        }
    }

    pub fn glyph(&self, codepoint: int, height: f64) -> Option<Glyph> {
        unsafe {
            let g = stbtt_FindGlyphIndex(&self.info, codepoint as c_int);
            if g == 0 {
                return None
            }

            let scale = stbtt_ScaleForPixelHeight(
                &self.info, height as c_float);

            let x0: c_int = 0;
            let y0: c_int = 0;
            let x1: c_int = 0;
            let y1: c_int = 0;
            stbtt_GetGlyphBitmapBox(
                &self.info, g, scale as c_float, scale,
                &x0, &y0, &x1, &y1);

            let advance: c_int = 0;
            let lsb: c_int = 0;
            stbtt_GetGlyphHMetrics(&self.info, g, &advance, &lsb);

            let width = (x1 - x0) as int;
            let height = (y1 - y0) as int;

            let pixels = vec::from_elem((width * height) as uint, 0u8);
            stbtt_MakeGlyphBitmap(
                &self.info, pixels.as_ptr() as *c_uchar,
                width as c_int, height as c_int,
                width as c_int, scale, scale, g);

            Some(Glyph{
                width: width,
                height: height,
                xOffset: x0 as f64,
                yOffset: y0 as f64,
                xAdvance: advance as f64 * scale as f64,
                pixels: pixels,
            })
        }
    }
}
