extern crate euclid;
extern crate time;

use std::mem;
use std::collections::HashMap;
use std::rc::Rc;
use euclid::{Point2D, Rect, Size2D};


/// Image buffer.
pub struct ImageBuffer {
    /// Image width.
    pub width: u32,
    /// Image height.
    pub height: u32,
    /// RGBA pixels, in rows from top left down, len must be width * height.
    pub pixels: Vec<u32>,
}

impl ImageBuffer {
    pub fn blank() -> ImageBuffer {
        ImageBuffer {
            width: 1,
            height: 1,
            pixels: vec![0xffffffff],
        }
    }

    pub fn from_fn<F>(width: u32, height: u32, f: F) -> ImageBuffer
        where F: Fn(u32, u32) -> u32
    {
        let pixels = (0..)
                         .take((width * height) as usize)
                         .map(|i| f(i % width, i / width))
                         .collect();
        ImageBuffer {
            width: width,
            height: height,
            pixels: pixels,
        }
    }
}

pub struct Builder<T> {
    phantom: ::std::marker::PhantomData<T>,
}

impl<T> Builder<T>
    where T: Clone + Eq
{
    pub fn new() -> Builder<T> {
        Builder { phantom: ::std::marker::PhantomData }
    }

    fn build_default_font<F>(&self, make_t: &mut F) -> FontData<T>
        where F: FnMut(ImageBuffer) -> T
    {
        static DEFAULT_FONT: &'static [u8] = include_bytes!("unscii16-256x112.raw");
        let (width, height) = (256, 112);
        let start_char = 32;
        let end_char = 127;
        let (char_width, char_height) = (8, 16);
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

            let tex_coords = Rect::new(Point2D::new(x as f32 / width as f32,
                                                    y as f32 / height as f32),
                                       Size2D::new(char_width as f32 / width as f32,
                                                   char_height as f32 / height as f32));

            map.insert(std::char::from_u32(i).unwrap(),
                       CharData {
                           image: ImageData {
                               texture: t.clone(),
                               size: Size2D::new(char_width, char_height),
                               tex_coords: tex_coords,
                           },
                           draw_offset: Point2D::new(0.0, char_height as f32),
                           advance: char_width as f32,
                       });
        }

        FontData {
            chars: map,
            height: char_height as f32,
        }
    }

    /// Construct an interface context instance.
    pub fn build<F, V>(self, mut make_t: F) -> Context<T, V>
        where F: FnMut(ImageBuffer) -> T,
              V: Vertex
    {
        let default_font = self.build_default_font(&mut make_t);
        let solid = make_t(ImageBuffer::blank());

        Context::new(default_font, solid)
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
    click_state: [ClickState; 3],

    // Make this Rc so it can be passed outside without copying and used as a reference in a
    // mutable op.
    default_font: Rc<FontData<T>>,
    solid_texture: T,

    text_input: Vec<KeyInput>,

    tick: u64,

    clip: Option<Rect<f32>>,
}

