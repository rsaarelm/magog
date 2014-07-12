use std::rand;
use std::rand::Rng;
use world::system::{Location, World};
use world::area::Area;
use world::mobs;
use world::mobs::{MobType, MobComp};

pub trait Spawn {
    fn spawn_loc(&mut self) -> Option<Location>;
    fn random_mob_type(&mut self) -> MobType;
    fn gen_mobs(&mut self);
}

impl Spawn for World {
    fn spawn_loc(&mut self) -> Option<Location> {
        // Maybe use a RNG stored in self later.
        rand::task_rng()
            .choose(self.open_locs().as_slice())
            .map(|&x| x)
    }

    fn random_mob_type(&mut self) -> MobType {
        // TODO: Spawn harder monsters in deeper depths
        // TODO: Use world seed based rng
        rand::task_rng()
            .choose(&[
                    mobs::Dreg,
                    mobs::GridBug,
                    mobs::Serpent,
                    ])
            .map(|&x| x)
            .unwrap()
    }

    fn gen_mobs(&mut self) {
        let spawn_count = 59;

        for _ in range(0u, spawn_count) {
            match self.spawn_loc() {
                None => return,
                Some(loc) => {
                    let mut e = self.new_entity();
                    e.set_component(MobComp::new(self.random_mob_type()));
                    e.set_component(loc);
                }
            }
        }
    }
}
