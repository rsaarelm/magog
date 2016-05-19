extern crate euclid;

use std::mem;
use std::ops::Range;
use std::collections::HashMap;
use euclid::{Rect, Point2D, Size2D};

/// An immediate mode graphical user interface context.
///
/// The context persists over a frame and receives commands that combine GUI
/// description and input handling. At the end of the frame, the commands are
/// converted into rendering instructions for the GUI.
pub struct Context<T> {
    draw_list: Vec<DrawBatch<T>>,
    /// Texture value used for solid-color drawing
    solid_texture: T,

    // TODO: This is for demo purposes, need a proper layout state.
    layout_pos: Point2D<f32>,

    mouse_pos: Point2D<f32>,
    click_state: ClickState,

    fonts: Vec<FontData<T>>,
}

impl<T> Context<T>
    where T: Clone + PartialEq
{
    /// Construct a new interface context instance.
    ///
    /// The client must provide an initial texture instance that is used for
    /// drawing solid shapes. This can be a texture that consists of a single
    /// opaque white pixel.
    pub fn new(solid_texture: T) -> Context<T> {
        Context {
            draw_list: Vec::new(),
            solid_texture: solid_texture,

            layout_pos: Point2D::new(0.0, 0.0),

            mouse_pos: Point2D::new(0.0, 0.0),
            click_state: ClickState::Unpressed,

            fonts: Vec::new(),
        }
    }

    pub fn begin_frame(&mut self) {
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
            if let Some(f) = x {
                self.tex_rect(Rect::new(pos - f.draw_offset, Size2D::new(f.advance, h)),
                              f.texcoords,
                              color);
                pos.x += f.advance;
            }
        }
    }

    pub fn button(&mut self, caption: &str) -> bool {
        let area = Rect::new(self.layout_pos, Size2D::new(64.0, 16.0));
        self.layout_pos.y += 20.0;

        let hover = area.contains(&self.mouse_pos);
        let press = self.click_state.is_pressed() && area.contains(&self.mouse_pos);

        self.fill_rect(area, [0.0, 0.0, 0.0, 1.0]);
        self.fill_rect(area.inflate(-1.0, -1.0),
                       if press {
                           [1.0, 1.0, 0.0, 1.0]
                       } else if hover {
                           [1.0, 0.5, 0.0, 1.0]
                       } else {
                           [1.0, 0.0, 0.0, 1.0]
                       });

        press && self.click_state.is_release()
    }

    pub fn tex_rect(&mut self, area: Rect<f32>, texcoords: Rect<f32>, color: [f32; 4]) {
        let (p1, p2) = (area.origin, area.bottom_right());
        let (t1, t2) = (texcoords.origin, texcoords.bottom_right());
        let idx = self.draw_list.len() - 1;
        let batch = &mut self.draw_list[idx];
        let idx_offset = batch.vertices.len() as u16;

        batch.vertices.push(Vertex {
            pos: [p1.x, p1.y],
            color: color,
            tex: [t1.x, t1.y],
        });
        batch.vertices.push(Vertex {
            pos: [p2.x, p1.y],
            color: color,
            tex: [t2.x, t1.y],
        });
        batch.vertices.push(Vertex {
            pos: [p2.x, p2.y],
            color: color,
            tex: [t2.x, t2.y],
        });
        batch.vertices.push(Vertex {
            pos: [p1.x, p2.y],
            color: color,
            tex: [t1.x, t2.y],
        });

        batch.triangle_indices.push(idx_offset);
        batch.triangle_indices.push(idx_offset + 1);
        batch.triangle_indices.push(idx_offset + 2);

        batch.triangle_indices.push(idx_offset);
        batch.triangle_indices.push(idx_offset + 2);
        batch.triangle_indices.push(idx_offset + 3);
    }

    pub fn fill_rect(&mut self, area: Rect<f32>, color: [f32; 4]) {
        self.start_solid_texture();
        self.tex_rect(area,
                      Rect::new(Point2D::new(0.0, 0.0), Size2D::new(0.0, 0.0)),
                      color);
    }

    fn start_solid_texture(&mut self) {
        let tex = self.solid_texture.clone();
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

    pub fn end_frame(&mut self) -> Vec<DrawBatch<T>> {
        // Clean up transient mouse click info.
        self.click_state = self.click_state.tick();

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
        // TODO
    }

    /// Register a nonprintable key state.
    pub fn input_key_state(&mut self, k: Keycode, is_down: bool) {
        // TODO
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
        static DEFAULT_FONT: &'static [u8] = include_bytes!("profont9.raw");
        let (width, height) = (96, 72);
        let columns = 16;
        let start_char = 32;
        let end_char = 127;
        let (char_width, char_height) = (6, 12);

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
pub struct DrawBatch<T> {
    /// Texture used for the current batch, details depend on backend
    /// implementation
    pub texture: T,
    /// Clipping rectangle for the current batch
    pub clip: Option<Rect<f32>>,
    /// Vertex data
    pub vertices: Vec<Vertex>,
    /// Indices into the vertex array for the triangles that make up the batch
    pub triangle_indices: Vec<u16>,
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 4],
    pub tex: [f32; 2],
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Font(usize);

pub struct FontData<T> {
    texture: T,
    chars: HashMap<char, CharData>,
    height: f32,
}

#[derive(Clone, Debug)]
struct CharData {
    texcoords: Rect<f32>,
    draw_offset: Point2D<f32>,
    advance: f32,
}
