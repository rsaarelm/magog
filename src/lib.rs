extern crate euclid;
extern crate image;

use std::mem;
use std::collections::HashMap;
use std::ops::Add;
use image::{GenericImage, Pixel};
use euclid::{Point2D, Rect, Size2D};

pub type ImageBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

mod atlas;

// Assuming this is the texture size maximum for low-end GPUs.
static ATLAS_SIZE_LIMIT: u32 = 2048;

/// Configuration for rendering style.
#[derive(Clone, PartialEq)]
pub struct Style {
    pub foreground_color: Option<[f32; 4]>,
    pub background_color: Option<[f32; 4]>,
    pub font: Option<Font>,
}

// TODO: Should there be a separate field that gets added to the core style
// with non-optional fields that has optional fields? The current scheme with
// Option-only doesn't encode "cached style *must* have non-option value for
// everything".

impl Default for Style {
    fn default() -> Self {
        Style {
            foreground_color: Some([1.0, 1.0, 1.0, 1.0]),
            background_color: Some([0.0, 0.0, 0.0, 1.0]),
            font: Some(Font(0)),
        }
    }
}

impl Add<Style> for Style {
    type Output = Style;
    fn add(self, other: Style) -> Style {
        unimplemented!();
    }
}


pub struct Builder<T> {
    fonts: Vec<FontData<T>>,
    images: Vec<ImageBuffer>,
}

