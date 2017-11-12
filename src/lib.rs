#[cfg(feature = "image")]
extern crate image;

extern crate euclid;
extern crate time;

use euclid::{Point2D, point2, Rect, rect, Size2D, Vector2D, vec2};
use std::collections::HashMap;
use std::iter;
use std::mem;

mod atlas;
pub use atlas::Atlas;

mod rect_util;
pub use rect_util::RectUtil;

/// Drawable image data for Vitral.
#[derive(Clone, PartialEq)]
pub struct ImageData<T> {
    pub texture: T,
    pub size: Size2D<u32>,
    pub tex_coords: Rect<f32>,
}

/// Simple 32-bit image container.
///
/// The pixel data structure is RGBA.
#[derive(Clone, Eq, PartialEq)]
pub struct ImageBuffer {
    /// Image size.
    pub size: Size2D<u32>,
    /// RGBA pixels, in rows from top left down, len must be width * height.
    pub pixels: Vec<u32>,
}

impl ImageBuffer {
    /// Build an empty buffer.
    pub fn new(width: u32, height: u32) -> ImageBuffer {
        ImageBuffer {
            size: Size2D::new(width, height),
            pixels: iter::repeat(0u32).take((width * height) as usize).collect(),
        }

    }

    /// Build the buffer from a function.
    pub fn from_fn<F>(width: u32, height: u32, f: F) -> ImageBuffer
    where
        F: Fn(u32, u32) -> u32,
    {
        let pixels = (0..)
            .take((width * height) as usize)
            .map(|i| f(i % width, i / width))
            .collect();
        ImageBuffer {
            size: Size2D::new(width, height),
            pixels,
        }
    }

    /// Build the buffer from RGBA pixel iterator.
    pub fn from_iter<I>(width: u32, height: u32, pixels: &mut I) -> ImageBuffer
    where
        I: Iterator<Item = u32>,
    {
        ImageBuffer {
            size: Size2D::new(width, height),
            pixels: pixels.take((width * height) as usize).collect(),
        }
    }

