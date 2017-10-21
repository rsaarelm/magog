use cache;
use canvas::Canvas;
use canvas_zoom::CanvasZoom;
use euclid::{Point2D, Rect, Size2D, TypedPoint2D};
use glium::{self, Surface};
use glium::glutin::{self, EventsLoop};
use glium::index::PrimitiveType;
use vitral::{self, Context};

pub type GliumTexture = glium::texture::SrgbTexture2d;

pub trait MagogContext: Context {
    fn draw_image_2color<U>(
        &mut self,
        image: &vitral::ImageData<usize>,
        pos: TypedPoint2D<f32, U>,
        color: [f32; 4],
        back_color: [f32; 4],
    ) where
        U: vitral::ConvertibleUnit;
}

pub struct Backend {
    events: EventsLoop,
    program: glium::Program,
    textures: Vec<GliumTexture>,

    keypress: Vec<KeyEvent>,

    canvas: Canvas,
    zoom: CanvasZoom,
    window_size: Size2D<u32>,

    ui_state: vitral::State<usize, Vertex>,
}

impl Backend {
    pub fn new(display: &glium::Display, events: EventsLoop, width: u32, height: u32) -> Backend {
        let program = program!(
            display,
            150 => {
            vertex: "
                #version 150 core

                uniform mat4 matrix;

                in vec2 pos;
                in vec4 color;
                in vec4 back_color;
                in vec2 tex_coord;

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

            fragment: "
                #version 150 core
                uniform sampler2D tex;
                in vec4 v_color;
                in vec4 v_back_color;
                in vec2 v_tex_coord;
                out vec4 f_color;

                void main() {
                    vec4 tex_color = texture(tex, v_tex_coord);

                    // Discard fully transparent pixels to keep them from
                    // writing into the depth buffer.
                    if (tex_color.a == 0.0) discard;

                    f_color = v_color * tex_color + v_back_color * (vec4(1, 1, 1, 1) - tex_color);
                }
            "}).unwrap();

        let (w, h) = display.get_framebuffer_dimensions();

        let mut textures = Vec::new();

        let ui_state = vitral::Builder::new()
            .default_font(cache::font())
            .solid_texture(cache::solid())
            .build(Size2D::new(width as f32, height as f32), |img| {
                Self::make_texture(&mut textures, display, img)
            });

        Backend {
            events,
            program,
            textures,

            keypress: Vec::new(),
            canvas: Canvas::new(display, width, height),
            zoom: CanvasZoom::PixelPerfect,
            window_size: Size2D::new(w, h),

            ui_state,
        }
    }

    fn make_empty_texture(&mut self, display: &glium::Display, width: u32, height: u32) -> usize {
        let tex = glium::texture::SrgbTexture2d::empty(display, width, height).unwrap();
        self.textures.push(tex);
        self.textures.len() - 1
    }

    fn write_to_texture(&mut self, img: &vitral::ImageBuffer, texture: usize) {
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

    fn make_texture(
        textures: &mut Vec<GliumTexture>,
        display: &glium::Display,
        img: vitral::ImageBuffer,
    ) -> usize {
        let mut raw = glium::texture::RawImage2d::from_raw_rgba(
            img.pixels,
            (img.size.width, img.size.height),
        );
        raw.format = glium::texture::ClientFormat::U8U8U8U8;

        let tex = GliumTexture::new(display, raw).unwrap();
        textures.push(tex);
        textures.len() - 1
    }

    fn process_events(&mut self) -> bool {
        self.keypress.clear();

        // polling and handling the events received by the window
        let mut event_list = Vec::new();
        self.events.poll_events(|event| event_list.push(event));

        for event in event_list {
            match event {
                glutin::Event::WindowEvent { event, .. } => {
                    match event {
                        glutin::WindowEvent::Closed => {
                            return false;
                        }
                        glutin::WindowEvent::MouseMoved { position: (x, y), .. } => {
                            let pos = self.zoom.screen_to_canvas(
                                self.window_size,
                                self.canvas.size(),
                                Point2D::new(x as f32, y as f32),
                            );
                            self.input_mouse_move(pos.x as i32, pos.y as i32);
                        }
                        glutin::WindowEvent::MouseInput { state, button, .. } => {
                            self.input_mouse_button(
                                match button {
                                    glutin::MouseButton::Left => vitral::MouseButton::Left,
                                    glutin::MouseButton::Right => vitral::MouseButton::Right,
                                    _ => vitral::MouseButton::Middle,
                                },
                                state == glutin::ElementState::Pressed,
                            )
                        }
                        glutin::WindowEvent::ReceivedCharacter(c) => self.input_char(c),
                        glutin::WindowEvent::KeyboardInput {
                            input: glutin::KeyboardInput {
                                state,
                                mut scancode,
                                virtual_keycode: Some(vk),
                                ..
                            },
                            ..
                        } => {
                            // XXX: winit has introduced a correction to scancodes, which makes it
                            // incompatible with my scancode interpreter. Need to de-correct here.
                            if cfg!(target_os = "linux") {
                                scancode += 8;
                            }

                            let is_down = state == glutin::ElementState::Pressed;

                            if is_down {
                                self.keypress.push(KeyEvent {
                                    key_code: vk,
                                    scancode: scancode as u8,
                                });
                            }

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
                            }
                            {
                                self.input_key_state(vk, is_down);
                            }
                        }
                        _ => (),
                    }
                }
                glutin::Event::Awakened => {
                    // TODO: Suspend/awaken behavior
                }
                glutin::Event::DeviceEvent { .. } => {}
                glutin::Event::Suspended(_) => {}
            }
        }

        true
    }

    pub fn poll_key(&mut self) -> Option<KeyEvent> { self.keypress.pop() }

    pub fn render(&mut self, display: &glium::Display) {
        let batches = self.end_frame();
        let mut target = self.canvas.get_framebuffer_target(display);
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        let (w, h) = target.get_dimensions();

        for batch in batches {
            // With the atlas cache, it's possible to get texture IDs for very recent images in the
            // render pipeline that don't have an actual GL texture associated with them yet.
            // Intercept them here and skip drawing them.
            if batch.texture >= self.textures.len() {
                continue;
            }

            // building the uniforms
            let uniforms =
                uniform! {
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
            let index_buffer = glium::IndexBuffer::new(
                display,
                PrimitiveType::TrianglesList,
                &batch.triangle_indices,
            ).unwrap();

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

    fn refresh_atlas(&mut self, display: &glium::Display) {
        cache::ATLAS.with(|a| for a in a.borrow_mut().atlases_mut() {
            let idx = *a.texture();
            assert!(idx <= self.textures.len());
            if idx == self.textures.len() {
                self.make_empty_texture(display, a.size().width, a.size().height);
            }

            a.update_texture(|buf, &idx| self.write_to_texture(buf, idx));
        });
    }

    fn update_window_size(&mut self, display: &glium::Display) {
        let (w, h) = display.get_framebuffer_dimensions();
        self.window_size = Size2D::new(w, h);
    }

    pub fn update(&mut self, display: &glium::Display) -> bool {
        self.refresh_atlas(display);
        self.update_window_size(display);
        self.render(display);
        self.canvas.draw(display, self.zoom);
        self.process_events()
    }

    pub fn save_screenshot(&mut self, basename: &str) {
        use time;
        use std::path::Path;
        use std::fs::{self, File};
        use image;

        let shot = self.canvas.screenshot();

        let timestamp = time::precise_time_s() as u64;
        // Create screenshot filenames by concatenating the current timestamp in
        // seconds with a running number from 00 to 99. 100 shots per second
        // should be good enough.

        // Default if we fail to generate any of the 100 candidates for this
        // second, just overwrite with the "xx" prefix then.
        let mut filename = format!("{}-{}{}.png", basename, timestamp, "xx");

        // Run through candidates for this second.
        for i in 0..100 {
            let test_filename = format!("{}-{}{:02}.png", basename, timestamp, i);
            // If file does not exist.
            if fs::metadata(&test_filename).is_err() {
                // Thread-safe claiming: create_dir will fail if the dir
                // already exists (it'll exist if another thread is gunning
                // for the same filename and managed to get past us here).
                // At least assuming that create_dir is atomic...
                let squat_dir = format!(".tmp-{}{:02}", timestamp, i);
                if fs::create_dir(&squat_dir).is_ok() {
                    File::create(&test_filename).unwrap();
                    filename = test_filename;
                    fs::remove_dir(&squat_dir).unwrap();
                    break;
                } else {
                    continue;
                }
            }
        }

        let _ = image::save_buffer(
            &Path::new(&filename),
            &shot,
            shot.width(),
            shot.height(),
            image::ColorType::RGB(8),
        );
    }
}

impl Context for Backend {
    type T = usize;
    type V = Vertex;

    fn state(&self) -> &vitral::State<usize, Vertex> { &self.ui_state }

    fn state_mut(&mut self) -> &mut vitral::State<usize, Vertex> { &mut self.ui_state }

    fn new_vertex(
        &mut self,
        pos: Point2D<f32>,
        tex_coord: Point2D<f32>,
        color: [f32; 4],
    ) -> Vertex {
        Vertex {
            pos: [pos.x, pos.y],
            color: color,
            back_color: [0.0, 0.0, 0.0, 1.0],
            tex_coord: [tex_coord.x, tex_coord.y],
        }
    }
}

impl<C: Context<T = usize, V = Vertex>> MagogContext for C {
    fn draw_image_2color<U>(
        &mut self,
        image: &vitral::ImageData<usize>,
        pos: TypedPoint2D<f32, U>,
        color: [f32; 4],
        back_color: [f32; 4],
    ) where
        U: vitral::ConvertibleUnit,
    {
        // XXX: Copy-pasting tex rect code from Vitral.
        let pos = vitral::ConvertibleUnit::convert_point(&self.scale_factor(), pos);

        self.state_mut().start_texture(image.texture);
        let size = Size2D::new(image.size.width as f32, image.size.height as f32);
        let area = Rect::new(pos, size);

        let idx = self.state_mut().push_vertex(Vertex::new(
            area.origin,
            image.tex_coords.origin,
            color,
            back_color,
        ));
        self.state_mut().push_vertex(Vertex::new(
            area.top_right(),
            image.tex_coords.top_right(),
            color,
            back_color,
        ));
        self.state_mut().push_vertex(Vertex::new(
            area.bottom_right(),
            image.tex_coords.bottom_right(),
            color,
            back_color,
        ));
        self.state_mut().push_vertex(Vertex::new(
            area.bottom_left(),
            image.tex_coords.bottom_left(),
            color,
            back_color,
        ));

        self.state_mut().push_triangle(idx, idx + 1, idx + 2);
        self.state_mut().push_triangle(idx, idx + 2, idx + 3);
    }
}

pub struct KeyEvent {
    pub key_code: glutin::VirtualKeyCode,
    pub scancode: u8,
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 4],
    pub back_color: [f32; 4],
    pub tex_coord: [f32; 2],
}
implement_vertex!(Vertex, pos, color, back_color, tex_coord);

impl Vertex {
    pub fn new(
        pos: Point2D<f32>,
        tex_coord: Point2D<f32>,
        color: [f32; 4],
        back_color: [f32; 4],
    ) -> Vertex {
        Vertex {
            pos: [pos.x, pos.y],
            color: color,
            back_color: back_color,
            tex_coord: [tex_coord.x, tex_coord.y],
        }
    }
}
