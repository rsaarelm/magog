use crate::{
    flick::{Flick, FLICKS_PER_SECOND},
    keycode::Keycode,
    scene::{Scene, SceneStack},
    {Canvas, DrawBatch, InputEvent, MouseButton, UiState, Vertex},
};
use euclid::{
    default::{Point2D, Size2D},
    point2, size2,
};
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize},
    event::{ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct AppConfig {
    pub frame_duration: Flick,
    pub resolution: Size2D<u32>,
    pub window_title: String,
}

impl AppConfig {
    pub fn new(title: impl Into<String>) -> AppConfig {
        AppConfig {
            frame_duration: Flick(FLICKS_PER_SECOND / 30),
            resolution: size2(640, 360),
            window_title: title.into(),
        }
    }

    pub fn frame_duration(mut self, frame_duration: Flick) -> AppConfig {
        self.frame_duration = frame_duration;
        self
    }
}

pub struct App<T> {
    config: AppConfig,
    world: T,
    scenes: SceneStack<T>,
}

impl<T: 'static> App<T> {
    pub fn new(config: AppConfig, world: T, scenes: Vec<Box<dyn Scene<T>>>) -> App<T> {
        let frame_duration = config.frame_duration;
        App {
            config,
            world,
            scenes: SceneStack::new(frame_duration, scenes),
        }
    }

    pub fn run(mut self) -> ! {
        // Winit setup
        //
        let event_loop = EventLoop::new();
        let (_pos, size) = window_geometry(self.config.resolution, &event_loop);
        let window = WindowBuilder::new()
            .with_title(&self.config.window_title)
            .with_inner_size(size)
            .build(&event_loop)
            .unwrap();
        let hidpi_factor = window.hidpi_factor();
        let size = window.inner_size().to_physical(hidpi_factor);
        let (width, height) = (size.width.round() as u32, size.height.round() as u32);

        // WGPU setup
        //
        let instance = wgpu::Instance::new();
        let surface = instance.create_surface(
            raw_window_handle::HasRawWindowHandle::raw_window_handle(&window),
        );
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
        });

        let mut device = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        });
        let mut sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width,
            height,
            present_mode: wgpu::PresentMode::Vsync,
        };
        let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // WGPU textures that contain the atlased graphics from the application.
        let mut textures: Vec<Texture> = Vec::new();

        let gfx = Gfx::new(&device, self.config.resolution);

        // Main loop
        //
        let mut render_buffer = RenderBuffer::new(&mut device, self.config.resolution);
        render_buffer.update_canvas_pos(size2(sc_desc.width, sc_desc.height));

        let mut input_events = Vec::new();
        let mut ui = UiState::default();
        let mut running = true;
        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        let physical = size.to_physical(hidpi_factor);
                        log::info!("Resizing to {:?}", physical);
                        sc_desc.width = physical.width.round() as u32;
                        sc_desc.height = physical.height.round() as u32;
                        swap_chain = device.create_swap_chain(&surface, &sc_desc);
                        render_buffer.update_canvas_pos(size2(sc_desc.width, sc_desc.height));
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let pos = position.to_physical(hidpi_factor);
                        let pos = crate::window_to_canvas_coordinates(
                            size2(sc_desc.width, sc_desc.height),
                            self.config.resolution,
                            point2(pos.x as i32, pos.y as i32),
                        );
                        ui.input_mouse_move(pos.x, pos.y);
                    }
                    WindowEvent::MouseInput { state, button, .. } => ui.input_mouse_button(
                        match button {
                            winit::event::MouseButton::Left => MouseButton::Left,
                            winit::event::MouseButton::Right => MouseButton::Right,
                            _ => MouseButton::Middle,
                        },
                        state == ElementState::Pressed,
                    ),
                    WindowEvent::ReceivedCharacter(c) => {
                        input_events.push(InputEvent::Typed(c));
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                scancode,
                                virtual_keycode,
                                ..
                            },
                        ..
                    } => {
                        let is_down = state == ElementState::Pressed;
                        let key = virtual_keycode
                            .and_then(|virtual_keycode| Keycode::try_from(virtual_keycode).ok());
                        // Winit adjusts the Linux scancodes, take into account. Don't know if
                        // this belongs here in the glium module or in the Keycode translation
                        // maps...
                        let scancode = if cfg!(target_os = "linux") {
                            scancode + 8
                        } else {
                            scancode
                        };
                        let hardware_key = Keycode::from_scancode(scancode);
                        if key.is_some() || hardware_key.is_some() {
                            input_events.push(InputEvent::KeyEvent {
                                is_down,
                                key,
                                hardware_key,
                            });
                        }
                    }
                    WindowEvent::Focused(has_focus) => {
                        if has_focus {
                            log::info!("Focused");
                            running = true;
                            *control_flow = ControlFlow::Poll;
                            self.scenes.update_clock();
                        } else {
                            log::info!("Unfocused");
                            running = false;
                            *control_flow = ControlFlow::Wait;
                        }
                    }
                    _ => {}
                },
                Event::EventsCleared => {
                    if !running {
                        // Window is out of focus, don't waste CPU cycles running app.
                        return;
                    }
                    // Load atlas cache stuff to textures
                    //
                    crate::state::ENGINE_STATE
                        .lock()
                        .unwrap()
                        .atlas_cache
                        .update_system_textures(&mut TextureInterface(&mut device), &mut textures);

                    // Main update step
                    //
                    self.scenes.update(&mut self.world);

                    let draw_batch = {
                        let screenshotter = Screenshotter {
                            device: &mut device,
                            render_buffer: &render_buffer,
                        };
                        let mut canvas =
                            Canvas::new(self.config.resolution, &mut ui, screenshotter);
                        self.scenes.render(&mut self.world, &mut canvas);

                        for event in input_events.drain(0..) {
                            self.scenes.input(&mut self.world, &event, &mut canvas);
                        }

                        canvas.end_frame()
                    };

                    if self.scenes.is_empty() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    // Render graphics to buffer.
                    let sub_frame = render_buffer.render_buffer.texture.create_default_view();
                    let command_buffer = gfx.render(&device, &sub_frame, &textures, &draw_batch);
                    device.get_queue().submit(&[command_buffer]);

                    // Render buffer to window.
                    let frame = swap_chain.get_next_texture();
                    let command_buffer = render_buffer.render(&device, &frame.view);
                    device.get_queue().submit(&[command_buffer]);
                }
                _ => (),
            }
        })
    }
}

