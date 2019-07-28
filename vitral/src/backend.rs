//! Glium-based backend for the Vitral GUI library.

#![deny(missing_docs)]

use crate::atlas_cache::AtlasCache;
use crate::canvas_zoom::CanvasZoom;
use crate::{
    Canvas, ImageBuffer, InputEvent, Keycode, MouseButton, Scene, SceneSwitch, TextureIndex, Vertex,
};
use glium::glutin::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use glium::glutin::{self, Event, WindowEvent};
use glium::index::PrimitiveType;
use glium::{self, Surface};
use std::error::Error;
use std::fmt::Debug;
use std::hash::Hash;

type Point2D<T> = euclid::Point2D<T, euclid::UnknownUnit>;
type Size2D<T> = euclid::Size2D<T, euclid::UnknownUnit>;

/// Default texture type used by the backend.
type GliumTexture = glium::texture::SrgbTexture2d;

/// Glium-rendering backend for Vitral.
pub struct Backend {
    display: glium::Display,
    events: glutin::EventsLoop,
    program: glium::Program,
    textures: Vec<GliumTexture>,
    render_buffer: RenderBuffer,
    zoom: CanvasZoom,
    window_size: Size2D<u32>,
}

impl Backend {
    /// Create a new Glium backend for Vitral.
    ///
    /// The backend requires an user-supplied vertex type as a type parameter and a shader program
    /// to render data of that type as argument to the constructor.
    pub fn new(
        display: glium::Display,
        events: glutin::EventsLoop,
        program: glium::Program,
        width: u32,
        height: u32,
    ) -> Backend {
        let (w, h) = get_size(&display);
        let render_buffer = RenderBuffer::new(&display, width, height);

        Backend {
            display,
            events,
            program,
            textures: Vec::new(),
            render_buffer,
            zoom: CanvasZoom::PixelPerfect,
            window_size: Size2D::new(w, h),
        }
    }

