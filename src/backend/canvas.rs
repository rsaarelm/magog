use image::{GenericImage, SubImage, Pixel};
use image::{ImageBuffer, Rgba};
use image;
use glium::{Surface};
use ::{AtlasBuilder, Atlas, AtlasItem, V2, IterTiles, ImageStore, color};
use super::event::{Event, MouseButton};
use super::mesh;
use super::{WidgetId, RenderTarget};
use super::window::{Window};

/// Width of the full font cell. Actual variable-width letters occupy some
/// portion of the left side of the cell.
pub static FONT_W: u32 = 8;
/// Height of the font.
pub static FONT_H: u32 = 8;

/// The first font image in the atlas image set.
static FONT_IDX: usize = 0;

/// Image index of the solid color texture block.
static SOLID_IDX: usize = 96;

// XXX: CanvasBuilder mostly replicates WindowBuilder API after the
// refactoring of Canvas to use window below the hood. In the future, the
// atlas building API should just be done separately.

/// Toplevel graphics drawing and input reading context.
pub struct CanvasBuilder {
    atlas_builder: AtlasBuilder,
}

impl CanvasBuilder {
    pub fn new() -> CanvasBuilder {
        let mut ret = CanvasBuilder {
            atlas_builder: AtlasBuilder::new(),
        };
        ret.init_font();
        ret.init_solid();
        ret
    }

    /// Build the canvas object.
    pub fn build(self, window: Window) -> Canvas {
        Canvas::new(self, window)
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
    pub window: Window,
    pub clear_color: ::Rgba,
    atlas: Atlas,
    buffer: mesh::Buffer,

    /// Imgui widget currently under mouse cursor.
    pub hot_widget: Option<WidgetId>,
    /// Imgui widget currently being interacted with.
    pub active_widget: Option<WidgetId>,
    /// Previous imgui widget.
    pub last_widget: Option<WidgetId>,
}

impl Canvas {
    fn new(builder: CanvasBuilder, window: Window) -> Canvas {
        let atlas = Atlas::new(&builder.atlas_builder);

        let tex_image = image::imageops::flip_vertical(&atlas.image);
        let buffer = mesh::Buffer::new(&window.display, tex_image);
        Canvas {
            window: window,
            clear_color: color::BLACK,
            atlas: atlas,
            buffer: buffer,

            hot_widget: None,
            active_widget: None,
            last_widget: None,
        }
    }

    #[inline(always)]
    fn canvas_to_device(&self, pos: [f32; 3]) -> [f32; 3] {
        let size = self.window.size();
        [-1.0 + (2.0 * (pos[0]) / size.0 as f32),
          1.0 - (2.0 * (pos[1]) / size.1 as f32),
         pos[2]]
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

    pub fn end_frame(&mut self) {
        let display = self.window.display.clone();
        let buffer = &mut self.buffer;
        let clear_color = self.clear_color;
        self.window.draw(|target| {
            target.clear_color(clear_color.r, clear_color.g, clear_color.b, clear_color.a);
            target.clear_depth(1.0);
            buffer.flush(&display, target)
        });
        self.window.end_frame();
    }

    /// Return a screenshot image of the last frame rendered.
    pub fn screenshot(&self) -> ImageBuffer<image::Rgb<u8>, Vec<u8>> {
        self.window.get_screenshot()
    }

    /*
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
    */

    pub fn mouse_pos(&self) -> V2<f32> { self.window.mouse_pos() }

    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.window.mouse_pressed(button)
    }

    //pub fn events<'a>(&'a mut self) -> EventIterator<'a> { self.window.events() }
    pub fn events(&mut self) -> Vec<Event> { self.window.events() }
}

impl RenderTarget for Canvas {
    fn add_mesh(&mut self, mut vertices: Vec<mesh::Vertex>, faces: Vec<[u16; 3]>) {
        // Input is vertices in canvas pixel space, translate this into the
        // [-1.0, 1.0] device coordinate space.
        for v in vertices.iter_mut() {
            v.pos = self.canvas_to_device(v.pos);
        }
        self.buffer.add_mesh(vertices, faces);
    }
}

/// Drawable images stored in the Canvas.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Image(usize);