/// Grow the window so it'll fit the current monitor.
fn window_geometry<T>(
    resolution: Size2D<u32>,
    event_loop: &EventLoop<T>,
) -> (LogicalPosition, LogicalSize) {
    // Don't make it a completely fullscreen window, that might put the window title bar
    // outside the screen.
    const BUFFER: f64 = 8.0;
    let width = resolution.width as f64;
    let height = resolution.height as f64;

    let monitor_size = event_loop.primary_monitor().size();
    // Get the most conservative DPI if there's a weird multi-monitor setup.
    let dpi_factor = event_loop
        .available_monitors()
        .map(|m| m.hidpi_factor())
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .expect("No monitors found!");
    log::info!("Scaling starting size to monitor");
    log::info!("Monitor size {:?}", monitor_size);
    log::info!("DPI Factor {}", dpi_factor);

    let mut window_size = PhysicalSize::new(width, height);
    while window_size.width + width <= monitor_size.width - BUFFER
        && window_size.height + height <= monitor_size.height - BUFFER
    {
        window_size.width += width;
        window_size.height += height;
    }
    log::info!("Adjusted window size: {:?}", window_size);
    let window_pos = PhysicalPosition::new(
        (monitor_size.width - window_size.width) / 2.0,
        (monitor_size.height - window_size.height) / 2.0,
    );

    (
        window_pos.to_logical(dpi_factor),
        window_size.to_logical(dpi_factor),
    )
}

// Graphics drawing
//

/// Pipeline for drawing colorized 2D graphics.
struct Gfx {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    resolution: Size2D<u32>,
}