    /// Open a Glium window and start a backend for it.
    ///
    /// The custom shader must support a uniform named `tex` for texture data.
    pub fn start<S: Into<String>>(
        width: u32,
        height: u32,
        title: S,
    ) -> Result<Backend, Box<dyn Error>> {
        let events = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new().with_title(title);
        let context = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)));
        let display = glium::Display::new(window, context, &events)?;
        let program = glium::Program::new(&display, DEFAULT_SHADER)?;

        {
            // Start the window as a good fit on the primary monitor.

            // Don't make it a completely fullscreen window, that might put the window title bar
            // outside the screen.
            const BUFFER: f64 = 8.0;
            let (width, height) = (width as f64, height as f64);

            let monitor_size = display
                .gl_window()
                .window()
                .get_primary_monitor()
                .get_dimensions();
            // Get the most conservative DPI if there's a weird multi-monitor setup.
            let dpi_factor = display
                .gl_window()
                .window()
                .get_available_monitors()
                .map(|m| m.get_hidpi_factor())
                .max_by(|x, y| x.partial_cmp(y).unwrap())
                .expect("No monitors found!");
            debug!("Scaling starting size to monitor");
            debug!("Monitor size {:?}", monitor_size);
            debug!("DPI Factor {}", dpi_factor);

            let mut window_size = PhysicalSize::new(width, height);
            while window_size.width + width <= monitor_size.width - BUFFER
                && window_size.height + height <= monitor_size.height - BUFFER
            {
                window_size.width += width;
                window_size.height += height;
            }
            debug!("Adjusted window size: {:?}", window_size);
            let window_pos = PhysicalPosition::new(
                (monitor_size.width - window_size.width) / 2.0,
                (monitor_size.height - window_size.height) / 2.0,
            );

            display
                .gl_window()
                .window()
                .set_inner_size(window_size.to_logical(dpi_factor));
            display
                .gl_window()
                .window()
                .set_position(window_pos.to_logical(dpi_factor));
        }

        Ok(Backend::new(display, events, program, width, height))
    }

    /// Return the pixel resolution of the backend.
    ///
    /// Note that this is the logical size which will stay the same even when the
    /// desktop window is resized.
    pub fn canvas_size(&self) -> Size2D<u32> { self.render_buffer.size }

    /// Return the current number of textures.
    pub fn texture_count(&self) -> usize { self.textures.len() }

    /// Make a new empty internal texture.
    ///
    /// The new `TextureIndex` must equal the value `self.texture_count()` would have returned
    /// just before calling this.
    pub fn make_empty_texture(&mut self, width: u32, height: u32) -> TextureIndex {
        let tex = glium::texture::SrgbTexture2d::empty(&self.display, width, height).unwrap();
        self.textures.push(tex);
        self.textures.len() - 1
    }

    /// Rewrite an internal texture.
    pub fn write_to_texture(&mut self, img: &ImageBuffer, texture: TextureIndex) {
        assert!(
            texture < self.textures.len(),
            "Trying to write nonexistent texture"
        );
        let rect = glium::Rect {
            left: 0,
            bottom: 0,
            width: img.size.width,
            height: img.size.height,
        };
        let mut raw = glium::texture::RawImage2d::from_raw_rgba(
            img.pixels.clone(),
            (img.size.width, img.size.height),
        );
        raw.format = glium::texture::ClientFormat::U8U8U8U8;

        self.textures[texture].write(rect, raw);
    }

    /// Make a new internal texture using image data.
    pub fn make_texture(&mut self, img: ImageBuffer) -> TextureIndex {
        let mut raw = glium::texture::RawImage2d::from_raw_rgba(
            img.pixels,
            (img.size.width, img.size.height),
        );
        raw.format = glium::texture::ClientFormat::U8U8U8U8;

        let tex = glium::texture::SrgbTexture2d::new(&self.display, raw).unwrap();
        self.textures.push(tex);
        self.textures.len() - 1
    }

    /// Update or construct textures based on changes in atlas cache.
    pub fn sync_with_atlas_cache<T: Eq + Hash + Clone + Debug>(
        &mut self,
        atlas_cache: &mut AtlasCache<T>,
    ) {
        for a in atlas_cache.atlases_mut() {
            let idx = a.texture();
            // If there are sheets in the atlas that don't have corresponding textures yet,
            // construct those now.
            while idx >= self.texture_count() {
                self.make_empty_texture(a.size().width, a.size().height);
            }

            // Write the updated texture atlas to internal texture.
            a.update_texture(|buf, idx| self.write_to_texture(buf, idx));
        }
    }

    fn dispatch<T>(
        &self,
        scene_stack: &mut Vec<Box<dyn Scene<T>>>,
        ctx: &mut T,
        event: InputEvent,
    ) -> Option<SceneSwitch<T>> {
        if !scene_stack.is_empty() {
            let idx = scene_stack.len() - 1;
            scene_stack[idx].input(ctx, event)
        } else {
            None
        }
    }

    fn process_events<T>(
        &mut self,
        canvas: &mut Canvas,
        scene_stack: &mut Vec<Box<dyn Scene<T>>>,
        ctx: &mut T,
    ) -> Result<Option<SceneSwitch<T>>, ()> {
        // polling and handling the events received by the window
        let mut event_list = Vec::new();
        self.events.poll_events(|event| event_list.push(event));
        // Accumulated scene switches from processing input
        let mut scene_switches = Vec::new();

        for e in event_list {
            match e {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.display.gl_window().window().id() => match event {
                    &WindowEvent::CloseRequested => return Err(()),
                    &WindowEvent::CursorMoved { position, .. } => {
                        let position = position
                            .to_physical(self.display.gl_window().window().get_hidpi_factor());
                        let pos = self.zoom.screen_to_canvas(
                            self.window_size,
                            self.render_buffer.size(),
                            Point2D::new(position.x as f32, position.y as f32),
                        );
                        canvas.input_mouse_move(pos.x as i32, pos.y as i32);
                    }
                    &WindowEvent::MouseInput { state, button, .. } => canvas.input_mouse_button(
                        match button {
                            glutin::MouseButton::Left => MouseButton::Left,
                            glutin::MouseButton::Right => MouseButton::Right,
                            _ => MouseButton::Middle,
                        },
                        state == glutin::ElementState::Pressed,
                    ),
                    &WindowEvent::ReceivedCharacter(c) => {
                        scene_switches.push(self.dispatch(scene_stack, ctx, InputEvent::Typed(c)));
                    }
                    &WindowEvent::KeyboardInput {
                        input:
                            glutin::KeyboardInput {
                                state,
                                scancode,
                                virtual_keycode,
                                ..
                            },
                        ..
                    } => {
                        let is_down = state == glutin::ElementState::Pressed;
                        let key = virtual_keycode.map_or(None, |virtual_keycode| {
                            Keycode::try_from(virtual_keycode).ok()
                        });
                        // Glutin adjusts the Linux scancodes, take into account. Don't know if
                        // this belongs here in the glium module or in the Keycode translation
                        // maps...
                        let scancode = if cfg!(target_os = "linux") {
                            scancode + 8
                        } else {
                            scancode
                        };
                        let hardware_key = Keycode::from_scancode(scancode);
                        if key.is_some() || hardware_key.is_some() {
                            scene_switches.push(self.dispatch(
                                scene_stack,
                                ctx,
                                InputEvent::KeyEvent {
                                    is_down,
                                    key,
                                    hardware_key,
                                },
                            ));
                        }
                    }
                    _ => (),
                },
                // Events in other windows, ignore
                Event::WindowEvent { .. } => {}
                Event::Awakened => {
                    // TODO: Suspend/awaken behavior
                }
                Event::DeviceEvent { .. } => {}
                Event::Suspended(_) => {}
            }
        }

        // Take the first scene switch that shows up.
        let scene_switch = scene_switches
            .into_iter()
            .fold(None, |prev, e| match (prev, e) {
                (Some(x), _) => Some(x),
                (None, y) => y,
            });

        Ok(scene_switch)
    }

    fn render(&mut self, canvas: &mut Canvas) {
        let mut target = self.render_buffer.get_framebuffer_target(&self.display);
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        let (w, h) = target.get_dimensions();

        for batch in canvas.end_frame() {
            // building the uniforms
            let uniforms = uniform! {
                matrix: [
                    [2.0 / w as f32, 0.0, 0.0, -1.0],
                    [0.0, -2.0 / h as f32, 0.0, 1.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0f32]
                ],
                tex: glium::uniforms::Sampler::new(&self.textures[batch.texture])
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
            };

            let vertex_buffer =
                { glium::VertexBuffer::new(&self.display, &batch.vertices).unwrap() };

            // building the index buffer
            let index_buffer = glium::IndexBuffer::new(
                &self.display,
                PrimitiveType::TrianglesList,
                &batch.triangle_indices,
            )
            .unwrap();

            let params = glium::draw_parameters::DrawParameters {
                scissor: batch.clip.map(|clip| glium::Rect {
                    left: clip.origin.x as u32,
                    bottom: h - (clip.origin.y + clip.size.height) as u32,
                    width: clip.size.width as u32,
                    height: clip.size.height as u32,
                }),
                blend: glium::Blend::alpha_blending(),
                ..Default::default()
            };

            target
                .draw(
                    &vertex_buffer,
                    &index_buffer,
                    &self.program,
                    &uniforms,
                    &params,
                )
                .unwrap();
        }
    }

    fn update_window_size(&mut self) {
        let (w, h) = get_size(&self.display);
        self.window_size = Size2D::new(w, h);
    }

    /// Display the backend and read input events.
    pub fn update<T>(
        &mut self,
        canvas: &mut Canvas,
        scene_stack: &mut Vec<Box<dyn Scene<T>>>,
        ctx: &mut T,
    ) -> Result<Option<SceneSwitch<T>>, ()> {
        self.update_window_size();
        self.render(canvas);
        self.render_buffer.draw(&self.display, self.zoom);
        self.process_events(canvas, scene_stack, ctx)
    }

    /// Return an image for the current contents of the screen.
    pub fn screenshot(&self) -> ImageBuffer { self.render_buffer.screenshot() }
}

