use time;
use std::mem;
use std::thread;
use std::convert::{Into};
use image::{GenericImage, SubImage, Pixel};
use image::{ImageBuffer, Rgba};
use image;
use glutin;
use glium::{self, DisplayBuild};
use ::{AtlasBuilder, Atlas, AtlasItem, V2, IterTiles, ImageStore};
use super::event::{Event, MouseButton};
use super::renderer::{Renderer};
use super::mesh;
use super::{WidgetId, CanvasMagnify};
use super::event_translator::{EventTranslator};

/// Width of the full font cell. Actual variable-width letters occupy some
/// portion of the left side of the cell.
pub static FONT_W: u32 = 8;
/// Height of the font.
pub static FONT_H: u32 = 8;

/// The first font image in the atlas image set.
static FONT_IDX: usize = 0;

/// Image index of the solid color texture block.
static SOLID_IDX: usize = 96;

/// Toplevel graphics drawing and input reading context.
pub struct CanvasBuilder {
    title: String,
    size: V2<u32>,
    frame_interval: Option<f64>,
    fullscreen: bool,
    layout_independent_keys: bool,
    magnify: CanvasMagnify,
    atlas_builder: AtlasBuilder,
}

impl CanvasBuilder {
    pub fn new() -> CanvasBuilder {
        let mut ret = CanvasBuilder {
            title: "".to_string(),
            size: V2(640, 360),
            frame_interval: None,
            fullscreen: false,
            layout_independent_keys: true,
            magnify: CanvasMagnify::PixelPerfect,
            atlas_builder: AtlasBuilder::new(),
        };
        ret.init_font();
        ret.init_solid();
        ret
    }

    /// Set the window title.
    pub fn set_title(mut self, title: &str) -> CanvasBuilder {
        self.title = title.to_string();
        self
    }

    /// Set the frame rate.
    pub fn set_frame_interval(mut self, interval_s: f64) -> CanvasBuilder {
        assert!(interval_s > 0.00001);
        self.frame_interval = Some(interval_s);
        self
    }

    /// Set the size of the logical canvas.
    pub fn set_size(mut self, width: u32, height: u32) -> CanvasBuilder {
        self.size = V2(width, height);
        self
    }

    /// Get the key values from the user's keyboard layout instead of the
    /// hardware keyboard map. Hardware keymap lookup may not work correctly
    /// on all platforms.
    pub fn use_layout_dependent_keys(mut self) -> CanvasBuilder {
        self.layout_independent_keys = false;
        self
    }

    /// Set the canvas to start in fullscreen mode.
    /// FIXME: Broken on Linux, https://github.com/tomaka/glutin/issues/148
    pub fn set_fullscreen(mut self, state: bool) -> CanvasBuilder {
        self.fullscreen = state;
        self
    }

    pub fn set_magnify(mut self, magnify: CanvasMagnify) -> CanvasBuilder {
        self.magnify = magnify;
        self
    }

    /// Build the canvas object.
    pub fn build(self) -> Canvas {
        Canvas::new(self)
    }

    /// Load the default font into the texture atlas.
    fn init_font(&mut self) {
        let mut font_sheet = ::color_key(
            &image::load_from_memory(include_bytes!("../assets/font.png")).unwrap(),
            0x808080FF);
        for tile in font_sheet.tiles(V2(8, 8)).take(96) {
            self.add_image(V2(0, -8),
                &SubImage::new(&mut font_sheet,
                               tile.mn().0, tile.mn().1, tile.dim().0, tile.dim().1));
        }
    }

    /// Load a solid color element into the texture atlas.
    fn init_solid(&mut self) {
        let image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(1, 1, |_, _| Rgba([0xffu8, 0xffu8, 0xffu8, 0xffu8]));
        let Image(idx) = self.add_image(V2(0, 0), &image);
        assert!(idx == SOLID_IDX);
    }
}

impl ImageStore<Image> for CanvasBuilder {
    /// Add an image into the canvas image atlas.
    fn add_image<P: Pixel<Subpixel=u8> + 'static, I: GenericImage<Pixel=P>>(
        &mut self, offset: V2<i32>, image: &I) -> Image {
        Image(self.atlas_builder.push(offset, image))
    }

}

/// Interface to render to a live display.
pub struct Canvas {
    display: glium::Display,
    renderer: Renderer,

    atlas: Atlas,