impl Gfx {
    pub fn new(device: &wgpu::Device, resolution: Size2D<u32>) -> Gfx {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 2,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler,
                },
            ],
        });

        // Rendering pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&bind_group_layout],
            }),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &device.create_shader_module(
                    &wgpu::read_spirv(std::io::Cursor::new(&include_bytes!("sprite.vert.spv")[..]))
                        .unwrap(),
                ),
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &device.create_shader_module(
                    &wgpu::read_spirv(std::io::Cursor::new(&include_bytes!("sprite.frag.spv")[..]))
                        .unwrap(),
                ),
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float2,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float2,
                        offset: 2 * 4,
                        shader_location: 1,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float4,
                        offset: 4 * 4,
                        shader_location: 2,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float4,
                        offset: 8 * 4,
                        shader_location: 3,
                    },
                ],
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Gfx {
            pipeline,
            bind_group_layout,
            resolution,
        }
    }

    pub fn draw(
        &self,
        device: &wgpu::Device,
        rpass: &mut wgpu::RenderPass,
        textures: &[Texture],
        batches: &[DrawBatch],
    ) {
        if batches.is_empty() {
            return;
        }

        // Transformation matrix for the batch, geometry coordinates are in pixels, this maps the
        // pixel buffer into device coordinates.
        type Uniforms = [f32; 16];
        let (w, h) = (self.resolution.width as f32, self.resolution.height as f32);
        #[rustfmt::skip]
        let matrix: Uniforms = [
            2.0/w,  0.0,  0.0, -1.0,
             0.0,  2.0/h, 0.0, -1.0,
             0.0,   0.0,  1.0,  0.0,
             0.0,   0.0,  0.0,  1.0,
        ];

        let uniform_buf = device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST)
            .fill_from_slice(&[matrix]);

        for batch in batches {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &uniform_buf,
                            range: 0..std::mem::size_of::<Uniforms>() as u64,
                        },
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: textures[batch.texture].texture_binding(),
                    },
                    wgpu::Binding {
                        binding: 2,
                        resource: textures[batch.texture].sampler_binding(),
                    },
                ],
            });

            let vertex_buf = device
                .create_buffer_mapped(batch.vertices.len(), wgpu::BufferUsage::VERTEX)
                .fill_from_slice(&batch.vertices);

            let index_buf = device
                .create_buffer_mapped(batch.triangle_indices.len(), wgpu::BufferUsage::INDEX)
                .fill_from_slice(&batch.triangle_indices);

            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.set_index_buffer(&index_buf, 0);
            rpass.set_vertex_buffers(0, &[(&vertex_buf, 0)]);
            if let Some(clip) = batch.clip {
                rpass.set_scissor_rect(
                    clip.origin.x as u32,
                    clip.origin.y as u32,
                    clip.size.width as u32,
                    clip.size.height as u32,
                );
            }
            rpass.draw_indexed(0..batch.triangle_indices.len() as u32, 0, 0..1);
        }
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        target: &wgpu::TextureView,

        textures: &[Texture],
        batches: &[DrawBatch],
    ) -> wgpu::CommandBuffer {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &target,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::BLACK,
                }],
                // Also need this for buffer renderer...
                depth_stencil_attachment: None,
            });
            self.draw(&device, &mut rpass, textures, batches);
        }
        encoder.finish()
    }
}

// Render buffer
//

/// Pipeline for drawing a pixel-perfect buffer on screen.
struct RenderBuffer {
    pipeline: wgpu::RenderPipeline,
    render_buffer: Texture,
    bind_group_layout: wgpu::BindGroupLayout,
    canvas_pos: Point2D<f32>,
    resolution: Size2D<u32>,
}

impl RenderBuffer {
    pub fn new(device: &mut wgpu::Device, resolution: Size2D<u32>) -> RenderBuffer {
        let canvas_pos = point2(-1.0, -1.0);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 2,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler,
                },
            ],
        });

        // Rendering pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&bind_group_layout],
            }),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &device.create_shader_module(
                    &wgpu::read_spirv(std::io::Cursor::new(&include_bytes!("blit.vert.spv")[..]))
                        .unwrap(),
                ),
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &device.create_shader_module(
                    &wgpu::read_spirv(std::io::Cursor::new(&include_bytes!("blit.frag.spv")[..]))
                        .unwrap(),
                ),
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let render_buffer = Texture::new_bgra(device, resolution.width, resolution.height);
        // TODO: Add depth buffer.

        RenderBuffer {
            pipeline,
            render_buffer,
            bind_group_layout,
            canvas_pos,
            resolution,
        }
    }

    pub fn update_canvas_pos(&mut self, window_size: Size2D<u32>) {
        self.canvas_pos = crate::pixel_canvas_pos(window_size, self.resolution);
    }

    pub fn draw(&self, device: &wgpu::Device, rpass: &mut wgpu::RenderPass) {
        type Uniforms = euclid::default::Point2D<f32>;

        let uniform_buf = device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST)
            .fill_from_slice(&[self.canvas_pos]);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buf,
                        range: 0..std::mem::size_of::<Uniforms>() as u64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: self.render_buffer.texture_binding(),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: self.render_buffer.sampler_binding(),
                },
            ],
        });

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.draw(0..4, 0..1);
    }

    pub fn render(&self, device: &wgpu::Device, target: &wgpu::TextureView) -> wgpu::CommandBuffer {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &target,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::BLACK,
                }],
                depth_stencil_attachment: None,
            });
            self.draw(&device, &mut rpass);
        }
        encoder.finish()
    }
}

