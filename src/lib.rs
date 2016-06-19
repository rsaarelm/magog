extern crate euclid;
extern crate image;

use std::mem;
use std::ops::Range;
use std::collections::HashMap;
use image::{Pixel, GenericImage};
use euclid::{Rect, Point2D, Size2D};

/// Configuration for rendering style.
#[derive(Clone, PartialEq)]
pub struct Style {
    pub foreground_color: [f32; 4],
    pub background_color: [f32; 4],
    pub font: Font,
    // Private field so that the struct doesn't show up as fully public and
    // fixed in the visible API.
    _reserved: std::marker::PhantomData<()>,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            foreground_color: [1.0, 1.0, 1.0, 1.0],
            background_color: [0.0, 0.0, 0.0, 1.0],
            font: Font(0),
            _reserved: std::marker::PhantomData,
        }
    }
}


pub struct Builder<T> {
    fonts: Vec<FontData<T>>,
    images: Vec<ImageData<T>>,
}

impl<T> Builder<T>
    where T: Clone + PartialEq
{
    pub fn new() -> Builder<T> {
        Builder {
            fonts: Vec::new(),
            images: Vec::new(),
        }
    }

    /// Add a series of images for the UI to use.
    ///
    /// The return value is a series of handles corresponding to the input
    /// images that can be used to request the images to be drawn later.
    ///
    /// make_t is a function that converts an image into a host texture
    /// resource and returns a host texture handle.
    pub fn add_images<F, I>(&mut self, make_t: F, images: Vec<I>) -> Vec<Image>
        where F: FnMut(I) -> T,
              I: image::GenericImage<Pixel = image::Rgba<u8>>
    {
        // TODO: If this is the very first call to add_images, add a
        // single-pixel opaque white texture as the very first image. Keep
        // track of its index internally but make sure not to return it as
        // Image value in the return vector. This will be used to draw
        // solid-color shapes.

        // TODO: Build atlas image from images and register it in the Builder.

        // TODO: Return Image values to the caller. Make sure not to return
        // the extra one-pixel image if that was generated.
        unimplemented!();
    }

    /// Add a font for the UI to use.
    ///
    /// The return value is a font handle or an error if the data was invalid.
    pub fn add_font<F, I, R>(&mut self,
                             make_t: F,
                             ttf_data: &[u8],
                             font_range: R)
                             -> Result<Font, ()>
        where F: FnMut(I) -> T,
              I: image::GenericImage<Pixel = image::Rgba<u8>>,
              R: IntoIterator<Item = char>
    {
        // TODO: A FontSource type that embody a TTF font or a bitmap font.

        // TODO: Parse TTF data using appropriate crate, return error if data
        // isn't valid TTF.

        // TODO: Rasterize fonts with codepoints in font_range into images.

        // TODO: Build atlas image from font and register it in the Builder.
        unimplemented!();
    }

    /// Construct an interface context instance.
    pub fn build<F, V>(mut self, mut make_t: F) -> Context<T, V>
        where F: FnMut(image::ImageBuffer<image::Rgba<u8>, Vec<u8>>) -> T,
              V: Vertex
    {
        // TODO: Use make_t to generate default font into font index 0.

        // TODO: Only add the solid image here if it hasn't been already added
        // in the first add_images call.
        let solid = image::ImageBuffer::from_pixel(1,
                                                   1,
                                                   image::Rgba::from_channels(255, 255, 255, 255));
        let solid_t = make_t(solid);
        self.images.push(ImageData {
            texture: solid_t,
            size: Size2D::new(1, 1),
            texcoords: Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1.0, 1.0)),
        });


        // TODO: If no add_images was called, do a single pixel solid texture
        // for the solid color draw.

        Context::new(self.fonts, self.images)
    }
}


/// An immediate mode graphical user interface context.
///
/// The context persists over a frame and receives commands that combine GUI
/// description and input handling. At the end of the frame, the commands are
/// converted into rendering instructions for the GUI.
pub struct Context<T, V> {
    draw_list: Vec<DrawBatch<T, V>>,
    /// Texture value used for solid-color drawing
    layout_pos: Point2D<f32>,

