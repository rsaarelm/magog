use image::{GenericImage, ImageBuf, Rgba};
use gfx::{Device, DeviceHelper, CommandBuffer, ToSlice};
use gfx;
use geom::{Rect, V2};
use super::{Color};

/// Custom sprite renderer.
pub struct Renderer<D: Device<C>, C: CommandBuffer> {
    pub graphics: gfx::Graphics<D, C>,
    program: gfx::ProgramHandle,
    frame: gfx::Frame,
    params: ShaderParam,
    state: gfx::DrawState,
    buffers: Vec<gfx::BufferHandle<Vertex>>,
}

impl<D: Device<C>, C: CommandBuffer> Renderer<D, C> {
    pub fn new(
        mut device: D,
        atlas_image: &ImageBuf<Rgba<u8>>) -> Renderer<D, C> {
        let program = device.link_program(
            VERTEX_SRC.clone(), FRAGMENT_SRC.clone()).unwrap();
        let atlas = Texture::from_rgba8(atlas_image, &mut device);
        let mut graphics = gfx::Graphics::new(device);

        // TODO: Size init.
        let (w, h) = (640u16, 360u16);

        let mut state = gfx::DrawState::new()
            .depth(gfx::state::Comparison::LessEqual, true)
            ;
        state.primitive.front_face = gfx::state::WindingOrder::Clockwise;

        let sampler_info = Some(graphics.device.create_sampler(
            gfx::tex::SamplerInfo::new(gfx::tex::FilterMethod::Scale, gfx::tex::WrapMode::Clamp)));

        let params = ShaderParam {
            s_texture: (atlas.tex, sampler_info),
        };

        Renderer {
            graphics: graphics,
            program: program,
            frame: gfx::Frame::new(w, h),
            params: params,
            state: state,
            buffers: Vec::new(),
        }
    }

    fn clear_buffers(&mut self) {
        loop {
            if let Some(b) = self.buffers.remove(0) {
                self.graphics.device.delete_buffer(b);
            } else {
                break;
            }
        }
    }

    pub fn clear<C: Color>(&mut self, color: &C) {
        self.graphics.clear(
            gfx::ClearData {
                color: color.to_rgba(),
                depth: 1.0,
                stencil: 0,
            }, gfx::COLOR | gfx::DEPTH,
            &self.frame);
    }

    pub fn end_frame(&mut self) {
        self.graphics.end_frame();

        self.clear_buffers();
    }

    pub fn set_window_size(&mut self, (w, h): (i32, i32)) {
        self.frame = gfx::Frame::new(w as u16, h as u16);
    }

    pub fn scissor(&mut self, Rect(V2(ax, ay), V2(aw, ah)): Rect<u32>) {
        self.state.scissor = Some(gfx::Rect{
            x: ax as u16, y: ay as u16, w: aw as u16, h: ah as u16
        });
    }

    fn draw(&mut self, data: &[Vertex], prim: gfx::PrimitiveType) {
        let buf = self.graphics.device.create_buffer(data.len(), gfx::BufferUsage::Stream);
        self.graphics.device.update_buffer(buf, data, 0);
        let mesh = gfx::Mesh::from_format(buf, data.len() as u32);
        let batch = self.graphics.make_batch(
            &self.program,
            &mesh,
            mesh.to_slice(prim),
            &self.state).unwrap();

        self.graphics.draw(&batch, &self.params, &self.frame);

        self.buffers.push(buf);
    }

    pub fn draw_triangles(&mut self, data: &[Vertex]) {
        self.state.primitive.method = gfx::state::RasterMethod::Fill(gfx::state::CullMode::Back);
        self.draw(data, gfx::PrimitiveType::TriangleList);
    }
}

static VERTEX_SRC: gfx::ShaderSource<'static> = shaders! {
GLSL_120: b"
    #version 120

    attribute vec3 a_pos;
    attribute vec4 a_color;
    attribute vec2 a_tex_coord;

    varying vec2 v_tex_coord;
    varying vec4 v_color;

    void main() {
        v_tex_coord = a_tex_coord;
        v_color = a_color;
        gl_Position = vec4(a_pos, 1.0);
    }
"
};

static FRAGMENT_SRC: gfx::ShaderSource<'static> = shaders! {
GLSL_120: b"
    #version 120

    uniform sampler2D s_texture;

    varying vec2 v_tex_coord;
    varying vec4 v_color;

    void main() {
        vec4 tex_color = texture2D(s_texture, v_tex_coord);
        if (tex_color.a == 0.0) discard;
        gl_FragColor = v_color * tex_color;
    }
"
};

#[shader_param(Program)]
struct ShaderParam {
    s_texture: gfx::shade::TextureParam,
}

#[vertex_format]
pub struct Vertex {
    #[name = "a_pos"]
    pub pos: [f32, ..3],

    #[name = "a_color"]
    pub color: [f32, ..4],

    #[name = "a_tex_coord"]
    pub tex_coord: [f32, ..2],
}

impl Clone for Vertex {
    fn clone(&self) -> Vertex { *self }
}

struct Texture {
    tex: gfx::TextureHandle,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    fn from_rgba8<D: Device<C>, C: CommandBuffer>(
        img: &ImageBuf<Rgba<u8>>,
        d: &mut D) -> Texture {
        let (w, h) = img.dimensions();
        let mut info = gfx::tex::TextureInfo::new();
        info.width = w as u16;
        info.height = h as u16;
        info.kind = gfx::tex::TextureKind::Texture2D;
        info.format = gfx::tex::RGBA8;

        let tex = d.create_texture(info).unwrap();
        d.update_texture(&tex, &info.to_image_info(), img.pixelbuf()).unwrap();

        Texture {
            tex: tex,
            width: w,
            height: h,
        }
    }
}
