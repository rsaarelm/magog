use opengles::gl2;

#[cfg(check_gl)]
pub fn gl_check_and_fail(site: &str) {
    let err = gl2::get_error();
    if err != gl2::NO_ERROR {
	fail!("OpenGL error at '{}': {}", site, err);
    }
}

#[cfg(not(check_gl))]
pub fn gl_check_and_fail(_site: &str) { }

macro_rules! gl_check
(
    ($func:expr) =>
    ({
	let ret = $func;
	gl_check::gl_check_and_fail(stringify!($func));
	ret
    });
)
