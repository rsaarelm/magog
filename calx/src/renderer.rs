pub struct Renderer {
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
        }
    }
}

#[vertex_format]
#[deriving(Copy)]
pub struct Vertex {
    #[name = "a_pos"]
    pub pos: [f32, ..3],

    #[name = "a_color"]
    pub color: [f32, ..4],

    #[name = "a_tex_coord"]
    pub tex_coord: [f32, ..2],
}
