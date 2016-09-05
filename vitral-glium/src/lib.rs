//! Glium-based backend for the Vitral GUI library.

#![deny(missing_docs)]

#[macro_use]
extern crate glium;
extern crate euclid;
extern crate vitral;

mod canvas;
mod canvas_zoom;

use euclid::{Point2D, Size2D};
use glium::{Surface, glutin};
use glium::index::PrimitiveType;
use canvas::Canvas;
pub use canvas_zoom::CanvasZoom;

/// Default texture type used by the backend.
pub type GliumTexture = glium::texture::SrgbTexture2d;

/// Glium-rendering backend for Vitral.
pub struct Backend<V> {
    program: glium::Program,
    textures: Vec<GliumTexture>,

    keypress: Vec<KeyEvent>,

    canvas: Canvas,
    zoom: CanvasZoom,
    window_size: Size2D<u32>,

    phantom: ::std::marker::PhantomData<V>,
}

impl<V: vitral::Vertex + glium::Vertex> Backend<V> {
    /// Create a new Glium backend for Vitral.
    ///
    /// The backend requires an user-supplied vertex type as a type parameter and a shader program
    /// to render data of that type as argument to the constructor.
    pub fn new(display: &glium::Display, program: glium::Program, width: u32, height: u32) -> Backend<V> {
        let (w, h) = display.get_framebuffer_dimensions();

        Backend {
            program: program,
            textures: Vec::new(),

            keypress: Vec::new(),

            canvas: Canvas::new(display, width, height),
            zoom: CanvasZoom::PixelPerfect,
            window_size: Size2D::new(w, h),

            phantom: ::std::marker::PhantomData,
        }
    }

    /// Create a new texture using Vitral's input.
    pub fn make_texture(&mut self, display: &glium::Display, img: vitral::ImageBuffer) -> usize {
        let raw = glium::texture::RawImage2d::from_raw_rgba(img.pixels, (img.width, img.height));
        let tex = glium::texture::SrgbTexture2d::new(display, raw).unwrap();
        self.textures.push(tex);
        self.textures.len() - 1
    }