/// Shader for two parametrizable colors and discarding fully transparent pixels
const DEFAULT_SHADER: glium::program::SourceCode<'_> = glium::program::SourceCode {
    vertex_shader: "
        #version 150 core

        uniform mat4 matrix;

        in vec2 pos;
        in vec2 tex_coord;
        in vec4 color;
        in vec4 back_color;

        out vec4 v_color;
        out vec4 v_back_color;
        out vec2 v_tex_coord;

        void main() {
            gl_Position = vec4(pos, 0.0, 1.0) * matrix;
            v_color = color;
            v_back_color = back_color;
            v_tex_coord = tex_coord;
        }
    ",

    fragment_shader: "
        #version 150 core
        uniform sampler2D tex;
        in vec4 v_color;
        in vec2 v_tex_coord;
        in vec4 v_back_color;
        out vec4 f_color;

        void main() {
            vec4 tex_color = texture(tex, v_tex_coord);

            // Discard fully transparent pixels to keep them from
            // writing into the depth buffer.
            if (tex_color.a == 0.0) discard;

            f_color = v_color * tex_color + v_back_color * (vec4(1, 1, 1, 1) - tex_color);
        }
    ",
    tessellation_control_shader: None,
    tessellation_evaluation_shader: None,
    geometry_shader: None,
};

implement_vertex!(Vertex, pos, tex_coord, color, back_color);

/// A deferred rendering buffer for pixel-perfect display.
struct RenderBuffer {
    size: Size2D<u32>,
    buffer: glium::texture::SrgbTexture2d,
    depth_buffer: glium::framebuffer::DepthRenderBuffer,
    shader: glium::Program,
}

