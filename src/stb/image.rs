use std::vec::Vec;
use libc::{c_uchar, c_void, c_int, c_char, free};
use std::slice::raw::buf_as_slice;
use std::io::File;
use std::mem;

#[link(name="stb")]
extern {
    fn stbi_load_from_memory(
        buffer: *const c_uchar, len: c_int, x: *mut c_int, y: *mut c_int,
        comp: *mut c_int, req_comp: c_int) -> *const c_uchar;

    fn stbi_write_png(
        filename: *const c_char, w: c_int, h: c_int, comp: c_int,
        data: *const c_void, stride_in_bytes: c_int);
}

pub struct Image {
    pub width: uint,
    pub height: uint,
    pub bpp: uint,
    pub pixels: Vec<u8>,
}

impl Image {
    pub fn load(path: &str, force_channels: uint) -> Option<Image> {
        let path = Path::new(path);
        if !path.exists() { return None; }
        let data = File::open(&path).read_to_end().unwrap();
        Image::load_from_memory(data.as_slice(), force_channels)
    }

    pub fn load_from_memory(data: &[u8], force_channels: uint) -> Option<Image> {
        unsafe {
            let mut w = 0 as c_int;
            let mut h = 0 as c_int;
            let mut bpp = 0 as c_int;

            let buffer = stbi_load_from_memory(
                data.as_ptr(), data.len() as c_int,
                &mut w, &mut h, &mut bpp, force_channels as c_int);

            if buffer.is_null() {
                return None
            }

            let bpp = if force_channels != 0 { force_channels } else { bpp as uint };

            let ret = Some(Image{
                width: w as uint,
                height: h as uint,
                bpp: bpp,
                pixels: buf_as_slice(
                    mem::transmute(buffer), (w * h) as uint * bpp, |s| Vec::from_slice(s)),
            });
            free(buffer as *mut c_void);
            ret
        }
    }

    pub fn new(width: uint, height: uint, bpp: uint) -> Image {
        assert!(bpp <= 4);
        Image{
            width: width,
            height: height,
            bpp: bpp,
            pixels: Vec::from_elem(width * height * bpp, 0u8),
        }
    }

    pub fn save_png(&self, path: &str) {
        unsafe {
            stbi_write_png(
                path.to_c_str().as_ptr(), self.width as c_int, self.height as c_int,
                self.bpp as c_int, self.pixels.as_ptr() as *const c_void, 0);
        }
    }
}
