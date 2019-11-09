use euclid::default::{Point2D, Rect, Size2D, Vector2D};
use euclid::{point2, rect, vec2};
use image::{RgbImage, RgbaImage};
use std::collections::HashMap;
use std::mem;

mod atlas;
mod atlas_cache;
pub use atlas_cache::ImageKey;
mod backend;
pub use backend::{App, AppConfig};
mod colors;
pub use colors::{color, scolor, to_linear, to_srgb, Rgba, SRgba, NAMED_COLORS};
mod flick;
pub use flick::{Flick, FLICKS_PER_SECOND};
mod keycode;
pub use keycode::Keycode;
mod rect_util;
pub use rect_util::RectUtil;
mod scene;
pub use scene::{InputEvent, Scene, SceneSwitch};
mod state;
pub use state::{
    add_sheet, add_tilesheet, add_tilesheet_font, get_frame_duration, get_image,
};

mod tilesheet;

/// Vitral representation for texture handle, consecutive positive integers.
pub(crate) type TextureIndex = usize;

/// Wrapper for the bytes of a PNG image file.
///
/// This is mostly intended for image data that is included in binaries using `include_bytes!`. It
/// implements an `Into` conversion to `RgbaImage` that will panic if the included bytes do not
/// resolve as an image file of the specified format.
///
/// This is a convenience type. If you are using data where you can't be sure it's a valid PNG,
/// call `image::load` explicitly to load it, check for errors and then convert the image to
/// `RgbaImage`.
pub struct PngBytes<'a>(pub &'a [u8]);

impl<'a> From<PngBytes<'a>> for RgbaImage {
    fn from(data: PngBytes<'_>) -> Self {
        use std::io::Cursor;

        let img = image::load(Cursor::new(data.0), image::ImageFormat::PNG)
            .expect("Failed to load PNG data");
        img.to_rgba()
    }
}

/// Drawable image data for Vitral.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ImageData {
    pub texture: TextureIndex,
    pub size: Size2D<u32>,
    pub tex_coords: Rect<f32>,
}

/// Vertex type for geometry points
#[derive(Copy, Clone)]
pub struct Vertex {
    /// 2D position
    pub pos: [f32; 2],
    /// Texture coordinates
    pub tex_coord: [f32; 2],
    /// Light pixel (foreground) color
    pub color: [f32; 4],
    /// Dark pixel (background) color
    pub back_color: [f32; 4],
}

impl Vertex {
    pub fn new(pos: Point2D<f32>, tex_coord: Point2D<f32>, color: Rgba) -> Self {
        Vertex {
            pos: [pos.x, pos.y],
            tex_coord: [tex_coord.x, tex_coord.y],
            color: color.into(),
            back_color: color::BLACK.into(),
        }
    }

    pub fn back_color(mut self, back_color: Rgba) -> Vertex {
        self.back_color = back_color.into();
        self
    }
}

/// An immediate mode graphical user interface context.
///
/// The context persists over a frame and receives commands that combine GUI
/// description and input handling. At the end of the frame, the commands are
/// converted into rendering instructions for the GUI.
pub struct Canvas<'a> {
    draw_list: Vec<DrawBatch>,

    screen_size: Size2D<i32>,
    ui: &'a mut UiState,
    backend: backend::Screenshotter<'a>,
}

