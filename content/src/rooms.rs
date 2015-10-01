use std::collections::{BTreeSet};
use num::{Integer};
use rand::{Rng};
use calx::{V2, Rect, RngExt, clamp, KernelTerrain};
use ::{StaticArea, FormType};
use terrain::{TerrainType};

/// Rooms have random size, but they are all placed inside grid cells.
/// Includes one line of buffer beyond the extents of the room plus outer
/// walls to allow the winding corridor to the neighboring room to always fit
/// in.
pub static CELL_SIZE: i32 = 16;

/// Generate a classic rooms and corridors rogue map.
pub fn rooms_and_corridors<R: Rng>(
    rng: &mut R, depth: i32) -> StaticArea<FormType> {
    // TODO: Vary room number.
    let num_rooms = 6;

    let mut _exit_generated = false;
    let mut area = StaticArea::new();

    let mut rooms: BTreeSet<Node> = BTreeSet::new();
    let mut walls: BTreeSet<Wall> = BTreeSet::new();

    let starting_room = Node(V2(0, 0));
    rooms.insert(starting_room);
    walls.extend(starting_room.walls().into_iter());

    dig_room(&mut area, rng, starting_room, Entrance);

    while rooms.len() < num_rooms {
        let wall = *rng.choose(&walls.clone().into_iter().collect::<Vec<Wall>>()).expect("No expandable walls");

        let wall_rooms = wall.rooms();
        for &(entrance_dir, room) in wall_rooms.iter() {
            if !rooms.contains(&room) {
                let room_type = if rooms.len() == num_rooms - 1 {
                    _exit_generated = true;
                    // The last room is the exit.
                    Exit
                } else {
                    // Random room.
                    RoomType::new(rng, depth)
                };
                dig_room(&mut area, rng, room, room_type);
                rooms.insert(room);
                walls.extend(room.walls_except(entrance_dir).into_iter());
            }
        }

        connect_rooms(&mut area, rng, wall);

        walls.remove(&wall);
    }

    assert!(_exit_generated, "No exit generated on map");

    area
}

fn door_positions(area: &StaticArea<FormType>, node: Node, dir: Direction) -> Vec<V2<i32>> {
    let (start, fwd_dir) = match dir {
        North => (V2(0, 0), V2(0, 1)),
        East => (V2(CELL_SIZE - 2, 0), V2(-1, 0)),
        South => (V2(CELL_SIZE - 2, CELL_SIZE - 2), V2(0, -1)),
        West => (V2(0, CELL_SIZE - 2), V2(1, 0))
    };

    let start = node.origin() + start;

    let side_dir = V2(fwd_dir.1, -fwd_dir.0);

    let mut ret = Vec::new();

    for side in (1..(CELL_SIZE - 2)) {
        let origin = start + side_dir * side;
        for fwd in (0..(CELL_SIZE - 2)) {
            let pos = origin + fwd_dir * fwd;

            if area.is_open(pos + side_dir) || area.is_open(pos - side_dir) {
                // Valid exit path can't cut into open space from the side.
                break;
            }

            if let Some(t) = area.terrain.get(&pos) {
                if t.is_wall() && area.is_open(pos + fwd_dir) {
                    ret.push(pos);
                    break;
                }
            }
        }
    }

    ret
}

fn dig_room<R: Rng>(area: &mut StaticArea<FormType>, rng: &mut R, node: Node, room_type: RoomType) {
    static MIN_SIZE: i32 = 5;

    let p1 = V2(
        rng.gen_range(0, CELL_SIZE - 1 - MIN_SIZE),
        rng.gen_range(0, CELL_SIZE - 1 - MIN_SIZE));

    let dim = V2(
        rng.gen_range(MIN_SIZE, CELL_SIZE - p1.0),
        rng.gen_range(MIN_SIZE, CELL_SIZE - p1.1));

    assert!(dim.0 >= MIN_SIZE && dim.1 >= MIN_SIZE);

    let origin = node.origin();
    let room = Rect(origin + p1, dim);

    for p in room.iter() {
        if room.edge_contains(&p) {
            area.terrain.insert(p, TerrainType::Wall);
        } else {
            area.terrain.insert(p, TerrainType::Floor);
        }
    }

    let inside = Rect(room.0 + V2(1, 1), room.1 - V2(2, 2));
    for p in inside.iter() {
        if room_type == Warren || rng.one_chance_in(24) {
            area.spawns.push((p, FormType::Creature));
        }

        if rng.one_chance_in(96) {
            area.spawns.push((p, FormType::Item));
        }
    }

    if room_type == Exit {
        area.terrain.insert(room.0 + dim / 2, TerrainType::Downstairs);
    }

    if room_type == Entrance {
        area.player_entrance = room.0 + dim / 2;
    }
}

fn connect_rooms<R: Rng>(area: &mut StaticArea<FormType>, rng: &mut R, wall: Wall) {
    let rooms = wall.rooms();
    let (dir1, room1) = rooms[0];
    let (dir2, room2) = rooms[1];
    let door1 = *rng.choose(&door_positions(area, room1, dir1)).expect("No valid door positions");
    let door2 = *rng.choose(&door_positions(area, room2, dir2)).expect("No valid door positions");

    dig_tunnel(area, door1, door2);
    area.terrain.insert(door1, TerrainType::Door);
    area.terrain.insert(door2, TerrainType::Door);
}

