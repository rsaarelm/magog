// Benchmark from https://github.com/lschmierer/ecs_bench

#![feature(test)]
extern crate test;
use test::Bencher;

#[macro_use]
extern crate serde_derive;
extern crate serde;

#[macro_use]
extern crate calx_ecs;

use calx_ecs::Entity;

/// Entities with velocity and position component.
pub const N_POS_VEL: usize = 1000;

/// Entities with position component only.
pub const N_POS: usize = 9000;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

Ecs! {
    pos: Position,
    vel: Velocity,
}

fn build() -> Ecs {
    let mut ecs = Ecs::new();

    // setup entities
    for _ in 0..N_POS_VEL {
        let e = ecs.make();
        ecs.pos.insert(e, Position { x: 0.0, y: 0.0 });
        ecs.vel.insert(e, Velocity { dx: 0.0, dy: 0.0 });
    }
    for _ in 0..N_POS {
        let e = ecs.make();
        ecs.pos.insert(e, Position { x: 0.0, y: 0.0 });
    }

    ecs
}

#[bench]
fn bench_build(b: &mut Bencher) {
    b.iter(build);
}

#[bench]
fn bench_update(b: &mut Bencher) {
    let mut ecs = build();

    b.iter(|| {
        // Update
        let with_velocity: Vec<Entity> = ecs.vel.ent_iter().cloned().collect();
        for &e in &with_velocity {
            let vel = ecs.vel[e];
            ecs.pos.get_mut(e).map(|pos| {
                pos.x += vel.dx;
                pos.y += vel.dy;
            });
        }

        // Render
        for &e in ecs.iter() {
            ecs.pos.get(e).map(|_pos| {});
        }
    });
}
