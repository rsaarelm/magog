extern crate euclid;
extern crate time;

use euclid::{Point2D, point2, Rect, rect, Size2D, Vector2D, vec2};
use euclid::{TypedPoint2D, TypedRect, TypedSize2D};
use std::collections::HashMap;
use std::iter;
use std::mem;
use std::rc::Rc;

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
#[derive(Clone)]
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

/// Builder for Vitral `State` structure.
pub struct Builder<T> {
    user_font: Option<Rc<FontData<T>>>,
    user_solid: Option<ImageData<T>>,
}

impl<T> Builder<T>
where
    T: Clone + Eq,
{
    pub fn new() -> Builder<T> {
        Builder {
            user_font: None,
            user_solid: None,
        }
    }

    /// Set a different font as the default font.
    pub fn default_font(mut self, font: Rc<FontData<T>>) -> Builder<T> {
        self.user_font = Some(font);
        self
    }

    /// Give your own `ImageData` for the solid texture.
    ///
    /// You want to use this if you have an image atlas and you want to have both drawing solid
    /// shapes and textured shapes use the same texture resource and go to the same draw batch.
    pub fn solid_texture(mut self, solid: ImageData<T>) -> Builder<T> {
        self.user_solid = Some(solid);
        self
    }

    fn build_default_font<F>(&self, make_t: &mut F) -> FontData<T>
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

    /// Construct an interface context instance.
    ///
    /// Needs to be provided a texture creation function. If the user has not specified them
    /// earlier, this will be used to construct a separate texture for the solid color and a
    /// default font texture.
    pub fn build<F, V>(self, screen_size: Size2D<f32>, mut make_t: F) -> State<T, V>
    where
        F: FnMut(ImageBuffer) -> T,
    {
        let font;
        if let Some(user_font) = self.user_font {
            font = user_font
        } else {
            font = Rc::new(self.build_default_font(&mut make_t));
        }

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

        State::new(solid, screen_size, font)
    }
}

/// An immediate mode graphical user interface context.
///
/// The context persists over a frame and receives commands that combine GUI
/// description and input handling. At the end of the frame, the commands are
/// converted into rendering instructions for the GUI.
pub struct State<T, V> {
    draw_list: Vec<DrawBatch<T, V>>,

    mouse_pos: Point2D<f32>,
    click_state: [ClickState; 3],

    // Make this Rc so it can be passed outside without copying and used as a reference in a
    // mutable op.
    default_font: Rc<FontData<T>>,
    solid_texture: ImageData<T>,

    text_input: Vec<KeyInput>,

    tick: u64,

    clip_stack: Vec<Rect<f32>>,

    screen_size: Size2D<f32>,
}