    /// Copy all pixels from source buffer to self starting at given coordinates in self.
    pub fn copy_from(&mut self, source: &ImageBuffer, x: u32, y: u32) {
        let blit_rect: Rect<u32> = rect(x, y, source.size.width, source.size.height);

        if let Some(blit_rect) = blit_rect.intersection(
            &rect(0, 0, self.size.width, self.size.height),
        )
        {
            for y2 in blit_rect.min_y()..blit_rect.max_y() {
                for x2 in blit_rect.min_x()..blit_rect.max_x() {
                    let self_idx = (x2 + y2 * self.size.width) as usize;
                    let source_idx = ((x2 - x) + (y2 - y) * source.size.width) as usize;
                    self.pixels[self_idx] = source.pixels[source_idx];
                }
            }
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        self.pixels[(x + y * self.size.width) as usize]
    }
}

/// Builder for Vitral `Core` structure.
pub struct Builder<T> {
    user_solid: Option<ImageData<T>>,
}

impl<T> Builder<T>
where
    T: Clone + Eq,
{
    pub fn new() -> Builder<T> { Builder { user_solid: None } }

    /// Give your own `ImageData` for the solid texture.
    ///
    /// You want to use this if you have an image atlas and you want to have both drawing solid
    /// shapes and textured shapes use the same texture resource and go to the same draw batch.
    pub fn solid_texture(mut self, solid: ImageData<T>) -> Builder<T> {
        self.user_solid = Some(solid);
        self
    }


    /// Construct an interface context instance.
    ///
    /// Needs to be provided a texture creation function. If the user has not specified them
    /// earlier, this will be used to construct a separate texture for the solid color and a
    /// default font texture.
    pub fn build<F, V: Vertex>(self, screen_size: Size2D<f32>, mut make_t: F) -> Core<T, V>
    where
        F: FnMut(ImageBuffer) -> T,
    {
        let solid;
        if let Some(user_solid) = self.user_solid {
            solid = user_solid;
        } else {
            solid = ImageData {
                texture: make_t(ImageBuffer::from_fn(1, 1, |_, _| 0xffffffff)),
                size: Size2D::new(1, 1),
                tex_coords: rect(0.0, 0.0, 1.0, 1.0),
            };
        }

        Core::new(solid, screen_size)
    }
}

/// Build the default Vitral font given a texture constructor.
pub fn build_default_font<F, T: Clone>(mut make_t: F) -> FontData<T>
where
    F: FnMut(ImageBuffer) -> T,
{
    static DEFAULT_FONT: &'static [u8] = include_bytes!("font-96x48.raw");
    let (char_width, char_height) = (6, 8);
    let (width, height) = (char_width * 16, char_height * 6);
    let start_char = 32;
    let end_char = 127;
    let columns = width / char_width;

    let img = ImageBuffer::from_fn(width, height, |x, y| {
        let a = DEFAULT_FONT[(x + y * width) as usize] as u32;
        (a << 24) | (a << 16) | (a << 8) | a
    });

    let t = make_t(img);

    let mut map = HashMap::new();

    for i in start_char..end_char {
        let x = char_width * ((i - start_char) % columns);
        let y = char_height * ((i - start_char) / columns);

        let tex_coords = rect(
            x as f32 / width as f32,
            y as f32 / height as f32,
            char_width as f32 / width as f32,
            char_height as f32 / height as f32,
        );

        map.insert(
            std::char::from_u32(i).unwrap(),
            CharData {
                image: ImageData {
                    texture: t.clone(),
                    size: Size2D::new(char_width, char_height),
                    tex_coords: tex_coords,
                },
                draw_offset: vec2(0.0, 0.0),
                advance: char_width as f32,
            },
        );
    }

    FontData {
        chars: map,
        height: char_height as f32,
    }
}

pub trait Vertex {
    /// Custom constructor with exactly the fields Vitral cares about.
    fn new(pos: Point2D<f32>, tex_coord: Point2D<f32>, color: [f32; 4]) -> Self;
}

/// An immediate mode graphical user interface context.
///
/// The context persists over a frame and receives commands that combine GUI
/// description and input handling. At the end of the frame, the commands are
/// converted into rendering instructions for the GUI.
pub struct Core<T, V> {
    draw_list: Vec<DrawBatch<T, V>>,

    mouse_pos: Point2D<f32>,
    click_state: [ClickState; 3],

    solid_texture: ImageData<T>,

    text_input: Vec<KeyInput>,

    tick: u64,