impl<'a> Canvas<'a> {
    pub(crate) fn new(
        screen_size: Size2D<u32>,
        ui: &'a mut UiState,
        backend: backend::Screenshotter<'a>,
    ) -> Canvas<'a> {
        Canvas {
            draw_list: Vec::new(),
            screen_size: screen_size.to_i32(),
            ui,
            backend,
        }
    }

    pub fn push_raw_vertex(&mut self, vertex: Vertex) -> u16 {
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
    pub fn push_vertex(&mut self, pos: Point2D<i32>, tex_coord: Point2D<f32>, color: Rgba) -> u16 {
        self.push_raw_vertex(Vertex::new(pos.to_f32(), tex_coord, color))
    }

    pub fn push_triangle(&mut self, i1: u16, i2: u16, i3: u16) {
        let idx = self.draw_list.len() - 1;
        let batch = &mut self.draw_list[idx];
        batch.triangle_indices.push(i1);
        batch.triangle_indices.push(i2);
        batch.triangle_indices.push(i3);
    }

    pub fn set_clip(&mut self, area: Rect<i32>) {
        self.ui.clip = Some(area);
        self.check_batch(None);
    }

    pub fn clear_clip(&mut self) {
        self.ui.clip = None;
        self.check_batch(None);
    }

    /// Return the current draw bounds
    pub fn bounds(&self) -> Rect<i32> {
        if let Some(clip) = self.ui.clip {
            clip
        } else {
            self.screen_bounds()
        }
    }

    /// Return the screen bounds
    pub fn screen_bounds(&self) -> Rect<i32> { Rect::new(point2(0, 0), self.screen_size) }

    pub fn start_solid_texture(&mut self) {
        // XXX HACK assuming solid texture is at origin of texture 0
        self.start_texture(0);
    }

    pub fn solid_texture_texcoord(&self) -> Point2D<f32> {
        // XXX HACK assuming solid texture is at origin of texture 0
        point2(0.0, 0.0)
    }

    pub fn start_texture(&mut self, texture: TextureIndex) { self.check_batch(Some(texture)); }

    fn current_batch_is_invalid(&self, texture: TextureIndex) -> bool {
        if self.draw_list.is_empty() {
            return true;
        }

        if self.draw_list[self.draw_list.len() - 1].texture != texture {
            return true;
        }

        if self.draw_list[self.draw_list.len() - 1].clip != self.ui.clip {
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
    fn check_batch(&mut self, texture_needed: Option<TextureIndex>) {
        if texture_needed.is_none() && self.draw_list.is_empty() {
            // Do nothing for stuff that only affects ongoing drawing.
            return;
        }

        let texture =
            texture_needed.unwrap_or_else(|| self.draw_list[self.draw_list.len() - 1].texture);

        let clip = self.ui.clip;

        if self.current_batch_is_invalid(texture) {
            self.draw_list.push(DrawBatch {
                texture,
                clip,
                vertices: Vec::new(),
                triangle_indices: Vec::new(),
            });
        }
    }

    pub fn draw_line(&mut self, thickness: f32, color: Rgba, p1: Point2D<i32>, p2: Point2D<i32>) {
        if p1 == p2 {
            return;
        }

        let p1 = p1.to_f32();
        let p2 = p2.to_f32();

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

        let idx = self.push_vertex(q1.round().to_i32(), t, color);
        self.push_vertex(q2.round().to_i32(), t, color);
        self.push_vertex(q3.round().to_i32(), t, color);
        self.push_vertex(q4.round().to_i32(), t, color);
        self.push_triangle(idx, idx + 1, idx + 2);
        self.push_triangle(idx, idx + 2, idx + 3);
    }

    pub fn draw_tex_rect(&mut self, area: &Rect<i32>, tex_coords: &Rect<f32>, color: Rgba) {
        let idx = self.push_vertex(area.origin, tex_coords.origin, color);
        self.push_vertex(area.top_right(), tex_coords.top_right(), color);
        self.push_vertex(area.bottom_right(), tex_coords.bottom_right(), color);
        self.push_vertex(area.bottom_left(), tex_coords.bottom_left(), color);

        self.push_triangle(idx, idx + 1, idx + 2);
        self.push_triangle(idx, idx + 2, idx + 3);
    }

    pub fn fill_rect(&mut self, area: &Rect<i32>, color: Rgba) {
        self.start_solid_texture();
        let p = self.solid_texture_texcoord();
        self.draw_tex_rect(area, &rect(p.x, p.y, 0.0, 0.0), color);
    }

    pub fn draw_image(&mut self, image: &ImageData, pos: Point2D<i32>, color: Rgba) {
        self.start_texture(image.texture);
        self.draw_tex_rect(
            &Rect::new(pos, image.size.to_i32()),
            &image.tex_coords,
            color,
        );
    }

    /// Draw a line of text to screen.
    ///
    /// The `align` parameter indicates whether pos is interpreted as top left, top middle or top
    /// right position of the string.
    ///
    /// The return value is the position for the next line.
    pub fn draw_text(
        &mut self,
        font: &FontData,
        pos: Point2D<i32>,
        align: Align,
        color: Rgba,
        text: &str,
    ) -> Point2D<i32> {
        let mut cursor_pos = pos;
        cursor_pos.x -= match align {
            Align::Left => 0,
            Align::Center => font.str_width(text) / 2,
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
    pub fn click_state(&self, area: &Rect<i32>) -> ButtonAction {
        // XXX: This is doing somewhat sneaky stuff to avoid having to track widget IDs across
        // frames. In a usual GUI, you first need to press the mouse button on a widget, then
        // release it on top of the same widget for the click to register. Here, the click will
        // register on whichever widget the cursor is on when you release the button.
        //
        // If this behavior becomes a problem, then some sort of ID tracking system will need to be
        // added.

        let is_hovering = area.contains(self.mouse_pos());

        let left_press =
            self.ui.click_state[MouseButton::Left as usize].is_pressed() && is_hovering;

        let right_press =
            self.ui.click_state[MouseButton::Right as usize].is_pressed() && is_hovering;

        let middle_press =
            self.ui.click_state[MouseButton::Middle as usize].is_pressed() && is_hovering;

        let is_pressed = left_press || right_press;

        // Determine the return value.
        if left_press && self.ui.click_state[MouseButton::Left as usize].is_release() {
            ButtonAction::LeftClicked
        } else if right_press && self.ui.click_state[MouseButton::Right as usize].is_release() {
            ButtonAction::RightClicked
        } else if middle_press && self.ui.click_state[MouseButton::Middle as usize].is_release() {
            ButtonAction::MiddleClicked
        } else if is_pressed {
            ButtonAction::Pressed
        } else if is_hovering {
            ButtonAction::Hover
        } else {
            ButtonAction::Inert
        }
    }

    pub fn begin_frame(&mut self) {
        self.check_batch(None);
        self.ui.tick += 1;
    }

    pub fn end_frame(&mut self) -> Vec<DrawBatch> {
        // Clean up transient mouse click info.
        for i in 0..3 {
            self.ui.click_state[i] = self.ui.click_state[i].tick();
        }

        let mut ret = Vec::new();
        mem::swap(&mut ret, &mut self.draw_list);
        ret
    }

    /// Get the mouse cursor position in global space.
    pub fn mouse_pos(&self) -> Point2D<i32> { self.ui.mouse_pos }

    /// Get whether mouse button was pressed
    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.ui.click_state[button as usize].is_pressed()
    }

    pub fn draw_image_2color(
        &mut self,
        image: &ImageData,
        pos: Point2D<i32>,
        color: Rgba,
        back_color: Rgba,
    ) {
        self.start_texture(image.texture);

        let area = rect(
            pos.x,
            pos.y,
            image.size.width as i32,
            image.size.height as i32,
        );

        let idx = self.push_raw_vertex(
            Vertex::new(area.origin.to_f32(), image.tex_coords.origin, color)
                .back_color(back_color),
        );
        self.push_raw_vertex(
            Vertex::new(
                area.top_right().to_f32(),
                image.tex_coords.top_right(),
                color,
            )
            .back_color(back_color),
        );
        self.push_raw_vertex(
            Vertex::new(
                area.bottom_right().to_f32(),
                image.tex_coords.bottom_right(),
                color,
            )
            .back_color(back_color),
        );
        self.push_raw_vertex(
            Vertex::new(
                area.bottom_left().to_f32(),
                image.tex_coords.bottom_left(),
                color,
            )
            .back_color(back_color),
        );

        self.push_triangle(idx, idx + 1, idx + 2);
        self.push_triangle(idx, idx + 2, idx + 3);
    }

    /// Draw text with colored outline.
    pub fn draw_outline_text(
        &mut self,
        font: &FontData,
        pos: Point2D<i32>,
        align: Align,
        color: Rgba,
        back_color: Rgba,
        text: &str,
    ) -> Point2D<i32> {
        for offset in &[vec2(-1, 0), vec2(1, 0), vec2(0, -1), vec2(0, 1)] {
            self.draw_text(font, pos + *offset, align, back_color, text);
        }

        self.draw_text(font, pos, align, color, text)
    }

    /// Screenshot using async callback.
    pub fn screenshot_cb(&mut self, cb: impl FnOnce(image::RgbImage) + 'static) {
        self.backend.screenshot(cb)
    }

    /// Screenshot function that blocks until the screenshot is received.
    pub fn screenshot(&mut self) -> RgbImage {
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        self.screenshot_cb(move |img| sender.send(img).unwrap());
        receiver.recv().unwrap()
    }
}

pub struct UiState {
    mouse_pos: Point2D<i32>,
    click_state: [ClickState; 3],
    tick: u64,
    clip: Option<Rect<i32>>,
}

impl Default for UiState {
    fn default() -> Self {
        UiState {
            mouse_pos: point2(0, 0),
            click_state: [
                ClickState::Unpressed,
                ClickState::Unpressed,
                ClickState::Unpressed,
            ],
            tick: 0,
            clip: None,
        }
    }
}

impl UiState {
    /// Register mouse button state.
    pub(crate) fn input_mouse_button(&mut self, id: MouseButton, is_down: bool) {
        if is_down {
            self.click_state[id as usize] =
                self.click_state[id as usize].input_press(self.mouse_pos);
        } else {
            self.click_state[id as usize] =
                self.click_state[id as usize].input_release(self.mouse_pos);
        }
    }

    /// Register mouse motion.
    pub(crate) fn input_mouse_move(&mut self, x: i32, y: i32) { self.mouse_pos = point2(x, y); }
}

/// A sequence of primitive draw operarations.
pub struct DrawBatch {
    /// Texture used for the current batch, details depend on backend
    /// implementation
    pub texture: TextureIndex,
    /// Clipping rectangle for the current batch
    pub clip: Option<Rect<i32>>,
    /// Vertex data
    pub vertices: Vec<Vertex>,
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
    Press(Point2D<i32>),
    Drag(Point2D<i32>),
    Release(Point2D<i32>, Point2D<i32>),
}

impl ClickState {
    fn tick(self) -> ClickState {
        match self {
            ClickState::Unpressed | ClickState::Release(_, _) => ClickState::Unpressed,
            ClickState::Press(p) | ClickState::Drag(p) => ClickState::Drag(p),
        }
    }

    fn input_press(self, pos: Point2D<i32>) -> ClickState {
        match self {
            ClickState::Unpressed | ClickState::Release(_, _) => ClickState::Press(pos),
            ClickState::Press(p) | ClickState::Drag(p) => ClickState::Drag(p),
        }
    }

    fn input_release(self, pos: Point2D<i32>) -> ClickState {
        match self {
            ClickState::Unpressed => ClickState::Unpressed,
            ClickState::Press(p) | ClickState::Drag(p) | ClickState::Release(p, _) => {
                ClickState::Release(p, pos)
            }
        }
    }

    fn is_pressed(&self) -> bool {
        match *self {
            ClickState::Press(_) | ClickState::Drag(_) | ClickState::Release(_, _) => true,
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

/// Font data for Vitral.
#[derive(Clone)]
pub struct FontData {
    /// Map from chars to glyph images.
    pub chars: HashMap<char, CharData>,
    /// Line height for this font.
    pub height: i32,
}

impl FontData {
    /// Return the size of a string of text in this font.
    pub fn render_size(&self, text: &str) -> Rect<i32> {
        let mut w = 0;

        for c in text.chars() {
            if let Some(f) = self.chars.get(&c) {
                w += f.advance;
            }
        }

        rect(0, 0, w, self.height)
    }

    /// Return the width of a char in the font.
    pub fn char_width(&self, c: char) -> Option<i32> { self.chars.get(&c).map(|c| c.advance) }

    pub fn str_width(&self, s: &str) -> i32 {
        s.chars().map(|c| self.char_width(c).unwrap_or(0)).sum()
    }
}

/// Drawable image data for Vitral.
#[derive(Copy, Clone, PartialEq)]
pub struct CharData {
    pub image: ImageData,
    pub draw_offset: Vector2D<i32>,
    pub advance: i32,
}

/// Action on a GUI button.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ButtonAction {
    Inert,
    Hover,
    Pressed,
    LeftClicked,
    RightClicked,
    MiddleClicked,
}

impl ButtonAction {
    pub fn left_clicked(self) -> bool { self == ButtonAction::LeftClicked }
    pub fn right_clicked(self) -> bool { self == ButtonAction::RightClicked }
}

/// Normalized device coordinates for the top left corner of a pixel-perfect canvas in a window.
pub(crate) fn pixel_canvas_pos(window_size: Size2D<u32>, canvas_size: Size2D<u32>) -> Point2D<f32> {
    // Clip window dimensions to even numbers, pixel-perfect rendering has artifacts with odd
    // window dimensions.
    let window_size = Size2D::new(window_size.width & !1, window_size.height & !1);

    // Scale based on whichever of X or Y axis is the tighter fit.
    let mut scale = (window_size.width as f32 / canvas_size.width as f32)
        .min(window_size.height as f32 / canvas_size.height as f32);

    if scale > 1.0 {
        // Snap to pixel scale if more than 1 window pixel per canvas pixel.
        scale = scale.floor();
    }

    point2(
        -scale * canvas_size.width as f32 / window_size.width as f32,
        -scale * canvas_size.height as f32 / window_size.height as f32,
    )
}

pub(crate) fn window_to_canvas_coordinates(
    window_size: Size2D<u32>,
    canvas_size: Size2D<u32>,
    window_pos: Point2D<i32>,
) -> Point2D<i32> {
    // Clip odd dimensions again.
    let window_size = Size2D::new(window_size.width & !1, window_size.height & !1);

    let rp = pixel_canvas_pos(window_size, canvas_size);
    let rs = Size2D::new(rp.x.abs() * 2.0, rp.y.abs() * 2.0);

    // Transform to device coordinates.
    let sx = window_pos.x as f32 * 2.0 / window_size.width as f32 - 1.0;
    let sy = window_pos.y as f32 * 2.0 / window_size.height as f32 - 1.0;

    point2(
        ((sx - rp.x) * canvas_size.width as f32 / rs.width) as i32,
        ((sy - rp.y) * canvas_size.height as f32 / rs.height) as i32,
    )
}