pub(crate) struct Screenshotter<'a> {
    device: &'a mut wgpu::Device,
    render_buffer: &'a RenderBuffer,
}

impl<'a> Screenshotter<'a> {
    pub fn screenshot(&mut self, cb: impl FnOnce(image::RgbImage) + 'static) {
        let (width, height) = (
            self.render_buffer.resolution.width,
            self.render_buffer.resolution.height,
        );

        // Copy render buffer texture into a readable WGPU buffer.
        let output_buffer = {
            let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                size: (width * height) as u64 * 4,
                usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
            });

            let texture_extent = wgpu::Extent3d {
                width,
                height,
                depth: 1,
            };

            let command_buffer = {
                let mut encoder = self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
                // Copy the data from the texture to the buffer
                encoder.copy_texture_to_buffer(
                    wgpu::TextureCopyView {
                        texture: &self.render_buffer.render_buffer.texture,
                        mip_level: 0,
                        array_layer: 0,
                        origin: wgpu::Origin3d::ZERO,
                    },
                    wgpu::BufferCopyView {
                        buffer: &output_buffer,
                        offset: 0,
                        row_pitch: 4 * width,
                        image_height: height,
                    },
                    texture_extent,
                );

                encoder.finish()
            };

            self.device.get_queue().submit(&[command_buffer]);
            output_buffer
        };

        // Convert to image in callback.
        output_buffer.map_read_async(
            0,
            (width * height) as u64 * 4 as u64,
            move |result: wgpu::BufferMapAsyncResult<&[u8]>| {
                let bytes = result.unwrap().data;
                let image = image::RgbImage::from_fn(width, height, |x, y| {
                    let i = (x * 4 + y * width * 4) as usize;
                    image::Pixel::from_channels(bytes[i + 2], bytes[i + 1], bytes[i], 0xff)
                });
                cb(image)
            },
        );

        self.device.poll(true);
    }
}

// Texture wrapper
//

struct TextureInterface<'a>(&'a mut wgpu::Device);

impl<'a> crate::atlas_cache::TextureInterface for TextureInterface<'a> {
    type Texture = Texture;

    fn update_texture(&mut self, texture: &mut Self::Texture, image: &image::RgbaImage) {
        texture.blit(self.0, &image.as_flat_samples().samples);
    }

    fn new_texture(&mut self, size: Size2D<u32>) -> Self::Texture {
        Texture::new(self.0, size.width, size.height)
    }
}

pub struct Texture {
    texture: wgpu::Texture,
    sampler: wgpu::Sampler,
    extent: wgpu::Extent3d,
    view: wgpu::TextureView,
}

impl Texture {
    /// Fill texture with raw image data in the correct format
    pub fn blit(&self, device: &mut wgpu::Device, bytes: &[u8]) {
        assert_eq!(
            bytes.len() as u32,
            4 * self.extent.width * self.extent.height
        );

        let temp_buf = device
            .create_buffer_mapped(bytes.len(), wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&bytes);
        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        init_encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                row_pitch: 4 * self.extent.width,
                image_height: self.extent.height,
            },
            wgpu::TextureCopyView {
                texture: &self.texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            self.extent,
        );
        device.get_queue().submit(&[init_encoder.finish()]);
    }

    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Texture {
        Texture::new_typed(device, width, height, wgpu::TextureFormat::Rgba8UnormSrgb)
    }

    pub fn new_bgra(device: &wgpu::Device, width: u32, height: u32) -> Texture {
        Texture::new_typed(device, width, height, wgpu::TextureFormat::Bgra8UnormSrgb)
    }

    fn new_typed(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Texture {
        let extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        // Generate Texture object.
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        let view = texture.create_default_view();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always,
        });

        Texture {
            texture,
            view,
            sampler,
            extent,
        }
    }

    pub fn texture_binding(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::TextureView(&self.view)
    }

    pub fn sampler_binding(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::Sampler(&self.sampler)
    }
}
