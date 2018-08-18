#![feature(rust_2018_preview)]
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

pub const N: usize = 10000;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct R {
    pub x: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct W1 {
    pub x: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct W2 {
    pub x: f32,
}

build_ecs! {
    r: R,
    w1: W1,
    w2: W2,
}

fn build() -> Ecs {
    let mut ecs = Ecs::new();

    for _ in 0..N {
        let e = ecs.make();
        ecs.r.insert(e, R { x: 0.0 });
        ecs.w1.insert(e, W1 { x: 0.0 });
        ecs.w2.insert(e, W2 { x: 0.0 });
    }

    ecs
}

#[bench]
fn bench_build(b: &mut Bencher) { b.iter(build); }

#[bench]
fn bench_update(b: &mut Bencher) {
    let mut ecs = build();

    b.iter(|| {
        let es: Vec<Entity> = ecs.r.ent_iter().cloned().collect();
        for &e in &es {
            let rx = ecs.r[e].x;
            ecs.w1.get_mut(e).map(|w1| w1.x = rx);
        }

        for &e in &es {
            let rx = ecs.r[e].x;
            ecs.w2.get_mut(e).map(|w2| w2.x = rx);
        }
    });
}