fn dig_tunnel(area: &mut StaticArea<FormType>, p1: V2<i32>, p2: V2<i32>) {
    let (n1, n2) = (Node::from_pos(p1), Node::from_pos(p2));

    let fwd = (n2.0 - n1.0) / 2;
    assert!(fwd.0.abs() + fwd.1.abs() == 1, "Unhandled tunnel configuration");

    // Project the vector between the end points on the sideways line to get
    // the direction to move sideways towards the target point.
    let side = V2(fwd.1, -fwd.0);
    let side = side * side.dot(p2 - p1).signum();

    let mut pos = p1;

    while pos.0.mod_floor(&CELL_SIZE) != CELL_SIZE - 1 && pos.1.mod_floor(&CELL_SIZE) != CELL_SIZE - 1 {
        area.terrain.insert(pos, TerrainType::Floor);
        pos = pos + fwd;
    }

    while (p2 - pos).dot(side) != 0 {
        area.terrain.insert(pos, TerrainType::Floor);
        pos = pos + side;
    }

    while pos != p2 {
        area.terrain.insert(pos, TerrainType::Floor);
        pos = pos + fwd;
    }

    area.terrain.insert(pos, TerrainType::Floor);
}

// Nodes designate room locations. They have even coordinates.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Node(V2<i32>);

impl Node {
    fn from_pos(pos: V2<i32>) -> Node {
        Node(V2(
            (pos.0 as f32 / CELL_SIZE as f32).floor() as i32 * 2,
            (pos.1 as f32 / CELL_SIZE as f32).floor() as i32 * 2))
    }

    fn build_walls(&self, exclude: Option<Direction>) -> Vec<Wall> {
        let mut ret = Vec::new();
        if exclude != Some(North) { ret.push(Wall(self.0 + V2( 0, -1))); }
        if exclude != Some(West)  { ret.push(Wall(self.0 + V2(-1,  0))); }
        if exclude != Some(East)  { ret.push(Wall(self.0 + V2( 1,  0))); }
        if exclude != Some(South) { ret.push(Wall(self.0 + V2( 0,  1))); }
        ret
    }

    pub fn walls(&self) -> Vec<Wall> {
        self.build_walls(None)
    }

    pub fn walls_except(&self, dir: Direction) -> Vec<Wall> {
        self.build_walls(Some(dir))
    }

    fn _is_valid(&self) -> bool {
        (self.0).0 % 2 == 0 && (self.0).1 % 2 == 0
    }

    pub fn origin(&self) -> V2<i32> {
        assert!(self._is_valid(), "Off-grid node coordinates");
        V2((self.0).0 / 2 * CELL_SIZE, (self.0).1 / 2 * CELL_SIZE)
    }
}

// Walls have one odd and one even coordinate.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Wall(pub V2<i32>);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Direction {
    North,
    East,
    South,
    West
}

use self::Direction::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum WallType {
    Horizontal,
    Vertical
}

use self::WallType::*;

impl Wall {
    fn classify(&self) -> WallType {
        if (self.0).0 % 2 == 0 {
            assert!((self.0).1 % 2 != 0);
            Horizontal
        } else {
            assert!((self.0).1 % 2 == 0);
            Vertical
        }
    }

    // Direction is the direction from node towards wall.
    pub fn rooms(&self) -> [(Direction, Node); 2] {
        match self.classify() {
            Vertical => [(East, Node(self.0 + V2(-1, 0))), (West, Node(self.0 + V2(1, 0)))],
            Horizontal => [(South, Node(self.0 + V2(0, -1))), (North, Node(self.0 + V2(0, 1)))],
        }
    }

    fn _is_valid(&self) -> bool {
        ((self.0).0 % 2 == 0) ^ ((self.0).1 % 2 == 0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum RoomType {
    Entrance,
    Exit,
    Regular,
    // Full of monsters.
    Warren,
    // TODO: Prefab vaults
}

use self::RoomType::*;

impl RoomType {
    pub fn new<R: Rng>(rng: &mut R, depth: i32) -> RoomType {
        if rng.one_chance_in(16 - clamp(0, 8, depth as u32 / 2)) {
            return Warren;
        }
        Regular
    }
}

#[cfg(test)]
mod test {
    use super::Direction::*;
    use super::WallType::*;
    use calx::{V2};
    use super::{Wall, Node, CELL_SIZE};

    #[test]
    fn test_wall_pos() {
        let horizontal = Wall(V2(0, -1));
        let vertical = Wall(V2(-1, 0));

        assert_eq!(Horizontal, horizontal.classify());
        assert_eq!(Vertical, vertical.classify());

        assert_eq!([(South, Node(V2(0, -2))), (North, Node(V2(0, 0)))], horizontal.rooms());
        assert_eq!([(East, Node(V2(-2, 0))), (West, Node(V2(0, 0)))], vertical.rooms());
    }

    #[test]
    fn test_from_pos() {
        assert_eq!(Node(V2(-2, -2)), Node::from_pos(V2(-CELL_SIZE / 2, -CELL_SIZE / 2)));
        assert_eq!(Node(V2(0, 0)), Node::from_pos(V2(CELL_SIZE / 2, CELL_SIZE / 2)));
    }
}
