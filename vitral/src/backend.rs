use crate::{
    flick::{Flick, FLICKS_PER_SECOND},
    keycode::Keycode,
    scene::{Scene, SceneStack},
    {Canvas, DrawBatch, InputEvent, MouseButton, UiState, Vertex},
};
use euclid::{
    default::{Point2D, Rect, Size2D},
    point2, size2,
};
use wgpu::util::DeviceExt;
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize},
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
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

    pub fn run(self) -> ! {
        futures::executor::block_on(self.run_async());
        panic!("Should never get here");
    }

    async fn run_async(mut self) -> ! {
        // Winit setup
        //
        let event_loop = EventLoop::new();
        let (_pos, size) = window_geometry(self.config.resolution, &event_loop);
        let window = WindowBuilder::new()
            .with_title(&self.config.window_title)
            .with_inner_size(size)
            .build(&event_loop)
            .unwrap();
        let size = window.inner_size();
        let (width, height) = (size.width, size.height);

        // WGPU setup
        //
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .expect("No WGPU adapter found");

        let (device, mut queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("Failed to create device");

        let mut sc_desc = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width,
            height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        surface.configure(&device, &sc_desc);

        // WGPU textures that contain the atlased graphics from the application.
        let mut textures: Vec<Texture> = Vec::new();

        let gfx = Gfx::new(&device, self.config.resolution);

        // Main loop
        //
        let mut render_buffer = RenderBuffer::new(&device, self.config.resolution);
        render_buffer.update_canvas_pos(size2(sc_desc.width, sc_desc.height));

        let mut input_events = Vec::new();
        let mut ui = UiState::default();
        let mut running = true;
        let mut redraw_requested = false;
        // Cached position for returning from fullscreen mode.
        let mut restore_position = window.outer_position().ok();

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        sc_desc.width = size.width;
                        sc_desc.height = size.height;
                        log::info!("Resizing to {:?}", (sc_desc.width, sc_desc.height));
                        surface.configure(&device, &sc_desc);
                        render_buffer.update_canvas_pos(size2(sc_desc.width, sc_desc.height));
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let pos = crate::window_to_canvas_coordinates(
                            size2(sc_desc.width, sc_desc.height),
                            self.config.resolution,
                            point2(position.x as i32, position.y as i32),
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
                                modifiers,
                            },
                        ..
                    } => {
                        let is_down = state == ElementState::Pressed;
                        if is_down
                            && virtual_keycode == Some(VirtualKeyCode::Return)
                            && modifiers.alt()
                        {
                            // Toggle fullscreen with Alt-Enter
                            if window.fullscreen().is_none() {
                                restore_position = window.outer_position().ok();
                                window.set_fullscreen(Some(Fullscreen::Borderless(
                                    window.primary_monitor(),
                                )));
                            } else {
                                window.set_fullscreen(None);
                                if let Some(pos) = restore_position {
                                    window.set_outer_position(pos);
                                }
                            }
                        }
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
                Event::RedrawRequested(_) => {
                    // Do a one-off render even if not running after redraw was requested.
                    self.scenes.update_clock();
                    redraw_requested = true;
                }
                Event::MainEventsCleared => {
                    // If window is out of focus, don't drain CPU cycles redrawing it unless a
                    // window redraw has been explicitly requested.
                    if !running && !redraw_requested {
                        return;
                    }
                    redraw_requested = false;

                    // Load atlas cache stuff to textures
                    //
                    crate::state::ENGINE_STATE
                        .lock()
                        .unwrap()
                        .atlas_cache
                        .update_system_textures(
                            &mut TextureInterface {
                                device: &device,
                                queue: &mut queue,
                            },
                            &mut textures,
                        );

                    // Main update step
                    //
                    self.scenes.update(&mut self.world);

                    let draw_batch = {
                        let screenshotter = Screenshotter {
                            device: &device,
                            queue: &mut queue,
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
                    let sub_frame = render_buffer
                        .render_buffer
                        .texture
                        .create_view(&Default::default());
                    let command_buffer = gfx.render(&device, &sub_frame, &textures, &draw_batch);
                    queue.submit(Some(command_buffer));

                    // Render buffer to window.
                    let frame = surface
                        .get_current_frame()
                        .expect("Swap chain next texture timeout");
                    let command_buffer = render_buffer.render(&device, &frame.output.texture.create_view(&wgpu::TextureViewDescriptor::default()));
                    queue.submit(Some(command_buffer));
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
) -> (LogicalPosition<f64>, LogicalSize<f64>) {
    // Don't make it a completely fullscreen window, that might put the window title bar
    // outside the screen.
    const BUFFER: f64 = 8.0;
    let width = resolution.width as f64;
    let height = resolution.height as f64;

    // Get the most conservative DPI if there's a weird multi-monitor setup.
    let dpi_factor = event_loop
        .available_monitors()
        .map(|m| m.scale_factor())
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .expect("No monitors found!");
    let monitor_size = event_loop
        .primary_monitor()
        .unwrap()
        .size()
        .to_logical::<f64>(dpi_factor);
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
        log::debug!("Gfx::new: create_bind_group_layout");
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: true,
                    },
                    count: None,
                },
            ],
        });

        // Rendering pipeline
        log::debug!("Gfx::new: create_render_pipeline");
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &device.create_shader_module(&wgpu::include_spirv!("sprite.vert.spv")),
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 2 * 4,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 4 * 4,
                            shader_location: 2,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 8 * 4,
                            shader_location: 3,
                        },
                    ],
                }],
            },

            fragment: Some(wgpu::FragmentState {
                module: &device.create_shader_module(&wgpu::include_spirv!("sprite.frag.spv")),
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },

            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        });

        log::debug!("Gfx::new finished");
        Gfx {
            pipeline,
            bind_group_layout,
            resolution,
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
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

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

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&matrix),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        struct BatchBuffers {
            bind_group: wgpu::BindGroup,
            vertex_buf: wgpu::Buffer,
            index_buf: wgpu::Buffer,
            n_indices: u32,
            clip: Option<Rect<i32>>,
        }

        let mut batch_buffers = Vec::new();
        for batch in batches {
            if batch.vertices.is_empty() {
                continue;
            }

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: textures[batch.texture].texture_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: textures[batch.texture].sampler_binding(),
                    },
                ],
            });

            let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&batch.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&batch.triangle_indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            batch_buffers.push(BatchBuffers {
                bind_group,
                vertex_buf,
                index_buf,
                n_indices: batch.triangle_indices.len() as u32,
                clip: batch.clip,
            });
        }

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            ..Default::default()
        });

        for i in 0..batch_buffers.len() {
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &batch_buffers[i].bind_group, &[]);
            rpass.set_index_buffer(
                batch_buffers[i].index_buf.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            rpass.set_vertex_buffer(0, batch_buffers[i].vertex_buf.slice(..));
            if let Some(clip) = batch_buffers[i].clip {
                // WGPU has a flipped y-axis, correct for that.
                let flipped_y =
                    self.resolution.height as i32 - 1 - clip.size.height - clip.origin.y;
                let flipped_y = flipped_y.max(0) as u32;

                rpass.set_scissor_rect(
                    clip.origin.x as u32,
                    flipped_y,
                    clip.size.width as u32,
                    clip.size.height as u32,
                );
            } else {
                rpass.set_scissor_rect(0, 0, self.resolution.width, self.resolution.height);
            }
            rpass.draw_indexed(0..batch_buffers[i].n_indices, 0, 0..1);
        }
        drop(rpass);
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
    pub fn new(device: &wgpu::Device, resolution: Size2D<u32>) -> RenderBuffer {
        let canvas_pos = point2(-1.0, -1.0);

        log::debug!("RenderBuffer::new bind_group_layout");
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: true,
                    },
                    count: None,
                },
            ],
        });

        log::debug!("RenderBuffer::new create_render_pipeline");
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &device.create_shader_module(&wgpu::include_spirv!("blit.vert.spv")),
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &device.create_shader_module(&wgpu::include_spirv!("blit.frag.spv")),
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },

            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        });

        let render_buffer = Texture::new_target(device, resolution.width, resolution.height);
        // TODO: Add depth buffer.

        log::debug!("RenderBuffer::new finished");
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

    pub fn render(&self, device: &wgpu::Device, target: &wgpu::TextureView) -> wgpu::CommandBuffer {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.canvas_pos.to_array()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.render_buffer.texture_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.render_buffer.sampler_binding(),
                },
            ],
        });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            ..Default::default()
        });

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.draw(0..4, 0..1);
        drop(rpass);

        encoder.finish()
    }
}