    fn process_events(&mut self,
                      display: &glium::Display,
                      context: &mut vitral::Context<usize, V>)
                      -> bool {
        self.keypress.clear();

        // polling and handling the events received by the window
        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return false,
                glutin::Event::MouseMoved(x, y) => {
                    let pos = self.zoom.screen_to_canvas(self.window_size,
                                                         self.canvas.size(),
                                                         Point2D::new(x as f32, y as f32));
                    context.input_mouse_move(pos.x as i32, pos.y as i32);
                }
                glutin::Event::MouseInput(state, button) => {
                    context.input_mouse_button(match button {
                                                   glutin::MouseButton::Left => {
                                                       vitral::MouseButton::Left
                                                   }
                                                   glutin::MouseButton::Right => {
                                                       vitral::MouseButton::Right
                                                   }
                                                   _ => vitral::MouseButton::Middle,
                                               },
                                               state == glutin::ElementState::Pressed)
                }
                glutin::Event::ReceivedCharacter(c) => context.input_char(c),
                glutin::Event::KeyboardInput(state, scancode, Some(vk)) => {
                    self.keypress.push(KeyEvent {
                        state: state,
                        key_code: vk,
                        scancode: scancode,
                    });

                    let is_down = state == glutin::ElementState::Pressed;

                    use glium::glutin::VirtualKeyCode::*;
                    if let Some(vk) = match vk {
                        Tab => Some(vitral::Keycode::Tab),
                        LShift | RShift => Some(vitral::Keycode::Shift),
                        LControl | RControl => Some(vitral::Keycode::Ctrl),
                        NumpadEnter | Return => Some(vitral::Keycode::Enter),
                        Back => Some(vitral::Keycode::Backspace),
                        Delete => Some(vitral::Keycode::Del),
                        Numpad8 | Up => Some(vitral::Keycode::Up),
                        Numpad2 | Down => Some(vitral::Keycode::Down),
                        Numpad4 | Left => Some(vitral::Keycode::Left),
                        Numpad6 | Right => Some(vitral::Keycode::Right),
                        _ => None,
                    } {
                        context.input_key_state(vk, is_down);
                    }
                }
                _ => (),
            }
        }

        true
    }

    /// Return the next keypress event if there is one.
    pub fn poll_key(&mut self) -> Option<KeyEvent> {
        self.keypress.pop()
    }

    fn render(&mut self, display: &glium::Display, context: &mut vitral::Context<usize, V>) {
        let mut target = self.canvas.get_framebuffer_target(display);
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        let (w, h) = target.get_dimensions();

        for batch in context.end_frame() {
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

            let vertex_buffer = {
                glium::VertexBuffer::new(display, &batch.vertices).unwrap()
            };

            // building the index buffer
            let index_buffer = glium::IndexBuffer::new(display,
                                                       PrimitiveType::TrianglesList,
                                                       &batch.triangle_indices)
                                   .unwrap();

            let params = glium::draw_parameters::DrawParameters {
                scissor: batch.clip.map(|clip| {
                    glium::Rect {
                        left: clip.origin.x as u32,
                        bottom: h - (clip.origin.y + clip.size.height) as u32,
                        width: clip.size.width as u32,
                        height: clip.size.height as u32,
                    }
                }),
                blend: glium::Blend::alpha_blending(),
                ..Default::default()
            };

            target.draw(&vertex_buffer,
                        &index_buffer,
                        &self.program,
                        &uniforms,
                        &params)
                  .unwrap();
        }
    }

    fn update_window_size(&mut self, display: &glium::Display) {
        let (w, h) = display.get_framebuffer_dimensions();
        self.window_size = Size2D::new(w, h);
    }

    /// Display the backend and read input events.
    pub fn update(&mut self,
                  display: &glium::Display,
                  context: &mut vitral::Context<usize, V>)
                  -> bool {
        self.update_window_size(display);
        self.render(display, context);
        self.canvas.draw(display, self.zoom);
        self.process_events(display, context)
    }
}


/// Type for key events not handled by Vitral.
pub struct KeyEvent {
    /// Was the key pressed or released
    pub state: glutin::ElementState,
    /// Layout-dependent keycode
    pub key_code: glutin::VirtualKeyCode,
    /// Keyboard layout independent hardware scancode for the key
    pub scancode: u8,
}

/// Create a shader program for the `DefaultVertex` type.
pub fn default_program(display: &glium::Display)
                       -> Result<glium::Program, glium::program::ProgramChooserCreationError> {
    program!(
        display,
        150 => {
        vertex: "
            #version 150 core

            uniform mat4 matrix;

            in vec2 pos;
            in vec4 color;
            in vec2 tex_coord;

            out vec4 v_color;
            out vec2 v_tex_coord;

            void main() {
                gl_Position = vec4(pos, 0.0, 1.0) * matrix;
                v_color = color;
                v_tex_coord = tex_coord;
            }
        ",

        fragment: "
            #version 150 core
            uniform sampler2D tex;
            in vec4 v_color;
            in vec2 v_tex_coord;
            out vec4 f_color;

            void main() {
                vec4 tex_color = texture(tex, v_tex_coord);

                // Discard fully transparent pixels to keep them from
                // writing into the depth buffer.
                if (tex_color.a == 0.0) discard;

                f_color = v_color * tex_color;
            }
        "})
}

/// A regular vertex that implements exactly the fields used by Vitral.
#[derive(Copy, Clone)]
pub struct DefaultVertex {
    /// 2D position
    pub pos: [f32; 2],
    /// RGBA color
    pub color: [f32; 4],
    /// Texture coordinates
    pub tex_coord: [f32; 2],
}
implement_vertex!(DefaultVertex, pos, color, tex_coord);

impl vitral::Vertex for DefaultVertex {
    fn new(pos: [f32; 2], color: [f32; 4], tex_coord: [f32; 2]) -> Self {
        DefaultVertex {
            pos: pos,
            color: color,
            tex_coord: tex_coord,
        }
    }
}
