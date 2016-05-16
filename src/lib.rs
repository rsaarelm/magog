extern crate euclid;

use std::mem;
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
        }
    }

    pub fn begin_frame(&mut self) {
        self.layout_pos = Point2D::new(10.0, 10.0);

        // TODO
    }

    pub fn text(&mut self, text: &str) {
        unimplemented!();
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

    pub fn fill_rect(&mut self, area: Rect<f32>, color: [f32; 4]) {
        let (tl, tr, bl, br) = (area.origin,
                                area.top_right(),
                                area.bottom_left(),
                                area.bottom_right());
        self.start_solid_texture();

        let idx = self.draw_list.len() - 1;
        let batch = &mut self.draw_list[idx];
        let idx_offset = batch.vertices.len() as u16;

        batch.vertices.push(Vertex {
            pos: [tl.x, tl.y],
            color: color,
            tex: [0.0, 0.0],
        });
        batch.vertices.push(Vertex {
            pos: [tr.x, tr.y],
            color: color,
            tex: [0.0, 0.0],
        });
        batch.vertices.push(Vertex {
            pos: [br.x, br.y],
            color: color,
            tex: [0.0, 0.0],
        });
        batch.vertices.push(Vertex {
            pos: [bl.x, bl.y],
            color: color,
            tex: [0.0, 0.0],
        });

        batch.triangle_indices.push(idx_offset);
        batch.triangle_indices.push(idx_offset + 1);
        batch.triangle_indices.push(idx_offset + 2);

        batch.triangle_indices.push(idx_offset);
        batch.triangle_indices.push(idx_offset + 2);
        batch.triangle_indices.push(idx_offset + 3);
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
    /// TODO: Font customization, point size, character ranges.
    pub fn init_font<F>(&mut self, ttf_data: &[u8], register_texture: F) -> Result<Font, ()>
        where F: FnOnce(&[u8], u32, u32) -> T
    {
        unimplemented!();
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
            ClickState::Release(_, _) => ClickState::Unpressed
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
            ClickState::Press(_) | ClickState::Drag(_) | ClickState::Release(_, _) => true,
            ClickState::Unpressed => false
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
pub struct Font(u64);