pub(crate) struct Screenshotter<'a> {
    device: &'a wgpu::Device,
    queue: &'a mut wgpu::Queue,
    render_buffer: &'a RenderBuffer,
}

impl<'a> Screenshotter<'a> {
    pub fn screenshot(&mut self, cb: impl FnOnce(image::RgbImage) + Send + 'static) {
        let (width, height) = (
            self.render_buffer.resolution.width,
            self.render_buffer.resolution.height,
        );

        // Copy render buffer texture into a readable WGPU buffer.
        let output_buffer = {
            let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (width * height) as u64 * 4,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let texture_extent = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };

            let command_buffer = {
                let mut encoder = self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                // Copy the data from the texture to the buffer
                encoder.copy_texture_to_buffer(
                    wgpu::ImageCopyTexture {
                        texture: &self.render_buffer.render_buffer.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: Default::default()
                    },
                    wgpu::ImageCopyBuffer {
                        buffer: &output_buffer,
                        layout: wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(std::num::NonZeroU32::new(4 * width).unwrap()),
                            rows_per_image: Some(std::num::NonZeroU32::new(height).unwrap()),
                        },
                    },
                    texture_extent,
                );

                encoder.finish()
            };

            self.queue.submit(Some(command_buffer));
            output_buffer
        };

        let _end = (width * height) as u64 * 4;
        let future = output_buffer.slice(..).map_async(wgpu::MapMode::Read);

        std::thread::spawn(move || {
            futures::executor::block_on(future).unwrap();
            let bytes = &*output_buffer.slice(..).get_mapped_range();
            let image = image::RgbImage::from_fn(width, height, |x, y| {
                let i = (x * 4 + (height - 1 - y) * width * 4) as usize;
                image::Rgb([bytes[i + 2], bytes[i + 1], bytes[i]])
            });
            cb(image);
        });

        self.device.poll(wgpu::Maintain::Wait);
    }
}