    mouse_pos: Point2D<f32>,
    click_state: ClickState,

    fonts: Vec<FontData<T>>,
    images: Vec<ImageData<T>>,

    text_input: Vec<KeyInput>,

    tick: u64,
}

impl<T, V: Vertex> Context<T, V>
    where T: Clone + PartialEq
{
    fn new(fonts: Vec<FontData<T>>, images: Vec<ImageData<T>>) -> Context<T, V> {
        Context {
            draw_list: Vec::new(),
            // solid_texture: Image(0),
            layout_pos: Point2D::new(0.0, 0.0),

            mouse_pos: Point2D::new(0.0, 0.0),
            click_state: ClickState::Unpressed,

            fonts: fonts,
            images: images,

            text_input: Vec::new(),

            tick: 0,
        }
    }

    pub fn begin_frame(&mut self) {
        self.tick += 1;

        self.layout_pos = Point2D::new(10.0, 10.0);

        // TODO
    }

    pub fn draw_text(&mut self, font: Font, mut pos: Point2D<f32>, color: [f32; 4], text: &str) {
        assert!(self.fonts.len() >= font.0);
        let id = font.0;
        let t = self.fonts[id].texture.clone();
        let h = self.fonts[id].height;
        self.start_texture(t);

        for c in text.chars() {
            // FIXME: Gratuitous cloning because of borrow checker.
            let x = self.fonts[id].chars.get(&c).cloned();
            // TODO: Draw some sort of symbol for characters missing from font.
            if let Some(f) = x {
                self.tex_rect(Rect::new(pos - f.draw_offset, Size2D::new(f.advance, h)),
                              f.texcoords,
                              color);
                pos.x += f.advance;
            }
        }
    }

    fn default_font(&self) -> Font {
        Font(0)
    }

    pub fn button(&mut self, caption: &str) -> bool {
        let font = self.default_font();
        let area = self.fonts[font.0]
                       .render_size(caption)
                       .inflate(4.0, 4.0)
                       .translate(&self.layout_pos);

        self.layout_pos.y += area.size.height + 2.0;

        let hover = area.contains(&self.mouse_pos);
        let press = self.click_state.is_pressed() && area.contains(&self.mouse_pos);

        let color = if press {
            [1.0, 1.0, 0.0, 1.0]
        } else if hover {
            [0.5, 1.0, 0.0, 1.0]
        } else {
            [0.0, 1.0, 0.0, 1.0]
        };

        self.fill_rect(area, color);
        self.fill_rect(area.inflate(-1.0, -1.0), [0.0, 0.0, 0.0, 1.0]);
        self.draw_text(font,
                       area.origin + Point2D::new(4.0, area.size.height - 4.0),
                       color,
                       caption);

        press && self.click_state.is_release()
    }

    pub fn tex_rect(&mut self, area: Rect<f32>, texcoords: Rect<f32>, color: [f32; 4]) {
        let (p1, p2) = (area.origin, area.bottom_right());
        let (t1, t2) = (texcoords.origin, texcoords.bottom_right());
        let idx = self.draw_list.len() - 1;
        let batch = &mut self.draw_list[idx];
        let idx_offset = batch.vertices.len() as u16;

        batch.vertices.push(Vertex::new([p1.x, p1.y], color, [t1.x, t1.y]));
        batch.vertices.push(Vertex::new([p2.x, p1.y], color, [t2.x, t1.y]));
        batch.vertices.push(Vertex::new([p2.x, p2.y], color, [t2.x, t2.y]));
        batch.vertices.push(Vertex::new([p1.x, p2.y], color, [t1.x, t2.y]));

        batch.triangle_indices.push(idx_offset);
        batch.triangle_indices.push(idx_offset + 1);
        batch.triangle_indices.push(idx_offset + 2);

        batch.triangle_indices.push(idx_offset);
        batch.triangle_indices.push(idx_offset + 2);
        batch.triangle_indices.push(idx_offset + 3);
    }

    pub fn fill_rect(&mut self, area: Rect<f32>, color: [f32; 4]) {
        self.start_solid_texture();
        // Image 0 must be solid texture.
        let tex_rect = self.images[0].texcoords;
        self.tex_rect(area, tex_rect, color);
    }

    pub fn text_input(&mut self,
                      font: Font,
                      pos: Point2D<f32>,
                      color: [f32; 4],
                      text_buffer: &mut String) {
        // TODO: Focus system. Only accept input if current input widget is focused.
        // (Also needs widget identifiers to know which is which.)
        for c in self.text_input.iter() {
            match *c {
                KeyInput::Printable(c) => {
                    if c >= ' ' {
                        text_buffer.push(c);
                    }
                }
                KeyInput::Other(Keycode::Backspace) => {
                    text_buffer.pop();
                }
                KeyInput::Other(_) => {}
            }
        }

        // TODO: Option to draw cursor mid-string (font may be
        // variable-width...), track cursor pos somehow (external ref or
        // internal cache)

        // TODO: Arrow keys move cursor

        // TODO: Filter function for input, eg. numbers only.

        // Nasty hack to show a blinking cursor. Will only work for cursor
        // always at the end of the input.

        // TODO: Maybe want to use wall clock time instead of GUI context ticks for this?
        if (self.tick / 30) % 3 == 0 {
            self.draw_text(font, pos, color, text_buffer);
        } else {
            self.draw_text(font, pos, color, &format!("{}_", text_buffer));
        }
    }

    fn start_solid_texture(&mut self) {
        assert!(self.images.len() > 0);
        // Builder must always setup Context so that the first image is the
        // solid color.
        let tex = self.images[0].texture.clone();
        self.start_texture(tex);
    }

    /// Ensure that there current draw batch has solid texture.
    fn start_texture(&mut self, texture: T) {
        // TODO: Actually have the solid texture value stashed somewhere.
        if self.draw_list.is_empty() ||
           self.draw_list[self.draw_list.len() - 1].texture != texture {
            self.draw_list.push(DrawBatch {
                texture: texture,
                clip: None,
                vertices: Vec::new(),
                triangle_indices: Vec::new(),
            });
        }
    }

    pub fn end_frame(&mut self) -> Vec<DrawBatch<T, V>> {
        // Clean up transient mouse click info.
        self.click_state = self.click_state.tick();

        // Clean up text buffer
        self.text_input.clear();

        let mut ret = Vec::new();
        mem::swap(&mut ret, &mut self.draw_list);
        ret
    }

    /// Register mouse button state.
    pub fn input_mouse_button(&mut self, id: MouseButton, is_down: bool) {
        if id == MouseButton::Left {
            if is_down {
                self.click_state = self.click_state.input_press(self.mouse_pos);
            } else {
                self.click_state = self.click_state.input_release(self.mouse_pos);
            }
        }
        // TODO handle other buttons
    }

    /// Register mouse motion.
    pub fn input_mouse_move(&mut self, x: i32, y: i32) {
        self.mouse_pos = Point2D::new(x as f32, y as f32);
    }

    /// Register printable character input.
    pub fn input_char(&mut self, c: char) {
        self.text_input.push(KeyInput::Printable(c));
    }

    /// Register a nonprintable key state.
    pub fn input_key_state(&mut self, k: Keycode, is_down: bool) {
        if is_down {
            self.text_input.push(KeyInput::Other(k));
        }
    }

    /// Build a font atlas from a TTF and construct a texture object.
    ///
    /// The output will be have 8 alpha channel bits per pixel.
    ///
    /// TODO: Font customization, point size, character ranges.
    pub fn init_font<F>(&mut self,
                        ttf_data: &[u8],
                        point_size: f32,
                        chars: Range<usize>,
                        register_texture: F)
                        -> Result<Font, ()>
        where F: FnOnce(&[u8], u32, u32) -> T
    {
        unimplemented!();
    }

    pub fn init_default_font<F>(&mut self, register_texture: F) -> Font
        where F: FnOnce(&[u8], u32, u32) -> T
    {
        static DEFAULT_FONT: &'static [u8] = include_bytes!("unscii16-256x112.raw");
        let (width, height) = (256, 112);
        let start_char = 32;
        let end_char = 127;
        let (char_width, char_height) = (8, 16);
        let columns = width / char_width;

        let t = register_texture(DEFAULT_FONT, width, height);

        let mut map = HashMap::new();

        for i in start_char..end_char {
            let x = char_width * ((i - start_char) % columns);
            let y = char_height * ((i - start_char) / columns);

            let texcoords = Rect::new(Point2D::new(x as f32 / width as f32,
                                                   y as f32 / height as f32),
                                      Size2D::new(char_width as f32 / width as f32,
                                                  char_height as f32 / height as f32));

            map.insert(std::char::from_u32(i).unwrap(),
                       CharData {
                           texcoords: texcoords,
                           draw_offset: Point2D::new(0.0, char_height as f32),
                           advance: char_width as f32,
                       });
        }

        self.fonts.push(FontData {
            texture: t,
            chars: map,
            height: char_height as f32,
        });

        Font(self.fonts.len() - 1)
    }
}

