use calx::{V2};

pub struct World {
    pub player_pos: V2<f32>,
    pub camera_pos: V2<f32>,
}

impl World {
    pub fn new() -> World {
        World {
            player_pos: V2(8.0, 8.0),
            camera_pos: V2(0.0, 0.0),
        }
    }

    pub fn _update(&mut self) {
        unimplemented!();
    }

    pub fn _set_dest(&mut self, _cell: V2<i32>) {
        unimplemented!();
    }
}
