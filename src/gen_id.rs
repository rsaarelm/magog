#[macro_escape]

use std::fmt::{Show, Formatter, Result};

#[deriving(Eq, Clone, Default)]
pub struct CodeId {
    line: uint,
    path: ~str,
}

impl Show for CodeId {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f.buf, "{}:{}", self.path, self.line)
    }
}

/// Generate an identifier from the current path and line number.
/// Used to uniquely identify source code locations, useful for doing
/// IMGUI.
/// Caveats: Will not create unique identifiers if called twice on the
/// same line of code. May cause collisions in a project with several
/// source files with the same name and path (can get this with
/// different crates built in different source trees?)
#[macro_export]
macro_rules! gen_id(
    () => ( gen_id::CodeId { line: line!(), path: file!().to_owned() } )
)
