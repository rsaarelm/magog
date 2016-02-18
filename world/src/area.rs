use rand::StdRng;
use rand::SeedableRng;
use std::collections::BTreeMap;
use calx_ecs::Entity;
use location::Location;
use content::{self, AreaSpec, TerrainType, Biome, StaticArea, FormType};
use world::World;
use form::Spawner;
use field::Field;
use query;
use action;

pub fn start_level(w: &mut World, depth: i32) {
    clear_nonplayers(w);
    w.flags.depth = depth;
    init_area(w, depth);
}

fn clear_nonplayers(w: &mut World) {
    let po = query::player(w);
    let entities: Vec<Entity> = w.ecs.iter().map(|&e| e).collect();
    for e in entities.into_iter() {
        // Don't destroy player or player's inventory.
        if let Some(p) = po {
            if e == p || w.spatial.contains(p, e) {
                continue;
            }
        }

        if query::location(w, e).is_some() {
            w.ecs.remove(e);
        }
    }
}

pub fn next_level(w: &mut World) {
    // This is assuming a really simple, original Rogue style descent-only, no
    // persistent maps style world.
    let new_depth = query::current_depth(w) + 1;
    start_level(w, new_depth);
    // 1st level is the overworld, so we want to call depth=2, first dungeon
    // level as "Depth 1" in game.
    caption!("Depth {}", new_depth - 1);
}

fn init_area(w: &mut World, depth: i32) {
    let biome = match depth {
        1 => Biome::Overland,
        _ => Biome::Dungeon,
    };

    let spec = AreaSpec::new(biome, depth);
    let mut rng: StdRng =
        SeedableRng::from_seed(&[w.flags.seed as usize + spec.depth as usize][..]);
    let static_area = if spec.depth == 1 {
        content::herringbone(&mut rng, &spec)
    } else {
        content::herringbone(&mut rng, &spec)
        // FIXME: rooms_and_corridors gen currently disabled. Also it was
        // kinda lame.
        // content::rooms_and_corridors(&mut rng, spec.depth)
    };

    let origin = Location::new(0, 0);
    let mut terrain = Field::new(biome.default_terrain());
    for (&pos, &t) in static_area.terrain.iter() {
        terrain.set(origin + pos, t);
    }

    w.terrain = terrain;

    spawn_player(w, origin + static_area.player_entrance);
    spawn_entities(w, origin, &spec, &static_area);
}

fn spawn_player(w: &mut World, start_loc: Location) -> Entity {
    // Either reuse the existing player or create a new one.
    let player = match query::player(w) {
        Some(player) => player,
        None => {
            // TODO: Use factory.
            use calx_color::color;
            use content::Brush;
            use components::{Desc, MapMemory, Health, Brain, BrainState, Alignment};
            use stats::Stats;
            use stats::Intrinsic::*;
            let player = w.ecs.make();
            w.ecs.desc.insert(player,
                              Desc::new("player", Brush::Human, color::AZURE));
            w.ecs.map_memory.insert(player, MapMemory::new());
            w.ecs.brain.insert(player,
                               Brain {
                                   state: BrainState::PlayerControl,
                                   alignment: Alignment::Good,
                               });
            w.ecs.stats.insert(player, Stats::new(10, &[Hands]).mana(5));
            w.ecs.health.insert(player, Health::new());
            action::recompose_stats(w, player);
            w.flags.player = Some(player);

            player
        }
    };

    action::forget_map(w, player);
    action::place_entity(w, player, start_loc);
    player
}

fn spawn_entities(w: &mut World,
                  origin: Location,
                  spec: &AreaSpec,
                  area: &StaticArea) {
    let mut spawner = Spawner::new();

    for &(pos, typ) in area.spawns.iter() {
        if let Some(form) = spawner.spawn(&mut w.flags.rng, spec, typ) {
            let e = form.build(w);
            action::place_entity(w, e, origin + pos);
        } else {
            println!("Failed to spawn {:?} for {:?}", typ, spec);
        }
    }
}