    clip: Option<Rect<f32>>,
    screen_size: Size2D<f32>,
}

impl<T, V: Vertex> Core<T, V>
where
    T: Clone + Eq,
{
    pub fn new(solid_texture: ImageData<T>, screen_size: Size2D<f32>) -> Core<T, V> {
        Core {
            draw_list: Vec::new(),

            mouse_pos: point2(0.0, 0.0),
            click_state: [
                ClickState::Unpressed,
                ClickState::Unpressed,
                ClickState::Unpressed,
            ],

            solid_texture,

            text_input: Vec::new(),

            tick: 0,

            clip: None,
            screen_size,
        }
    }

    pub fn push_raw_vertex(&mut self, vertex: V) -> u16 {
        let idx = self.draw_list.len() - 1;
        let batch = &mut self.draw_list[idx];
        let idx_offset = batch.vertices.len() as u16;
        batch.vertices.push(vertex);
        idx_offset
    }

    /// Push vertex into the draw batch, return its index offset.
    ///
    /// Index offsets are guaranteed to be consecutive and ascending as long as the current draw
    /// batch has not been switched, so you can grab the return value from the first `vertex_push`
    /// and express the rest by adding offsets to it.
    pub fn push_vertex(
        &mut self,
        pos: Point2D<f32>,
        tex_coord: Point2D<f32>,
        color: [f32; 4],
    ) -> u16 {
        self.push_raw_vertex(V::new(pos, tex_coord, color))
    }

    pub fn push_triangle(&mut self, i1: u16, i2: u16, i3: u16) {
        let idx = self.draw_list.len() - 1;
        let batch = &mut self.draw_list[idx];
        batch.triangle_indices.push(i1);
        batch.triangle_indices.push(i2);
        batch.triangle_indices.push(i3);
    }

    pub fn set_clip(&mut self, area: Rect<f32>) {
        self.clip = Some(area);
        self.check_batch(None);
    }

    pub fn clear_clip(&mut self) {
        self.clip = None;
        self.check_batch(None);
    }

    /// Return the current draw bounds
    pub fn bounds(&self) -> Rect<f32> {
        if let Some(clip) = self.clip {
            clip
        } else {
            self.screen_bounds()
        }
    }

    /// Return the screen bounds
    pub fn screen_bounds(&self) -> Rect<f32> { Rect::new(point2(0.0, 0.0), self.screen_size) }

    pub fn start_solid_texture(&mut self) {
        let t = self.solid_texture.texture.clone();
        self.start_texture(t);
    }

    pub fn solid_texture_texcoord(&self) -> Point2D<f32> { self.solid_texture.tex_coords.origin }

    pub fn start_texture(&mut self, texture: T) { self.check_batch(Some(texture)); }

    fn current_batch_is_invalid(&self, texture: T) -> bool {
        if self.draw_list.is_empty() {
            return true;
        }

        if self.draw_list[self.draw_list.len() - 1].texture != texture {
            return true;
        }

        if self.draw_list[self.draw_list.len() - 1].clip != self.clip {
            return true;
        }

        // Getting too close to u16 limit for comfort.
        if self.draw_list[self.draw_list.len() - 1].vertices.len() > (1 << 15) {
            return true;
        }

        false
    }

    /// Start a new render batch if needed.
    ///
    /// Need to start a new batch if render state has changed or if the current one is growing too
    /// large for the u16 indices.
    fn check_batch(&mut self, texture_needed: Option<T>) {
        if texture_needed.is_none() && self.draw_list.is_empty() {
            // Do nothing for stuff that only affects ongoing drawing.
            return;
        }

        let texture = texture_needed.unwrap_or_else(|| {
            self.draw_list[self.draw_list.len() - 1].texture.clone()
        });

        let clip = self.clip;

        if self.current_batch_is_invalid(texture.clone()) {
            self.draw_list.push(DrawBatch {
                texture,
                clip,
                vertices: Vec::new(),
                triangle_indices: Vec::new(),
            });
        }
    }

    pub fn draw_line(
        &mut self,
        thickness: f32,
        color: [f32; 4],
        p1: Point2D<f32>,
        p2: Point2D<f32>,
    ) {
        if p1 == p2 {
            return;
        }

        self.start_solid_texture();
        let t = self.solid_texture_texcoord();

        // Displacements from the one-dimensional base line.
        let mut front = p2 - p1;
        front = front / front.dot(front).sqrt() * (thickness / 2.0);

        let side = vec2(-front.y, front.x);

        let q1 = p1 - side - front + vec2(0.5, 0.5);
        let q2 = p1 + side - front + vec2(0.5, 0.5);
        let q3 = p2 + side + front + vec2(0.5, 0.5);
        let q4 = p2 - side + front + vec2(0.5, 0.5);

        let idx = self.push_vertex(q1, t, color);
        self.push_vertex(q2, t, color);
        self.push_vertex(q3, t, color);
        self.push_vertex(q4, t, color);
        self.push_triangle(idx, idx + 1, idx + 2);
        self.push_triangle(idx, idx + 2, idx + 3);
    }

    pub fn draw_tex_rect(&mut self, area: &Rect<f32>, tex_coords: &Rect<f32>, color: [f32; 4]) {
        let idx = self.push_vertex(area.origin, tex_coords.origin, color);
        self.push_vertex(area.top_right(), tex_coords.top_right(), color);
        self.push_vertex(area.bottom_right(), tex_coords.bottom_right(), color);
        self.push_vertex(area.bottom_left(), tex_coords.bottom_left(), color);

        self.push_triangle(idx, idx + 1, idx + 2);
        self.push_triangle(idx, idx + 2, idx + 3);
    }

    pub fn fill_rect(&mut self, area: &Rect<f32>, color: [f32; 4]) {
        self.start_solid_texture();
        let p = self.solid_texture_texcoord();
        self.draw_tex_rect(area, &rect(p.x, p.y, 0.0, 0.0), color);
    }

    pub fn draw_image(&mut self, image: &ImageData<T>, pos: Point2D<f32>, color: [f32; 4]) {
        self.start_texture(image.texture.clone());
        let size = Size2D::new(image.size.width as f32, image.size.height as f32);
        self.draw_tex_rect(&Rect::new(pos, size), &image.tex_coords, color);
    }

    /// Draw a line of text to screen.
    ///
    /// The `align` parameter indicates whether pos is interpreted as top left, top middle or top
    /// right position of the string.
    ///
    /// The return value is the position for the next line.
    pub fn draw_text(
        &mut self,
        font: &FontData<T>,
        pos: Point2D<f32>,
        align: Align,
        color: [f32; 4],
        text: &str,
    ) -> Point2D<f32> {
        let mut cursor_pos = pos;
        cursor_pos.x -= match align {
            Align::Left => 0.0,
            Align::Center => font.str_width(text) / 2.0,
            Align::Right => font.str_width(text),
        };

        for c in text.chars() {
            // XXX: Gratuitous cloning because of borrow checker.
            let x = font.chars.get(&c).cloned();
            // TODO: Draw some sort of symbol for characters missing from font.
            if let Some(f) = x {
                self.draw_image(&f.image, cursor_pos - f.draw_offset, color);
                cursor_pos.x += f.advance;
            }
        }

        point2(pos.x, pos.y + font.height)
    }

    /// Return the mouse input state for the current bounds area.
    pub fn click_state(&self, area: &Rect<f32>) -> ButtonAction {
        // XXX: This is doing somewhat sneaky stuff to avoid having to track widget IDs across
        // frames. In a usual GUI, you first need to press the mouse button on a widget, then
        // release it on top of the same widget for the click to register. Here, the click will
        // register on whichever widget the cursor is on when you release the button.
        //
        // If this behavior becomes a problem, then some sort of ID tracking system will need to be
        // added.

        let is_hovering = area.contains(&self.mouse_pos());

        let left_press = self.click_state[MouseButton::Left as usize].is_pressed() && is_hovering;

        let right_press = self.click_state[MouseButton::Right as usize].is_pressed() && is_hovering;

        let is_pressed = left_press || right_press;

        // Determine the return value.
        if left_press && self.click_state[MouseButton::Left as usize].is_release() {
            ButtonAction::LeftClicked
        } else if right_press && self.click_state[MouseButton::Right as usize].is_release() {
            ButtonAction::RightClicked
        } else if is_pressed {
            ButtonAction::Pressed
        } else if is_hovering {
            ButtonAction::Hover
        } else {
            ButtonAction::Inert
        }
    }

    pub fn begin_frame(&mut self) { self.tick += 1; }

    pub fn end_frame(&mut self) -> Vec<DrawBatch<T, V>> {
        // Clean up transient mouse click info.
        for i in 0..3 {
            self.click_state[i] = self.click_state[i].tick();
        }

        // Clean up text buffer
        self.text_input.clear();

        let mut ret = Vec::new();
        mem::swap(&mut ret, &mut self.draw_list);
        ret
    }

    /// Get the mouse cursor position in global space.
    pub fn mouse_pos(&self) -> Point2D<f32> { self.mouse_pos }

    /// Register mouse button state.
    pub fn input_mouse_button(&mut self, id: MouseButton, is_down: bool) {
        if is_down {
            self.click_state[id as usize] =
                self.click_state[id as usize].input_press(self.mouse_pos());
        } else {
            self.click_state[id as usize] =
                self.click_state[id as usize].input_release(self.mouse_pos());
        }
    }

    /// Register mouse motion.
    pub fn input_mouse_move(&mut self, x: i32, y: i32) {
        self.mouse_pos = point2(x as f32, y as f32);
    }

    /// Get whether mouse button was pressed
    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.click_state[button as usize].is_pressed()
    }