impl<T, V> State<T, V>
where
    T: Clone + Eq,
{
    fn new(
        solid_texture: ImageData<T>,
        screen_size: Size2D<f32>,
        default_font: Rc<FontData<T>>,
    ) -> State<T, V> {
        State {
            draw_list: Vec::new(),

            mouse_pos: point2(0.0, 0.0),
            click_state: [
                ClickState::Unpressed,
                ClickState::Unpressed,
                ClickState::Unpressed,
            ],

            default_font,
            solid_texture,

            text_input: Vec::new(),

            tick: 0,

            clip_stack: Vec::new(),

            screen_size,
        }
    }

    /// Push vertex into the draw batch, return its index offset.
    ///
    /// Index offsets are guaranteed to be consecutive and ascending as long as the current draw
    /// batch has not been switched, so you can grab the return value from the first `vertex_push`
    /// and express the rest by adding offsets to it.
    pub fn push_vertex(&mut self, vtx: V) -> u16 {
        let idx = self.draw_list.len() - 1;
        let batch = &mut self.draw_list[idx];
        let idx_offset = batch.vertices.len() as u16;
        batch.vertices.push(vtx);
        idx_offset
    }

    pub fn push_triangle(&mut self, i1: u16, i2: u16, i3: u16) {
        let idx = self.draw_list.len() - 1;
        let batch = &mut self.draw_list[idx];
        batch.triangle_indices.push(i1);
        batch.triangle_indices.push(i2);
        batch.triangle_indices.push(i3);
    }

    /// Push a clipping rectangle into the clip stack.
    fn push_clip_rect(&mut self, area: Rect<f32>) {
        self.clip_stack.push(area);
        self.check_batch(None);
    }

    /// Pop the last clipping rectangle from the clip stack.
    ///
    /// The clip stack must have had at least one rectangle added with `push_clip_rect`.
    fn pop_clip_rect(&mut self) -> Rect<f32> {
        self.clip_stack.pop().expect("Popping an empty clip stack")
    }

    /// Return current clip rectangle, if any.
    fn clip_rect(&self) -> Option<Rect<f32>> {
        if self.clip_stack.is_empty() {
            None
        } else {
            Some(self.clip_stack[self.clip_stack.len() - 1])
        }
    }

    pub fn start_solid_texture(&mut self) {
        let t = self.solid_texture.texture.clone();
        self.start_texture(t);
    }

    fn solid_texture_texcoord(&self) -> Point2D<f32> { self.solid_texture.tex_coords.origin }

    pub fn start_texture(&mut self, texture: T) { self.check_batch(Some(texture)); }

    fn current_batch_is_invalid(&self, texture: T) -> bool {
        if self.draw_list.is_empty() {
            return true;
        }

        if self.draw_list[self.draw_list.len() - 1].texture != texture {
            return true;
        }

        if self.draw_list[self.draw_list.len() - 1].clip != self.clip_rect() {
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

        let clip = self.clip_rect();

        if self.current_batch_is_invalid(texture.clone()) {
            self.draw_list.push(DrawBatch {
                texture,
                clip,
                vertices: Vec::new(),
                triangle_indices: Vec::new(),
            });
        }
    }
}

/// Command interface for a Vitral GUI.
pub trait Context: Sized {
    type T: Clone + Eq;
    type V;

    /// Return the internal GUI state.
    ///
    /// This is mostly intended for other trait methods, not for direct use.
    fn state(&self) -> &State<Self::T, Self::V>;

    /// Return mutable reference to the internal GUI state.
    ///
    /// This is mostly intended for other trait methods, not for direct use.
    fn state_mut(&mut self) -> &mut State<Self::T, Self::V>;

    /// Construct a new vertex.
    ///
    /// Properties other than position, texture coordinate and color are provided by the
    /// implementation.
    fn new_vertex(
        &mut self,
        pos: Point2D<f32>,
        tex_coord: Point2D<f32>,
        color: [f32; 4],
    ) -> Self::V;

    /// Return reference to the currently active font.
    fn current_font(&self) -> Rc<FontData<Self::T>> { self.state().default_font.clone() }

    fn push_vertex<U: ConvertibleUnit>(
        &mut self,
        pos: TypedPoint2D<f32, U>,
        tex_coord: Point2D<f32>,
        color: [f32; 4],
    ) -> u16 {
        let pos = ConvertibleUnit::convert_point(&self.scale_factor(), pos);
        // NB: Transform is called on incoming vertices here, if any other place is pushing
        // vertices to the underlying state, make sure they go through `transform` as well.
        let pos = self.transform(pos);
        let v = self.new_vertex(pos, tex_coord, color);
        self.state_mut().push_vertex(v)
    }

    /// Transform point from the space of this context to global space.
    fn transform(&self, in_pos: Point2D<f32>) -> Point2D<f32> { in_pos }

    fn draw_line<U: ConvertibleUnit>(
        &mut self,
        thickness: f32,
        color: [f32; 4],
        p1: TypedPoint2D<f32, U>,
        p2: TypedPoint2D<f32, U>,
    ) {
        if p1 == p2 {
            return;
        }

        // Convert to screen space here because before applying thickness so that thickness will
        // always be in pixel units.
        let p1 = ConvertibleUnit::convert_point(&self.scale_factor(), p1);
        let p2 = ConvertibleUnit::convert_point(&self.scale_factor(), p2);

        self.state_mut().start_solid_texture();
        let t = self.state().solid_texture_texcoord();

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
        self.state_mut().push_triangle(idx, idx + 1, idx + 2);
        self.state_mut().push_triangle(idx, idx + 2, idx + 3);
    }

    fn draw_tex_rect<U: ConvertibleUnit>(
        &mut self,
        area: TypedRect<f32, U>,
        tex_coords: Rect<f32>,
        color: [f32; 4],
    ) {
        let idx = self.push_vertex(area.origin, tex_coords.origin, color);
        self.push_vertex(area.top_right(), tex_coords.top_right(), color);
        self.push_vertex(area.bottom_right(), tex_coords.bottom_right(), color);
        self.push_vertex(area.bottom_left(), tex_coords.bottom_left(), color);

        self.state_mut().push_triangle(idx, idx + 1, idx + 2);
        self.state_mut().push_triangle(idx, idx + 2, idx + 3);
    }

    fn fill_rect<U: ConvertibleUnit>(&mut self, area: TypedRect<f32, U>, color: [f32; 4]) {
        self.state_mut().start_solid_texture();
        let p = self.state().solid_texture_texcoord();
        self.draw_tex_rect(area, rect(p.x, p.y, 0.0, 0.0), color);
    }

    fn draw_image<U: ConvertibleUnit>(
        &mut self,
        image: &ImageData<Self::T>,
        pos: TypedPoint2D<f32, U>,
        color: [f32; 4],
    ) {
        let pos = ConvertibleUnit::convert_point(&self.scale_factor(), pos);

        self.state_mut().start_texture(image.texture.clone());
        let size = Size2D::new(image.size.width as f32, image.size.height as f32);
        self.draw_tex_rect(Rect::new(pos, size), image.tex_coords, color);
    }

    /// Draw a line of text to screen.
    ///
    /// The `align` parameter indicates whether pos is interpreted as top left, top middle or top
    /// right position of the string.
    ///
    /// The return value is the position for the next line.
    fn draw_text<U: ConvertibleUnit>(
        &mut self,
        pos: TypedPoint2D<f32, U>,
        align: Align,
        color: [f32; 4],
        text: &str,
    ) -> TypedPoint2D<f32, U> {
        // Convert to pixel space here, because font offsetting will operate in pixel space.
        let mut pixel_pos = ConvertibleUnit::convert_point(&self.scale_factor(), pos);

        pixel_pos.x -= match align {
            Align::Left => 0.0,
            Align::Center => self.current_font().str_width(text) / 2.0,
            Align::Right => self.current_font().str_width(text),
        };

        for c in text.chars() {
            // XXX: Gratuitous cloning because of borrow checker.
            let x = self.current_font().chars.get(&c).cloned();
            // TODO: Draw some sort of symbol for characters missing from font.
            if let Some(f) = x {
                self.draw_image(&f.image, pixel_pos - f.draw_offset, color);
                pixel_pos.x += f.advance;
            }
        }

        let (_, delta) = U::from_pixel_scale(&self.scale_factor(), 0.0, self.current_font().height);

        point2(pos.x, pos.y + delta)
    }

    /// Return the mouse input state for the current bounds area.
    fn click_state(&self) -> ButtonAction {
        let is_hovering = self.global_bounds().contains(&self.mouse_pos());

        let left_press = self.state().click_state[MouseButton::Left as usize].is_pressed() &&
            is_hovering;

        let right_press = self.state().click_state[MouseButton::Right as usize].is_pressed() &&
            is_hovering;

        let is_pressed = left_press || right_press;

        // Determine the return value.
        if left_press && self.state().click_state[MouseButton::Left as usize].is_release() {
            ButtonAction::LeftClicked
        } else if right_press &&
                   self.state().click_state[MouseButton::Right as usize].is_release()
        {
            ButtonAction::RightClicked
        } else if is_pressed {
            ButtonAction::Pressed
        } else if is_hovering {
            ButtonAction::Hover
        } else {
            ButtonAction::Inert
        }
    }

    /// Draw a button in the current bounds
    fn button(&mut self, caption: &str) -> ButtonAction {
        let ret = self.click_state();

        // Choose color.
        // TODO: Way to parametrize UI colors in style data.
        let color = match ret {
            ButtonAction::Pressed => [1.0, 1.0, 0.0, 1.0],
            ButtonAction::Hover => [0.5, 1.0, 0.0, 1.0],
            _ => [0.0, 1.0, 0.0, 1.0],
        };

        // Draw button in current bounds.
        let area = self.bounds();
        self.fill_rect(area, color);
        self.fill_rect(area.inflate(-1.0, -1.0), [0.0, 0.0, 0.0, 1.0]);

        // Vertically center the caption.
        let mut pos =
            ConvertibleUnit::convert_point(&self.scale_factor(), FracPoint2D::new(0.5, 0.0));
        pos.y = (self.bounds().size.height - self.current_font().height) / 2.0;
        self.draw_text(pos, Align::Center, color, caption);

        ret
    }

    fn begin_frame(&mut self) { self.state_mut().tick += 1; }

    fn end_frame(&mut self) -> Vec<DrawBatch<Self::T, Self::V>> {
        // Clean up transient mouse click info.
        for i in 0..3 {
            self.state_mut().click_state[i] = self.state().click_state[i].tick();
        }

        // Clean up text buffer
        self.state_mut().text_input.clear();

        let mut ret = Vec::new();
        mem::swap(&mut ret, &mut self.state_mut().draw_list);
        ret
    }

    /// Create a sub-context with geometry bound to a rectangle.
    fn bound_r<U: ConvertibleUnit>(&mut self, area: TypedRect<f32, U>) -> Bounds<Self> {
        let area = ConvertibleUnit::convert_rect(&self.scale_factor(), area);
        Bounds {
            parent: self,
            area: area,
            is_clipped: false,
        }
    }

    /// Create a sub-context with geometry bound to a rectangle and clip drawing to the bounds.
    fn bound_clipped_r<U: ConvertibleUnit>(&mut self, area: TypedRect<f32, U>) -> Bounds<Self> {
        let area = ConvertibleUnit::convert_rect(&self.scale_factor(), area);
        self.state_mut().push_clip_rect(area);
        Bounds {
            parent: self,
            area: area,
            is_clipped: true,
        }
    }

    /// Helper method for calling `bound_r` with pixel coordinates.
    fn bound(&mut self, x: u32, y: u32, w: u32, h: u32) -> Bounds<Self> {
        self.bound_r(rect::<f32, PixelUnit>(
            x as f32,
            y as f32,
            w as f32,
            h as f32,
        ))
    }

    /// Helper method for calling `bound_clipped_r` with pixel coordinates.
    fn bound_clipped(&mut self, x: u32, y: u32, w: u32, h: u32) -> Bounds<Self> {
        self.bound_clipped_r(rect::<f32, PixelUnit>(
            x as f32,
            y as f32,
            w as f32,
            h as f32,
        ))
    }

    /// Helper method for calling `bound_r` with fractional coordinates.
    fn bound_f(&mut self, x: f32, y: f32, w: f32, h: f32) -> Bounds<Self> {
        self.bound_r(TypedRect::<f32, FractionalUnit>::new(
            TypedPoint2D::new(x, y),
            TypedSize2D::new(w, h),
        ))
    }

    /// Helper method for calling `bound_clipped_r` with fractional coordinates.
    fn bound_clipped_f(&mut self, x: f32, y: f32, w: f32, h: f32) -> Bounds<Self> {
        self.bound_clipped_r(rect::<f32, FractionalUnit>(x, y, w, h))
    }

    /// Get the local space bounds rectangle of this context.
    fn bounds(&self) -> Rect<f32> { Rect::new(point2(0.0, 0.0), self.state().screen_size) }

    /// Get the global space bounds rectangle of this context.
    fn global_bounds(&self) -> Rect<f32> {
        let mut ret = self.bounds();
        ret.origin = self.transform(ret.origin);
        ret
    }

    fn scale_factor(&self) -> Size2D<f32> { self.bounds().size }

    /// Get the mouse cursor position in global space.
    fn mouse_pos(&self) -> Point2D<f32> { self.state().mouse_pos }

    /// Register mouse button state.
    fn input_mouse_button(&mut self, id: MouseButton, is_down: bool) {
        if is_down {
            self.state_mut().click_state[id as usize] = self.state().click_state[id as usize]
                .input_press(self.mouse_pos());
        } else {
            self.state_mut().click_state[id as usize] = self.state().click_state[id as usize]
                .input_release(self.mouse_pos());
        }
    }

    /// Register mouse motion.
    fn input_mouse_move(&mut self, x: i32, y: i32) {
        self.state_mut().mouse_pos = point2(x as f32, y as f32);
    }

    /// Get whether mouse button was pressed
    fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.state().click_state[button as usize].is_pressed()
    }

    /// Register printable character input.
    fn input_char(&mut self, c: char) { self.state_mut().text_input.push(KeyInput::Printable(c)); }

    /// Register a nonprintable key state.
    fn input_key_state(&mut self, k: Keycode, is_down: bool) {
        if is_down {
            self.state_mut().text_input.push(KeyInput::Other(k));
        }
    }

    fn text_input(&mut self, color: [f32; 4], text_buffer: &mut String) {
        // TODO: Focus system. Only accept input if current input widget is focused.
        // (Also needs widget identifiers to know which is which.)
        for c in &self.state().text_input {
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
            self.draw_text(
                point2::<f32, PixelUnit>(0.0, 0.0),
                Align::Left,
                color,
                text_buffer,
            );
        } else {
            self.draw_text(
                point2::<f32, PixelUnit>(0.0, 0.0),
                Align::Left,
                color,
                &format!("{}_", text_buffer),
            );
        }
    }
}