impl RenderBuffer {
    pub fn new(display: &glium::Display, width: u32, height: u32) -> RenderBuffer {
        let shader = program!(
            display,
            150 => {
            vertex: "
                #version 150 core

                in vec2 pos;
                in vec2 tex_coord;

                out vec2 v_tex_coord;

                void main() {
                    v_tex_coord = tex_coord;
                    gl_Position = vec4(pos, 0.0, 1.0);
                }",
            fragment: "
                #version 150 core

                uniform sampler2D tex;
                in vec2 v_tex_coord;

                out vec4 f_color;

                void main() {
                    vec4 tex_color = texture(tex, v_tex_coord);
                    tex_color.a = 1.0;
                    f_color = tex_color;
                }"})
        .unwrap();

        let buffer = glium::texture::SrgbTexture2d::empty(display, width, height).unwrap();

        let depth_buffer = glium::framebuffer::DepthRenderBuffer::new(
            display,
            glium::texture::DepthFormat::F32,
            width,
            height,
        )
        .unwrap();

        RenderBuffer {
            size: Size2D::new(width, height),
            buffer,
            depth_buffer,
            shader,
        }
    }

    /// Get the render target to the pixel-perfect framebuffer.
    pub fn get_framebuffer_target(
        &mut self,
        display: &glium::Display,
    ) -> glium::framebuffer::SimpleFrameBuffer<'_> {
        glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
            display,
            &self.buffer,
            &self.depth_buffer,
        )
        .unwrap()
    }

    pub fn draw(&mut self, display: &glium::Display, zoom: CanvasZoom) {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);

        let (w, h) = get_size(display);

        // Build the geometry for the on-screen rectangle.
        let s_rect = zoom.fit_canvas(Size2D::new(w, h), self.size);

        let (sx, sy) = (s_rect.origin.x, s_rect.origin.y);
        let (sw, sh) = (s_rect.size.width, s_rect.size.height);

        // XXX: This could use glium::Surface::blit_whole_color_to instead of
        // the handmade blitting, but that was buggy on Windows around
        // 2015-03.

        let vertices = {
            #[derive(Copy, Clone)]
            struct BlitVertex {
                pos: [f32; 2],
                tex_coord: [f32; 2],
            }
            implement_vertex!(BlitVertex, pos, tex_coord);

            glium::VertexBuffer::new(
                display,
                &[
                    BlitVertex {
                        pos: [sx, sy],
                        tex_coord: [0.0, 0.0],
                    },
                    BlitVertex {
                        pos: [sx + sw, sy],
                        tex_coord: [1.0, 0.0],
                    },
                    BlitVertex {
                        pos: [sx + sw, sy + sh],
                        tex_coord: [1.0, 1.0],
                    },
                    BlitVertex {
                        pos: [sx, sy + sh],
                        tex_coord: [0.0, 1.0],
                    },
                ],
            )
            .unwrap()
        };

        let indices = glium::IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &[0u16, 1, 2, 0, 2, 3],
        )
        .unwrap();

        // Set up the rest of the draw parameters.
        let mut params: glium::DrawParameters<'_> = Default::default();
        // Set an explicit viewport to apply the custom resolution that fixes
        // pixel perfect rounding errors.
        params.viewport = Some(glium::Rect {
            left: 0,
            bottom: 0,
            width: w,
            height: h,
        });

        // TODO: Option to use smooth filter & non-pixel-perfect scaling
        let mag_filter = glium::uniforms::MagnifySamplerFilter::Nearest;

        let uniforms = glium::uniforms::UniformsStorage::new(
            "tex",
            glium::uniforms::Sampler(
                &self.buffer,
                glium::uniforms::SamplerBehavior {
                    magnify_filter: mag_filter,
                    minify_filter: glium::uniforms::MinifySamplerFilter::Linear,
                    ..Default::default()
                },
            ),
        );

        // Draw the graphics buffer to the window.
        target
            .draw(&vertices, &indices, &self.shader, &uniforms, &params)
            .unwrap();
        target.finish().unwrap();
    }

    pub fn size(&self) -> Size2D<u32> { self.size }

    pub fn screenshot(&self) -> ImageBuffer {
        let image: glium::texture::RawImage2d<'_, u8> = self.buffer.read();

        ImageBuffer::from_fn(image.width, image.height, |x, y| {
            let i = (x * 4 + (image.height - y - 1) * image.width * 4) as usize;
            image.data[i] as u32
                + ((image.data[i + 1] as u32) << 8)
                + ((image.data[i + 2] as u32) << 16)
                + ((image.data[i + 3] as u32) << 24)
        })
    }
}

fn get_size(display: &glium::Display) -> (u32, u32) {
    let size = display
        .gl_window()
        .window()
        .get_inner_size()
        .unwrap_or(LogicalSize::new(800.0, 600.0))
        .to_physical(display.gl_window().window().get_hidpi_factor());

    (size.width as u32, size.height as u32)
}
