use calx::{V2, lerp};
use calx::{HexGeom, astar_path_with};
use ::{PathPos};

// Temporary structures to support a single player mob.

#[derive(PartialEq, Copy, Clone, Debug)]
enum Anim {
    Idle,
    Move(f32, V2<i32>),
}

#[derive(Debug)]
pub struct Mob {
    pub pos: V2<i32>,
    /// Position in the direction of animation offset.
    anim: Anim,
    path: Vec<V2<i32>>,
}

impl Mob {
    pub fn new() -> Mob {
        Mob {
            pos: V2(9, 7),
            anim: Anim::Idle,
            path: Vec::new(),
        }
    }

    pub fn draw_pos(&self) -> V2<f32> {
        match self.anim {
            Anim::Idle => self.pos.map(|x| x as f32),
            Anim::Move(t, off_pos) =>
                lerp(off_pos.map(|x| x as f32), self.pos.map(|x| x as f32), t)
        }
    }

    pub fn update(&mut self) {
        const WALK_SPEED: f32 = 0.25;
        match self.anim {
            Anim::Move(t, off_pos) => {
                let t = t + WALK_SPEED;
                if t >= 1.0 {
                    self.anim = Anim::Idle;
                } else {
                    self.anim = Anim::Move(t, off_pos)
                }
            }
            _ => {}
        }

        if !self.path.is_empty() && self.anim == Anim::Idle {
            self.anim = Anim::Move(WALK_SPEED, self.pos);
            self.pos = self.path.remove(0);
        }
    }
}

pub struct World {
    player: Mob,
}

impl World {
    pub fn new() -> World {
        World {
            player: Mob::new()
        }
    }

    pub fn player_draw_pos(&self) -> V2<f32> {
        self.player.draw_pos()
    }

    pub fn update(&mut self) {
        self.player.update();
    }

    pub fn set_dest(&mut self, cell: V2<i32>) {
        if let Some(path) = astar_path_with(
            |x, y| (x.0-y.0).hex_dist(), PathPos(self.player.pos), PathPos(cell), 1000) {
            self.player.path = path.into_iter().map(|x| x.0).collect();
        }
    }
}