impl<T, V: Vertex> Context<T, V>
    where T: Clone + Eq
{
    fn new(default_font: FontData<T>, solid_texture: T) -> Context<T, V> {
        Context {
            draw_list: Vec::new(),
            // solid_texture: Image(0),
            layout_pos: Point2D::new(0.0, 0.0),

            mouse_pos: Point2D::new(0.0, 0.0),
            click_state: [ClickState::Unpressed, ClickState::Unpressed, ClickState::Unpressed],

            default_font: Rc::new(default_font),
            solid_texture: solid_texture,

            text_input: Vec::new(),

            tick: 0,

            clip: None,
        }
    }

    pub fn begin_frame(&mut self) {
        self.tick += 1;

        self.layout_pos = Point2D::new(10.0, 10.0);

        // TODO
    }

    pub fn draw_text(&mut self,
                     font: &FontData<T>,
                     mut pos: Point2D<f32>,
                     color: [f32; 4],
                     text: &str) {
        for c in text.chars() {
            // FIXME: Gratuitous cloning because of borrow checker.
            let x = font.chars.get(&c).cloned();
            // TODO: Draw some sort of symbol for characters missing from font.
            if let Some(f) = x {
                self.draw_image(&f.image, pos - f.draw_offset, color);
                pos.x += f.advance;
            }
        }
    }

    pub fn default_font<'a>(&self) -> Rc<FontData<T>> {
        self.default_font.clone()
    }

    pub fn button(&mut self, caption: &str) -> bool {
        let font = self.default_font.clone();
        let area = font.render_size(caption)
                       .inflate(4.0, 4.0)
                       .translate(&self.layout_pos);

        self.layout_pos.y += area.size.height + 2.0;

        let hover = area.contains(&self.mouse_pos);
        let press = self.click_state[MouseButton::Left as usize].is_pressed() &&
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
        self.draw_text(&*font,
                       area.origin + Point2D::new(4.0, area.size.height - 4.0),
                       color,
                       caption);

        press && self.click_state[MouseButton::Left as usize].is_release()
    }

    pub fn draw_image(&mut self, image: &ImageData<T>, pos: Point2D<f32>, color: [f32; 4]) {
        self.start_texture(image.texture.clone());
        let size = Size2D::new(image.size.width as f32, image.size.height as f32);
        self.push_rect(Rect::new(pos, size), image.tex_coords, color);
    }

    fn push_rect(&mut self, area: Rect<f32>, tex_coords: Rect<f32>, color: [f32; 4]) {
        let (p1, p2) = (area.origin, area.bottom_right());
        let (t1, t2) = (tex_coords.origin, tex_coords.bottom_right());
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

    /// Set or clear clip rectangle.
    pub fn set_clip_rect(&mut self, area: Option<Rect<f32>>) {
        self.clip = area;
        self.check_batch(None);
    }

    /// Return current clip rectangle, if any.
    pub fn clip_rect(&self) -> Option<Rect<f32>> {
        self.clip
    }

    pub fn fill_rect(&mut self, area: Rect<f32>, color: [f32; 4]) {
        self.start_solid_texture();
        self.push_rect(area,
                       Rect::new(Point2D::new(0.0, 0.0), Size2D::new(0.0, 0.0)),
                       color);
    }

    pub fn draw_line(&mut self,
                     thickness: f32,
                     p1: Point2D<f32>,
                     p2: Point2D<f32>,
                     color: [f32; 4]) {
        if p1 == p2 {
            return;
        }

        self.start_solid_texture();
        let t = Point2D::new(0.0, 0.0);

        // Displacements from the one-dimensional base line.
        let mut front = p2 - p1;
        front = front / front.dot(front).sqrt() * (thickness / 2.0);

        let side = Point2D::new(-front.y, front.x);

        let q1 = p1 - side - front + Point2D::new(0.5, 0.5);
        let q2 = p1 + side - front + Point2D::new(0.5, 0.5);
        let q3 = p2 + side + front + Point2D::new(0.5, 0.5);
        let q4 = p2 - side + front + Point2D::new(0.5, 0.5);

        let idx = self.draw_list.len() - 1;
        let batch = &mut self.draw_list[idx];
        let idx_offset = batch.vertices.len() as u16;

        batch.vertices.push(Vertex::new([q1.x, q1.y], color, [t.x, t.y]));
        batch.vertices.push(Vertex::new([q2.x, q2.y], color, [t.x, t.y]));
        batch.vertices.push(Vertex::new([q3.x, q3.y], color, [t.x, t.y]));
        batch.vertices.push(Vertex::new([q4.x, q4.y], color, [t.x, t.y]));

        batch.triangle_indices.push(idx_offset);
        batch.triangle_indices.push(idx_offset + 1);
        batch.triangle_indices.push(idx_offset + 2);

        batch.triangle_indices.push(idx_offset);
        batch.triangle_indices.push(idx_offset + 2);
        batch.triangle_indices.push(idx_offset + 3);
    }

    pub fn text_input(&mut self,
                      font: &FontData<T>,
                      pos: Point2D<f32>,
                      color: [f32; 4],
                      text_buffer: &mut String) {
        // TODO: Focus system. Only accept input if current input widget is focused.
        // (Also needs widget identifiers to know which is which.)
        for c in &self.text_input {
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

        if ((time::precise_time_s() * 3.0) % 3.0) as u32 == 0 {
            self.draw_text(font, pos, color, text_buffer);
        } else {
            self.draw_text(font, pos, color, &format!("{}_", text_buffer));
        }
    }

    fn start_solid_texture(&mut self) {
        let t = self.solid_texture.clone();
        self.start_texture(t);
    }

    /// Ensure that there current draw batch has solid texture.
    fn start_texture(&mut self, texture: T) {
        self.check_batch(Some(texture));
    }

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

        if self.current_batch_is_invalid(texture.clone()) {
            self.draw_list.push(DrawBatch {
                texture: texture,
                clip: self.clip,
                vertices: Vec::new(),
                triangle_indices: Vec::new(),
            });
        }
    }

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

    /// Register mouse button state.
    pub fn input_mouse_button(&mut self, id: MouseButton, is_down: bool) {
        if is_down {
            self.click_state[id as usize] = self.click_state[id as usize]
                                                .input_press(self.mouse_pos);
        } else {
            self.click_state[id as usize] = self.click_state[id as usize]
                                                .input_release(self.mouse_pos);
        }
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

    /// Get the current mouse position
    pub fn mouse_pos(&self) -> Point2D<f32> {
        self.mouse_pos
    }

    /// Get whether mouse button was pressed
    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.click_state[button as usize].is_pressed()
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
    fn new(pos: [f32; 2], color: [f32; 4], tex_coord: [f32; 2]) -> Self;
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

        Rect::new(Point2D::new(0.0, 0.0), Size2D::new(w, self.height))
    }
}

/// Drawable image data for Vitral.
#[derive(Clone, PartialEq)]
pub struct CharData<T> {
    pub image: ImageData<T>,
    pub draw_offset: Point2D<f32>,
    pub advance: f32,
}

/// Drawable image data for Vitral.
#[derive(Clone, PartialEq)]
pub struct ImageData<T> {
    pub texture: T,
    pub size: Size2D<u32>,
    pub tex_coords: Rect<f32>,
}