// Texture wrapper
//

struct TextureInterface<'a> {
    device: &'a wgpu::Device,
    queue: &'a mut wgpu::Queue,
}

impl<'a> crate::atlas_cache::TextureInterface for TextureInterface<'a> {
    type Texture = Texture;

    fn update_texture(&mut self, texture: &mut Self::Texture, image: &image::RgbaImage) {
        texture.blit(self.device, self.queue, &image.as_flat_samples().samples);
    }

    fn new_texture(&mut self, size: Size2D<u32>) -> Self::Texture {
        Texture::new(self.device, size.width, size.height)
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
    pub fn blit(&self, device: &wgpu::Device, queue: &mut wgpu::Queue, bytes: &[u8]) {
        assert_eq!(
            bytes.len() as u32,
            4 * self.extent.width * self.extent.height
        );

        let temp_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &bytes,
            usage: wgpu::BufferUsages::COPY_SRC,
        });
        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        init_encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &temp_buf,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(std::num::NonZeroU32::new(4 * self.extent.width).unwrap()),
                    rows_per_image: Some(std::num::NonZeroU32::new(self.extent.height).unwrap()),
                },
            },
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: Default::default()
            },
            self.extent,
        );
        queue.submit(Some(init_encoder.finish()));
    }

    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Texture {
        Texture::new_typed(
            device,
            width,
            height,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        )
    }

    pub fn new_target(device: &wgpu::Device, width: u32, height: u32) -> Texture {
        Texture::new_typed(
            device,
            width,
            height,
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
        )
    }

    fn new_typed(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Texture {
        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        // Generate Texture object.
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
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
