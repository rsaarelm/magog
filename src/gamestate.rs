use std::io::File;
use std::io::fs::PathExtensions;
use std::collections::HashMap;
use calx::color;
use calx::Context;
use calx::Event;
use calx::Key;
use calx::{Fonter, V2};
use world;
use world::action;
use world::action::Input::{Step, Melee};
use world::action::ControlState::*;
use world::{Msg, FovStatus};
use world::Dir6;
use world::Dir6::*;
use world::{Entity};
use world::item::{Slot};
use worldview;
use sprite::{WorldSprites, GibSprite};
use tilecache;
use tilecache::icon;
use msg_queue::MsgQueue;

pub struct GameState {
    /// Transient effect sprites drawn in game world view.
    world_spr: WorldSprites,
    /// Counters for entities with flashing damage animation.
    damage_timers: HashMap<Entity, uint>,

    /// Flag for autoexploration.
    // TODO: Probably going to need a general "ongoing activity" system at
    // some point.
    exploring: bool,

    msg: MsgQueue,
    ui_state: UiState,
}

enum UiState {
    Gameplay,
    Inventory
}

impl GameState {
    pub fn new(seed: Option<u32>) -> GameState {
        world::init_world(seed);
        GameState {
            world_spr: WorldSprites::new(),
            damage_timers: HashMap::new(),
            exploring: false,
            msg: MsgQueue::new(),
            ui_state: UiState::Gameplay,
        }
    }

    fn draw_player_ui(&mut self, ctx: &mut Context, player: Entity) {
        let hp = player.hp();
        let max_hp = player.max_hp();

        // Draw heart containers.
        for i in range(0, (max_hp + 1) / 2) {
            let pos = V2(i as int * 8, 8);
            let idx = if hp >= (i + 1) * 2 { icon::HEART }
                else if hp == i * 2 + 1 { icon::HALF_HEART }
                else { icon::NO_HEART };
            ctx.draw_image(pos, 0.0, tilecache::get(idx), &color::FIREBRICK);
        }
    }

    fn base_paint(&mut self, ctx: &mut Context) {
        let camera = world::camera();
        worldview::draw_world(&camera, ctx, &self.damage_timers);

        self.world_spr.draw(|x| (camera + x).fov_status() == Some(FovStatus::Seen), &camera, ctx);
        self.world_spr.update();

        let location_name = camera.name();
        let _ = write!(&mut ctx.text_writer(V2(640 - location_name.len() as int * 8, 8), 0.1, color::LIGHTGRAY)
                       .set_border(color::BLACK),
                       "{}", location_name);

        self.msg.draw(ctx);
        if let Some(player) = action::player() {
            self.draw_player_ui(ctx, player);
        }

        let fps = 1.0 / ctx.render_duration;
        let _ = write!(&mut ctx.text_writer(V2(0, 16), 0.1, color::LIGHTGREEN)
                       .set_border(color::BLACK),
                       "FPS {:.0}", fps);
    }

    fn base_update(&mut self, ctx: &mut Context) {
        // Process events
        loop {
            match world::pop_msg() {
                Some(Msg::Gib(loc)) => {
                    self.world_spr.add(box GibSprite::new(loc));
                }
                Some(Msg::Damage(entity)) => {
                    self.damage_timers.insert(entity, 2);
                }
                Some(Msg::Text(txt)) => {
                    self.msg.msg(txt)
                }
                Some(Msg::Caption(txt)) => {
                    self.msg.caption(txt)
                }
                Some(x) => {
                    println!("Unhandled Msg type {}", x);
                }
                None => break
            }
        }

        self.base_paint(ctx);

        if action::control_state() == ReadyToUpdate {
            action::update();
        }

        if self.exploring {
            if action::control_state() == AwaitingInput {
                self.exploring = self.autoexplore();
            }
        }

        self.damage_timers = self.damage_timers.clone().into_iter()
            .filter(|&(_, t)| t > 0u)
            .map(|(e, t)| (e, t - 1))
            .collect();

        self.msg.update();
    }

    fn inventory_update(&mut self, ctx: &mut Context) {
        let player = action::player().unwrap();
        let mut cursor = ctx.text_writer(V2(0, 8), 0.1, color::GAINSBORO);
        for slot_data in SLOT_DATA.iter() {
            let name = match player.equipped(slot_data.slot) {
                Some(item) => item.name(),
                None => "".to_string()
            };
            let _ = write!(&mut cursor, "{}] {}: {}\n",
                slot_data.key, slot_data.name, name);
        }
    }

