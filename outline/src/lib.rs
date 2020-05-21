mod de;
pub use crate::de::from_outline;

mod ser;
pub use crate::ser::into_outline;

mod outline;
pub use crate::outline::{Outline, INDENT_PREFIX};

mod symbol;
pub use crate::symbol::Sym;

pub type Symbol = Sym<String>;

mod util;
pub use crate::util::normalize_title;

#[cfg(test)]
mod tests;