impl<T> Builder<T>
    where T: Clone + PartialEq
{
    pub fn new() -> Builder<T> {
        Builder {
            fonts: Vec::new(),
            images: vec![
                image::ImageBuffer::from_pixel(
                    1, 1, image::Rgba::from_channels(255, 255, 255, 255)),
            ],
        }
    }

    /// Add an image for the UI to use.
    ///
    /// The return value is a handle that can be used to request the images to
    /// be drawn later.
    pub fn add_image<I>(&mut self, img: &I) -> Image
        where I: image::GenericImage<Pixel = image::Rgba<u8>>
    {
        let mut image = ImageBuffer::new(img.width(), img.height());
        image.copy_from(img, 0, 0);
        if image.width() > ATLAS_SIZE_LIMIT ||
           image.height() > ATLAS_SIZE_LIMIT {
            panic!("Image with dimensions ({}, {}) is too large, maximum is ({}, {})",
                   image.width(),
                   image.height(),
                   ATLAS_SIZE_LIMIT,
                   ATLAS_SIZE_LIMIT);
        }

        self.images.push(image);
        Image(self.images.len() - 1)
    }

    /// Add a font for the UI to use.
    ///
    /// The return value is a font handle or an error if the data was invalid.
    pub fn add_font<F, R>(&mut self,
                          mut make_t: F,
                          ttf_data: &[u8],
                          font_range: R)
                          -> Result<Font, ()>
        where F: FnMut(ImageBuffer) -> T,
              R: IntoIterator<Item = char>
    {
        // Ensure that the first font is always the baked-in default one.
        if self.fonts.is_empty() {
            self.add_default_font(&mut make_t);
        }

        // TODO: A FontSource type that embody a TTF font or a bitmap font.

        // TODO: Parse TTF data using appropriate crate, return error if data
        // isn't valid TTF.

        // TODO: Rasterize fonts with codepoints in font_range into images.

        // TODO: Build atlas image from font and register it in the Builder.
        unimplemented!();
    }

    fn add_default_font<F>(&mut self, make_t: &mut F)
        where F: FnMut(ImageBuffer) -> T
    {
        assert!(self.fonts.is_empty());

        static DEFAULT_FONT: &'static [u8] = include_bytes!("unscii16-256x112.raw");
        let (width, height) = (256, 112);
        let start_char = 32;
        let end_char = 127;
        let (char_width, char_height) = (8, 16);
        let columns = width / char_width;

        let img = image::ImageBuffer::from_fn(width, height, |x, y| {
            let a = DEFAULT_FONT[(x + y * width) as usize];
            image::Rgba::from_channels(a, a, a, a)
        });

        let t = make_t(img);

        let mut map = HashMap::new();

        for i in start_char..end_char {
            let x = char_width * ((i - start_char) % columns);
            let y = char_height * ((i - start_char) / columns);

            let texcoords = Rect::new(Point2D::new(x as f32 / width as f32,
                                                   y as f32 / height as f32),
                                      Size2D::new(char_width as f32 /
                                                  width as f32,
                                                  char_height as f32 /
                                                  height as f32));

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
    }

    fn construct_atlas<F>(&self, make_t: &mut F) -> Vec<ImageData<T>>
        where F: FnMut(ImageBuffer) -> T
    {
        // TODO: Handle case where you need multiple atlases.
        let (atlas, positions) = atlas::build_atlas(&self.images,
                                                    ATLAS_SIZE_LIMIT)
                                     .unwrap();
        let w = atlas.width() as f32;
        let h = atlas.height() as f32;
        let atlas = make_t(atlas);

        positions.iter()
                 .enumerate()
                 .map(|(i, p)| {
                     let (i_w, i_h) = (self.images[i].width(),
                                       self.images[i].height());
                     ImageData {
                         texture: atlas.clone(),
                         size: Size2D::new(i_w, i_h),
                         texcoords: Rect::new(Point2D::new(p.x as f32 / w,
                                                           p.y as f32 / h),
                                              Size2D::new(i_w as f32 / w,
                                                          i_h as f32 / h)),
                     }
                 })
                 .collect()
    }

    /// Construct an interface context instance.
    pub fn build<F, V>(mut self, mut make_t: F) -> Context<T, V>
        where F: FnMut(ImageBuffer) -> T,
              V: Vertex
    {
        if self.fonts.is_empty() {
            self.add_default_font(&mut make_t);
        }

        let images = self.construct_atlas(&mut make_t);

        Context::new(self.fonts, images)
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

    styles: Vec<Style>,
    current_style: Style,
}

impl<T, V: Vertex> Context<T, V>
    where T: Clone + PartialEq
{
    fn new(fonts: Vec<FontData<T>>,
           images: Vec<ImageData<T>>)
           -> Context<T, V> {
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

            styles: Vec::new(),
            current_style: Default::default(),
        }
    }

    pub fn begin_frame(&mut self) {
        self.tick += 1;

        self.layout_pos = Point2D::new(10.0, 10.0);

        // TODO
    }

    pub fn draw_text(&mut self,
                     font: Font,
                     mut pos: Point2D<f32>,
                     color: [f32; 4],
                     text: &str) {
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
                self.push_rect(Rect::new(pos - f.draw_offset,
                                         Size2D::new(f.advance, h)),
                               f.texcoords,
                               color);
                pos.x += f.advance;
            }
        }
    }

    pub fn default_font(&self) -> Font {
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
        let press = self.click_state.is_pressed() &&
                    area.contains(&self.mouse_pos);

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

    pub fn draw_image(&mut self,
                      image: &ImageData<T>,
                      pos: Point2D<f32>,
                      color: [f32; 4]) {
        self.start_texture(image.texture.clone());
        let size = Size2D::new(image.size.width as f32,
                               image.size.height as f32);
        self.push_rect(Rect::new(pos, size), image.texcoords, color);
    }

    fn push_rect(&mut self,
                 area: Rect<f32>,
                 texcoords: Rect<f32>,
                 color: [f32; 4]) {
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
        self.push_rect(area, tex_rect, color);
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
                self.click_state = self.click_state
                                       .input_release(self.mouse_pos);
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

    /// Add a style, any set values will override existing styles.
    pub fn push_style(&mut self, style: Style) {
        self.styles.push(style);
        self.recompute_style();
    }

    /// Remove the latest pushed style and revert to the previous one.
    pub fn pop_style(&mut self) {
        self.styles.pop();
        self.recompute_style();
    }

    fn recompute_style(&mut self) {
        let mut style = Style::default();
        for i in self.styles.iter() {
            style = style + i.clone();
        }

        self.current_style = style;
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

#[derive(Clone, PartialEq)]
pub struct ImageData<T> {
    pub texture: T,
    pub size: Size2D<u32>,
    pub texcoords: Rect<f32>,
}
