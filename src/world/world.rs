struct World {
    seed: uint32,
}

impl World {
    pub fn new(seed: uint32) -> World {
        World {
            seed: seed,
        }
    }

    pub fn terrain_get(&self, loc: Location) -> Option<TerrainType> {
        fail!("TODO");
    }

    pub fn terrain_set(&mut self, loc: Location, t: TerrainType) {
        fail!("TODO"); 
    }

    pub fn terrain_clear(&mut self, loc: Location) {
        fail!("TODO");
    }

    pub fn insert_mob(&mut self, mob: Mob) -> MobId {
        fail!("TODO");
    }

    pub fn remove_mob(&mut self, id: MobId) {
        fail!("TODO");
    }

    // pub fn iter_mobs(

    pub fn find_mut_mob<'a>(&'a mut self, id: MobId) Option<&'a mut Mob> {
        fail!("TODO");
    }

    pub fn rng_seed(&self) -> uint32 { self.seed }
}