    pub fn inventory_process(&mut self, event: Event) -> bool {
        let player = action::player().unwrap();
        match event {
            Event::Render(ctx) => {
                self.update(ctx);
            }
            Event::KeyPressed(Key::Escape) | Event::KeyPressed(Key::Tab) => {
                self.ui_state = UiState::Gameplay
            }
            Event::KeyPressed(_) => {}

            Event::Char(ch) => {
                for slot_data in SLOT_DATA.iter() {
                    if ch == slot_data.key {
                        if slot_data.slot.is_gear_slot() {
                            // Unequip gear
                            match player.free_bag_slot() {
                                None => {
                                    // No room in bag, can't unequip until
                                    // drop something.
                                    // TODO: Message about full bag.
                                }
                                Some(swap_slot) => {
                                    player.swap_equipped(slot_data.slot, swap_slot);
                                }
                            }
                        }
                        if slot_data.slot.is_bag_slot() {
                            // Bag items get equipped if they have are gear
                            // with a preferred slot.
                            if let Some(item) = player.equipped(slot_data.slot) {
                                let equip_slots = item.equip_slots();
                                for &swap_slot in equip_slots.iter() {
                                    if player.equipped(swap_slot).is_none() {
                                        player.swap_equipped(slot_data.slot, swap_slot);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if ch == slot_data.key.to_uppercase() {
                        // Drop item in slot.
                        if let Some(item) = player.equipped(slot_data.slot) {
                            item.place(player.location().unwrap());
                        }
                        break;
                    }
                }
            }

            _ => ()
        }
        true
    }


    /// Repaint view, update game world if needed.
    pub fn update(&mut self, ctx: &mut Context) {
        ctx.clear(&color::BLACK);

        match self.ui_state {
            UiState::Gameplay => self.base_update(ctx),
            UiState::Inventory => self.inventory_update(ctx),
        }
    }

    pub fn save_game(&self) {
        let save_data = world::save();
        let mut file = File::create(&Path::new("/tmp/magog_save.json"));
        file.write_str(save_data.as_slice()).unwrap();
    }

    pub fn load_game(&mut self) {
        let path = Path::new("/tmp/magog_save.json");
        if !path.exists() { return; }
        let save_data = File::open(&path).read_to_string().unwrap();
        // TODO: Handle failed load nicely.
        world::load(save_data.as_slice()).unwrap();
    }

    fn smart_move(&mut self, dir: Dir6) {
        let player = action::player().unwrap();
        let loc = player.location().unwrap();
        for &d in vec![dir, dir + 1, dir - 1].iter() {
            let target_loc = loc + d.to_v2();
            if target_loc.has_mobs() {
                action::input(Melee(d));
                return;
            } else if player.can_step(d) {
                action::input(Step(d));
                return;
            }
        }
    }

    fn autoexplore(&mut self) -> bool {
        let player = action::player().unwrap();
        if player.is_threatened() {
            return false;
        }
        if let Some(pathing) = action::autoexplore_map(32) {
            let loc = player.location().unwrap();
            let steps = pathing.sorted_neighbors(&loc);
            if steps.len() == 0 {
                return false;
            }

            action::input(Step(loc.dir6_towards(steps[0]).unwrap()));
            return true;
        }

        false
    }

    /// Context-specific interaction with the current cell.
    fn interact(&mut self) {
        let player = action::player().unwrap();
        let loc = player.location().unwrap();
        if let Some(item) = loc.top_item() {
            player.pick_up(item);
            return;
        }
    }

    /// Process a player control keypress.
    pub fn gameplay_process_key(&mut self, key: Key) -> bool {
        if action::control_state() != AwaitingInput {
            return false;
        }

        if self.exploring {
            self.exploring = false;
        }

        match key {
            Key::Q | Key::Pad7 => { self.smart_move(NorthWest); }
            Key::W | Key::Pad8 | Key::Up => { self.smart_move(North); }
            Key::E | Key::Pad9 => { self.smart_move(NorthEast); }
            Key::A | Key::Pad1 => { self.smart_move(SouthWest); }
            Key::S | Key::Pad2 | Key::Down => { self.smart_move(South); }
            Key::D | Key::Pad3 => { self.smart_move(SouthEast); }

            Key::Enter => { self.interact(); }
            Key::X => { self.exploring = true; }

            // Open inventory
            Key::Tab => { self.ui_state = UiState::Inventory; }

            Key::F5 => { self.save_game(); }
            Key::F9 => { self.load_game(); }
            _ => { return false; }
        }
        return true;
    }

    pub fn gameplay_process(&mut self, event: Event) -> bool {
        match event {
            Event::Render(ctx) => {
                self.update(ctx);
            }
            // TODO: Better quit confirmation than just pressing esc.
            Event::KeyPressed(Key::Escape) => {
                return false;
            }
            Event::KeyPressed(k) => {
                self.gameplay_process_key(k);
            }

            Event::Char(ch) => {
                // TODO: Chars and keypresses in same lookup (use variants?)
                match ch {
                    // Debug
                    '>' => { action::next_level(); }
                    _ => ()
                }
            }

            _ => ()
        }
        true
    }

    pub fn process(&mut self, event: Event) -> bool {
        match self.ui_state {
            UiState::Gameplay => self.gameplay_process(event),
            UiState::Inventory => self.inventory_process(event),
        }
    }
}

struct SlotData {
    key: char,
    slot: Slot,
    name: &'static str,
}

static SLOT_DATA: [SlotData; 34] = [
    SlotData { key: '1', slot: Slot::Spell1,     name: "Ability" },
    SlotData { key: '2', slot: Slot::Spell2,     name: "Ability" },
    SlotData { key: '3', slot: Slot::Spell3,     name: "Ability" },
    SlotData { key: '4', slot: Slot::Spell4,     name: "Ability" },
    SlotData { key: '5', slot: Slot::Spell5,     name: "Ability" },
    SlotData { key: '6', slot: Slot::Spell6,     name: "Ability" },
    SlotData { key: '7', slot: Slot::Spell7,     name: "Ability" },
    SlotData { key: '8', slot: Slot::Spell8,     name: "Ability" },
    SlotData { key: 'a', slot: Slot::Melee,      name: "Weapon " },
    SlotData { key: 'b', slot: Slot::Ranged,     name: "Ranged " },
    SlotData { key: 'c', slot: Slot::Head,       name: "Head   " },
    SlotData { key: 'd', slot: Slot::Body,       name: "Body   " },
    SlotData { key: 'e', slot: Slot::Feet,       name: "Feet   " },
    SlotData { key: 'f', slot: Slot::TrinketF,   name: "Trinket" },
    SlotData { key: 'g', slot: Slot::TrinketG,   name: "Trinket" },
    SlotData { key: 'h', slot: Slot::TrinketH,   name: "Trinket" },
    SlotData { key: 'i', slot: Slot::TrinketI,   name: "Trinket" },
    SlotData { key: 'j', slot: Slot::InventoryJ, name: "       " },
    SlotData { key: 'k', slot: Slot::InventoryK, name: "       " },
    SlotData { key: 'l', slot: Slot::InventoryL, name: "       " },
    SlotData { key: 'm', slot: Slot::InventoryM, name: "       " },
    SlotData { key: 'n', slot: Slot::InventoryN, name: "       " },
    SlotData { key: 'o', slot: Slot::InventoryO, name: "       " },
    SlotData { key: 'p', slot: Slot::InventoryP, name: "       " },
    SlotData { key: 'q', slot: Slot::InventoryQ, name: "       " },
    SlotData { key: 'r', slot: Slot::InventoryR, name: "       " },
    SlotData { key: 's', slot: Slot::InventoryS, name: "       " },
    SlotData { key: 't', slot: Slot::InventoryT, name: "       " },
    SlotData { key: 'u', slot: Slot::InventoryU, name: "       " },
    SlotData { key: 'v', slot: Slot::InventoryV, name: "       " },
    SlotData { key: 'w', slot: Slot::InventoryW, name: "       " },
    SlotData { key: 'x', slot: Slot::InventoryX, name: "       " },
    SlotData { key: 'y', slot: Slot::InventoryY, name: "       " },
    SlotData { key: 'z', slot: Slot::InventoryZ, name: "       " },
];