    state: State,
    frame_interval: Option<f64>,
    last_render_time: f64,
    size: V2<u32>,
    window_resolution: V2<i32>,

    meshes: Vec<Mesh>,

    /// Time in seconds it took to render the last frame.
    pub render_duration: f64,

    translator: EventTranslator,
    /// Imgui widget currently under mouse cursor.
    pub hot_widget: Option<WidgetId>,
    /// Imgui widget currently being interacted with.
    pub active_widget: Option<WidgetId>,
    /// Previous imgui widget.
    pub last_widget: Option<WidgetId>,
}

#[derive(PartialEq)]
enum State {
    Normal,
    EndFrame,
}

impl Canvas {
    fn new(builder: CanvasBuilder) -> Canvas {
        use glutin::{GlRequest, Api};
        let size = builder.size;
        let title = &builder.title[..];
        let frame_interval = builder.frame_interval;
        let atlas = Atlas::new(&builder.atlas_builder);

        let mut glutin = glutin::WindowBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 2)))
            .with_title(title.to_string());

        if builder.fullscreen {
            // FIXME: Glutin's X11 fullscreen is broken, this is only enabled
            // for Windows.
            if cfg!(windows) {
                glutin = glutin.with_fullscreen(glutin::get_primary_monitor());
            }
        } else {
            // Zoom up the window to the biggest even pixel multiple that fits
            // the user's monitor.
            let window_border_guesstimate = 32;
            let (w, h) = glutin::get_primary_monitor().get_dimensions();
            let window_size = V2(w, h) - V2(window_border_guesstimate, window_border_guesstimate);

            let (mut x, mut y) = (size.0, size.1);
            while x * 2 <= window_size.0 && y * 2 <= window_size.1 {
                x *= 2;
                y *= 2;
            }

            glutin = glutin.with_dimensions(x, y);
        }

        let display = glutin.build_glium().unwrap();

        let (w, h) = display.get_framebuffer_dimensions();

        let tex_image = image::imageops::flip_vertical(&atlas.image);
        let renderer = Renderer::new(size, &display, tex_image, builder.magnify);

        Canvas {
            display: display,
            renderer: renderer,

            atlas: atlas,

            state: State::Normal,
            frame_interval: frame_interval,
            last_render_time: time::precise_time_s(),
            size: size,
            window_resolution: V2(w as i32, h as i32),

            meshes: vec![Mesh::new()],

            render_duration: 0.1f64,

            translator: EventTranslator::new(builder.layout_independent_keys),
            hot_widget: None,
            active_widget: None,
            last_widget: None,
        }
    }

    /// Clear the screen.
    pub fn clear(&mut self) {
        // TODO: use the color.
        self.meshes = vec![Mesh::new()];
    }

    #[inline(always)]
    fn canvas_to_device(&self, pos: V2<f32>, z: f32) -> [f32; 3] {
        [-1.0 + (2.0 * (pos.0) / self.size.0 as f32),
          1.0 - (2.0 * (pos.1) / self.size.1 as f32),
         z]
    }

    /// Add a vertex to the geometry data of the current frame.
    pub fn push_vertex<C, C2>(&mut self, pos: V2<f32>, layer: f32, tex_coord: V2<f32>,
                              color: C, back_color: C2)
        where C: Into<::Rgba>,
              C2: Into<::Rgba>
    {
        assert!(self.meshes.len() > 0, "Empty mesh stack");
        let top = self.meshes.len() - 1;
        assert!(self.meshes[top].vertices.len() < 1<<16,
                "Too many accumulated vertices for index buffer, call flush() between meshes");
        let pos = self.canvas_to_device(pos, layer);

        self.meshes[top].vertices.push(mesh::Vertex {
            pos: pos,
            tex_coord: [tex_coord.0, tex_coord.1],
            color: color.into().into_array(),
            back_color: back_color.into().into_array(),
        });
    }

    /// Return the current vertex count, important for determining the indices
    /// for newly inserted vertices.
    pub fn num_vertices(&self) -> u16 {
        assert!(self.meshes.len() > 0, "Empty mesh stack");
        let top = self.meshes.len() - 1;
        self.meshes[top].vertices.len() as u16
    }


    /// Add a triangle defined by index values into the list of vertices
    /// inserted with push_vertex.
    pub fn push_triangle(&mut self, p0: u16, p1: u16, p2: u16) {
        assert!(self.meshes.len() > 0, "Empty mesh stack");
        let top = self.meshes.len() - 1;
        self.meshes[top].indices.push(p0);
        self.meshes[top].indices.push(p1);
        self.meshes[top].indices.push(p2);
    }

    /// Flush the input queue, can invalidate any cached vertex positions.
    ///
    /// Call this after you finish drawing meshes with `push_vertex` and
    /// `push_triangle`. If you push more that 2^16 vertices, the
    /// renderer index buffer will stop working correctly.
    pub fn flush(&mut self) {
        assert!(self.meshes.len() > 0, "Empty mesh stack");
        let top = self.meshes.len() - 1;
        // Some random threshold a bit below 2^16.
        if self.meshes[top].vertices.len() > 8192 {
            self.meshes.push(Mesh::new());
        }
    }

    /// Return the image corresponding to a char in the built-in font.
    pub fn font_image(&self, c: char) -> Option<Image> {
        let idx = c as usize;
        // Hardcoded limiting of the font to printable ASCII.
        if idx >= 32 && idx < 128 {
            Some(Image(idx - 32 + FONT_IDX))
        } else {
            None
        }
    }

    /// Return a texture coordinate to a #FFFFFFFF texel for solid color
    /// graphics.
    pub fn solid_tex_coord(&self) -> V2<f32> { self.atlas.items[SOLID_IDX].tex.0 }

    pub fn image_data<'b>(&'b self, Image(idx): Image) -> &'b AtlasItem {
        &self.atlas.items[idx]
    }

    /// Return a screenshot image of the last frame rendered.
    pub fn screenshot(&self) -> ImageBuffer<image::Rgb<u8>, Vec<u8>> {
        self.renderer.canvas_pixels()
    }

    fn imgui_prepare(&mut self) {
        // Initial setup for imgui.
        self.hot_widget = None;
    }

    fn imgui_finish(&mut self) {
        if self.translator.mouse_pressed[MouseButton::Left as usize].is_none() {
            self.active_widget = None;
        } else {
            // Setup a dummy widget so that dragging a mouse onto a widget
            // with the button held down won't activate that widget.
            if self.active_widget.is_none() {
                self.active_widget = Some(WidgetId::dummy());
            }
        }
    }

    pub fn mouse_pos(&self) -> V2<f32> { self.translator.mouse_pos }

    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.translator.mouse_pressed[button as usize].is_some()
    }

    pub fn next_event(&mut self) -> Event {
        // After a render event, control will return here on a new
        // iter call. Do post-render work here.
        if self.state == State::EndFrame {
            self.state = State::Normal;

            let mut target = self.display.draw();

            self.renderer.init(&self.display);
            let meshes = mem::replace(&mut self.meshes, vec![Mesh::new()]);
            for mesh in meshes.into_iter() {
                // Move out the accumulated geometry data.
                self.renderer.draw(&self.display, mesh.vertices, mesh.indices);
            }
            self.renderer.show(&self.display, &mut target);
            // TODO: Do something smarter than panic! on SwapBuffersError.
            target.finish().unwrap();

            self.imgui_finish();
        }

        let mut app_focused = true;
        loop {
            if let Some(e) = self.translator.next(&mut self.display, &self.renderer) {
                if let Event::FocusChange(b) = e {
                    app_focused = b;
                } else {
                    return e;
                }
            }

            let t = time::precise_time_s();
            if app_focused && self.frame_interval.map_or(true,
                |x| t - self.last_render_time >= x) {
                let delta = t - self.last_render_time;
                let sensitivity = 0.25f64;
                self.render_duration = (1f64 - sensitivity) * self.render_duration + sensitivity * delta;

                self.last_render_time = t;

                self.state = State::EndFrame;

                let (w, h) = self.display.get_framebuffer_dimensions();
                self.window_resolution = V2(w as i32, h as i32);

                self.imgui_prepare();

                // Return the render callback.
                return Event::RenderFrame;
            } else {
                // Go to sleep if there's time left.
                if let Some(mut remaining_s) = self.frame_interval {
                    remaining_s -= t - self.last_render_time;
                    if remaining_s > 0.0 {
                        thread::sleep_ms((remaining_s * 1e3) as u32);
                    }
                }
            }
        }
    }
}

/// Drawable images stored in the Canvas.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Image(usize);

struct Mesh {
    vertices: Vec<mesh::Vertex>,
    indices: Vec<u16>,
}

impl Mesh {
    fn new() -> Mesh {
        Mesh {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}