/// A sequence of primitive draw operarations.
pub struct DrawBatch<T, V> {
    /// Texture used for the current batch, details depend on backend
    /// implementation
    pub texture: T,
    /// Clipping rectangle for the current batch
    pub clip: Option<Rect<f32>>,
    /// Vertex data
    pub vertices: Vec<V>,
    /// Indices into the vertex array for the triangles that make up the batch
    pub triangle_indices: Vec<u16>,
}

pub trait Vertex {
    fn new(pos: [f32; 2], color: [f32; 4], texcoord: [f32; 2]) -> Self;
}

/// Text alignment.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum ClickState {
    Unpressed,
    Press(Point2D<f32>),
    Drag(Point2D<f32>),
    Release(Point2D<f32>, Point2D<f32>),
}

impl ClickState {
    fn tick(self) -> ClickState {
        match self {
            ClickState::Unpressed => ClickState::Unpressed,
            ClickState::Press(p) => ClickState::Drag(p),
            ClickState::Drag(p) => ClickState::Drag(p),
            ClickState::Release(_, _) => ClickState::Unpressed,
        }
    }

    fn input_press(self, pos: Point2D<f32>) -> ClickState {
        match self {
            ClickState::Unpressed => ClickState::Press(pos),
            ClickState::Press(p) => ClickState::Drag(p),
            ClickState::Drag(p) => ClickState::Drag(p),
            ClickState::Release(_, _) => ClickState::Press(pos),
        }
    }