    /// Register printable character input.
    pub fn input_char(&mut self, c: char) { self.text_input.push(KeyInput::Printable(c)); }

    /// Register a nonprintable key state.
    pub fn input_key_state(&mut self, k: Keycode, is_down: bool) {
        if is_down {
            self.text_input.push(KeyInput::Other(k));
        }
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

/// Text alignment.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Align {
    Left,
    Center,
    Right,
}

/// Mouse button identifier.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// Mouse click state.
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
            ClickState::Unpressed |
            ClickState::Release(_, _) => ClickState::Unpressed,
            ClickState::Press(p) |
            ClickState::Drag(p) => ClickState::Drag(p),
        }
    }

    fn input_press(self, pos: Point2D<f32>) -> ClickState {
        match self {
            ClickState::Unpressed |
            ClickState::Release(_, _) => ClickState::Press(pos),
            ClickState::Press(p) |
            ClickState::Drag(p) => ClickState::Drag(p),
        }
    }

    fn input_release(self, pos: Point2D<f32>) -> ClickState {
        match self {
            ClickState::Unpressed => ClickState::Unpressed,
            ClickState::Press(p) |
            ClickState::Drag(p) |
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

/// Identifiers for nonprintable keys used in text editing widgets.
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

/// Font data for Vitral.
#[derive(Clone)]
pub struct FontData<T> {
    /// Map from chars to glyph images.
    pub chars: HashMap<char, CharData<T>>,
    /// Line height for this font.
    pub height: f32,
}

impl<T> FontData<T> {
    /// Return the size of a string of text in this font.
    pub fn render_size(&self, text: &str) -> Rect<f32> {
        let mut w = 0.0;

        for c in text.chars() {
            if let Some(f) = self.chars.get(&c) {
                w += f.advance;
            }
        }

        rect(0.0, 0.0, w, self.height)
    }

    /// Return the width of a char in the font.
    pub fn char_width(&self, c: char) -> Option<f32> { self.chars.get(&c).map(|c| c.advance) }

    pub fn str_width(&self, s: &str) -> f32 {
        s.chars().map(|c| self.char_width(c).unwrap_or(0.0)).sum()
    }
}

/// Drawable image data for Vitral.
#[derive(Clone, PartialEq)]
pub struct CharData<T> {
    pub image: ImageData<T>,
    pub draw_offset: Vector2D<f32>,
    pub advance: f32,
}

/// Action on a GUI button.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ButtonAction {
    Inert,
    Hover,
    Pressed,
    LeftClicked,
    RightClicked,
}

impl ButtonAction {
    pub fn left_clicked(&self) -> bool { self == &ButtonAction::LeftClicked }
    pub fn right_clicked(&self) -> bool { self == &ButtonAction::RightClicked }
}

#[cfg(feature = "image")]
impl<I, P> From<I> for ImageBuffer
where
    I: image::GenericImage<Pixel = P>,
    P: image::Pixel<Subpixel = u8>,
{
    fn from(image: I) -> ImageBuffer {
        let (w, h) = image.dimensions();
        let size = Size2D::new(w, h);

        let pixels = image
            .pixels()
            .map(|(_, _, p)| {
                let (r, g, b, a) = p.channels4();
                r as u32 + ((g as u32) << 8) + ((b as u32) << 16) + ((a as u32) << 24)
            })
            .collect();

        ImageBuffer { size, pixels }
    }
}

#[cfg(feature = "image")]
impl From<ImageBuffer> for image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    fn from(image: ImageBuffer) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        use image::Pixel;

        image::ImageBuffer::from_fn(image.size.width, image.size.height, |x, y| {
            let p = image.pixels[(x + y * image.size.width) as usize];
            image::Rgba::from_channels(p as u8, (p >> 8) as u8, (p >> 16) as u8, (p >> 24) as u8)
        })
    }
}

#[cfg(test)]
mod test {
    #[cfg(feature = "image")]
    #[test]
    fn image_roundtrip() {
        use euclid::size2;
        use image;
        use super::ImageBuffer;

        let image = ImageBuffer {
            pixels: vec![0xca11ab1e, 0x5ca1ab1e, 0xdeadbeef, 0xb01dface],
            size: size2(2, 2)
        };

        let image2: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image.clone().into();
        let image2: ImageBuffer = image2.into();

        assert!(image == image2);
    }
}
