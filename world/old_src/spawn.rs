use std::rand;
use std::rand::Rng;
use spatial::{Location, Position};
use system::{World};
use area::Area;
use mapgen::AreaSpec;
use mobs::{MobType, MobComp, MOB_KINDS};

/// Game object factory.
pub trait Spawn {
    fn spawn_loc(&mut self) -> Option<Location>;
    fn random_mob_type(&mut self, spec: &AreaSpec) -> MobType;
    fn gen_mobs(&mut self, spec: &AreaSpec);
}

impl Spawn for World {
    fn spawn_loc(&mut self) -> Option<Location> {
        // Maybe use a RNG stored in self later.
        rand::task_rng()
            .choose(self.open_locs().as_slice())
            .map(|&x| x)
    }

    fn random_mob_type(&mut self, spec: &AreaSpec) -> MobType {
        // TODO: Use world seed based rng

        let typs = MOB_KINDS.iter()
            .filter(|mk| mk.area_spec.can_spawn(spec))
            .map(|mk| mk.typ)
            .collect::<Vec<MobType>>();

        rand::task_rng().choose(typs.as_slice())
            .map(|&x| x)
            .unwrap()
    }

    fn gen_mobs(&mut self, spec: &AreaSpec) {
        let spawn_count = 59;

        for _ in range(0u, spawn_count) {
            match self.spawn_loc() {
                None => return,
                Some(loc) => {
                    let mut e = self.new_entity();
                    e.set_component(MobComp::new(self.random_mob_type(spec)));
                    e.set_location(loc);
                }
            }
        }
    }
}