    fn input_release(self, pos: Point2D<f32>) -> ClickState {
        match self {
            ClickState::Unpressed => ClickState::Unpressed,
            ClickState::Press(p) => ClickState::Release(p, pos),
            ClickState::Drag(p) => ClickState::Release(p, pos),
            ClickState::Release(p, _) => ClickState::Release(p, pos),
        }
    }

    fn is_pressed(&self) -> bool {
        match *self {
            ClickState::Press(_) |
            ClickState::Drag(_) |
            ClickState::Release(_, _) => true,
            ClickState::Unpressed => false,
        }
    }

    fn is_release(&self) -> bool {
        if let ClickState::Release(_, _) = *self {
            true
        } else {
            false
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Keycode {
    Tab,
    Shift,
    Ctrl,
    Enter,
    Backspace,
    Del,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum KeyInput {
    Printable(char),
    Other(Keycode),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Font(usize);

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Image(usize);

pub struct FontData<T> {
    texture: T,
    chars: HashMap<char, CharData>,
    height: f32,
}

impl<T> FontData<T> {
    pub fn render_size(&self, text: &str) -> Rect<f32> {
        let mut w = 0.0;

        for c in text.chars() {
            if let Some(f) = self.chars.get(&c) {
                w += f.advance;
            }
        }

        Rect::new(Point2D::new(0.0, 0.0), Size2D::new(w, self.height))
    }
}

#[derive(Clone, Debug)]
struct CharData {
    texcoords: Rect<f32>,
    draw_offset: Point2D<f32>,
    advance: f32,
}

struct ImageData<T> {
    texture: T,
    size: Size2D<u32>,
    texcoords: Rect<f32>,
}
