use crate::cache;
use calx;
use std::error::Error;
use std::io;
use vitral::{self, Core, KeyEvent};

pub struct Backend {
    inner: vitral::Backend,
}

impl Backend {
    pub fn start<S: Into<String>>(
        width: u32,
        height: u32,
        title: S,
    ) -> Result<Backend, Box<dyn Error>> {
        let inner = vitral::Backend::start(width, height, title)?;

        Ok(Backend { inner })
    }

    /// Helper method for making a vitral `Core` of the correct type
    pub fn new_core(&mut self) -> Core {
        // Make sure to reuse the existing solid texture so that the Core builder won't do new
        // texture allocations.
        vitral::Builder::new()
            .solid_texture(cache::solid())
            .build(self.inner.canvas_size().cast(), |img| {
                self.inner.make_texture(img)
            })
    }

    /// Return the next keypress event if there is one.
    pub fn poll_key(&mut self) -> Option<KeyEvent> { self.inner.poll_key() }

    /// Display the backend and read input events.
    pub fn update(&mut self, core: &mut Core) -> bool {
        cache::ATLAS.with(|a| self.inner.sync_with_atlas_cache(&mut a.borrow_mut()));
        self.inner.update(core)
    }

    pub fn save_screenshot(&self, basename: &str) -> io::Result<()> {
        calx::save_screenshot(basename, &self.inner.screenshot().into())
    }
}
