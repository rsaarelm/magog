use std::libc::*;
use std::ptr::{is_null, to_mut_unsafe_ptr};
use std::vec::raw::from_buf_raw;

extern {
    fn stbi_load_from_memory(
        buffer: *c_uchar, len: c_int, x: *mut c_int, y: *mut c_int,
        comp: *mut c_int, req_comp: c_int) -> *c_uchar;
}

pub struct Image {
    width: uint,
    height: uint,
    bpp: uint,
    pixels: ~[u8],
}

impl Image {
    pub fn new(data: &[u8]) -> Option<Image> {
        unsafe {
            let mut w = 0 as c_int;
            let mut h = 0 as c_int;
            let mut bpp = 0 as c_int;

            let buffer = stbi_load_from_memory(
                data.as_ptr() as *c_uchar, data.len() as c_int,
                to_mut_unsafe_ptr(&mut w),
                to_mut_unsafe_ptr(&mut h),
                to_mut_unsafe_ptr(&mut bpp),
                0);

            if is_null(buffer) {
                return None
            }

            let ret = Some(Image{
                width: w as uint,
                height: h as uint,
                bpp: bpp as uint,
                pixels: from_buf_raw(buffer, (w * h * bpp) as uint)
            });
            free(buffer as *mut c_void);
            ret
        }
    }
}