pub struct Bounds<'a, C: Context + 'a> {
    parent: &'a mut C,
    area: Rect<f32>,
    is_clipped: bool,
}

impl<'a, C: Context> Context for Bounds<'a, C> {
    type T = C::T;
    type V = C::V;

    fn state(&self) -> &State<Self::T, Self::V> { self.parent.state() }

    fn state_mut(&mut self) -> &mut State<Self::T, Self::V> { self.parent.state_mut() }

    fn new_vertex(
        &mut self,
        pos: Point2D<f32>,
        tex_coord: Point2D<f32>,
        color: [f32; 4],
    ) -> Self::V {
        self.parent.new_vertex(pos, tex_coord, color)
    }

    fn transform(&self, in_pos: Point2D<f32>) -> Point2D<f32> {
        self.parent.transform(in_pos + self.area.origin.to_vector())
    }

    fn bounds(&self) -> Rect<f32> { Rect::new(point2(0.0, 0.0), self.area.size) }
}

impl<'a, C: Context> Drop for Bounds<'a, C> {
    fn drop(&mut self) {
        // If this is a clipping bounds context, remove the clip when going out of scope.
        if self.is_clipped {
            self.state_mut().pop_clip_rect();
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

/// Unit type for `euclid` primitives for representing fractional units in [0.0, 1.0].
///
/// These units will be scaled to the area of the `Context` they're used in, (0.5, 0.5) will always
/// be in the center of the area.
pub struct FractionalUnit;

/// Explicit unit type for pixel units, treated the same as `euclid::UnknownUnit`.
pub struct PixelUnit;

pub trait ConvertibleUnit: Sized {
    fn scale_factor(scale: &Size2D<f32>) -> (f32, f32);

    fn to_pixel_scale(scale: &Size2D<f32>, x: f32, y: f32) -> (f32, f32) {
        let (w, h) = Self::scale_factor(scale);
        (x * w, y * h)
    }

    fn from_pixel_scale(scale: &Size2D<f32>, x: f32, y: f32) -> (f32, f32) {
        let (w, h) = Self::scale_factor(scale);
        (x / w, y / h)
    }

    fn convert_rect(scale: &Size2D<f32>, rect: TypedRect<f32, Self>) -> Rect<f32> {
        Rect::new(
            Self::convert_point(scale, rect.origin),
            Self::convert_size(scale, rect.size),
        )
    }

    fn convert_point(scale: &Size2D<f32>, point: TypedPoint2D<f32, Self>) -> Point2D<f32> {
        let (x, y) = Self::to_pixel_scale(scale, point.x, point.y);
        point2(x, y)
    }

    fn convert_size(scale: &Size2D<f32>, size: TypedSize2D<f32, Self>) -> Size2D<f32> {
        let (width, height) = Self::to_pixel_scale(scale, size.width, size.height);
        Size2D::new(width, height)
    }
}

impl ConvertibleUnit for euclid::UnknownUnit {
    fn scale_factor(_: &Size2D<f32>) -> (f32, f32) { (1.0, 1.0) }
}

impl ConvertibleUnit for PixelUnit {
    fn scale_factor(_: &Size2D<f32>) -> (f32, f32) { (1.0, 1.0) }
}

impl ConvertibleUnit for FractionalUnit {
    fn scale_factor(scale: &Size2D<f32>) -> (f32, f32) { (scale.width, scale.height) }
}

/// Alias for proportional unit point type.
pub type FracPoint2D = TypedPoint2D<f32, FractionalUnit>;

/// Alias for proportional unit size type.
pub type FracSize2D = TypedSize2D<f32, FractionalUnit>;

/// Alias for proportional unit rectangle type.
pub type FracRect = TypedRect<f32, FractionalUnit>;
