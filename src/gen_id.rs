#[macro_escape]

use std::fmt;

#[deriving(Eq, Clone, Default)]
pub struct CodeId {
    line: uint,
    path: String,
}

impl fmt::Show for CodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.path, self.line)
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
