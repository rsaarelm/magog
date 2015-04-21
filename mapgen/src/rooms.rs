use std::collections::{BTreeSet};
use rand::{Rng};
use calx::{V2, Rect, RngExt, clamp};
use ::{StaticArea, SpawnType};
use terrain::{TerrainType};

/// Rooms have random size, but they are all placed inside grid cells.
/// Includes one line of buffer beyond the extents of the room plus outer
/// walls to allow the winding corridor to the neighboring room to always fit
/// in.
pub static CELL_SIZE: i32 = 11;

/// Generate a classic rooms and corridors rogue map.
pub fn rooms_and_corridors<R: Rng>(
    rng: &mut R, depth: i32) -> StaticArea<SpawnType> {
    // TODO: Vary room number.
    let num_rooms = 6;

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

    area
}

fn door_positions(area: &StaticArea<SpawnType>, node: Node, dir: Direction) -> Vec<V2<i32>> {
    // TODO: Examine the StaticArea for valid door positions towards direction.
    unimplemented!();
}

fn dig_room<R: Rng>(area: &mut StaticArea<SpawnType>, rng: &mut R, node: Node, room_type: RoomType) {
    // XXX: HACK STUPID VERSION
    // TODO: Dig varied room sizes.
    // TODO: Place spawns
    let origin = node.origin();
    let room = Rect(origin, V2(10, 10));
    for p in room.iter() {
        if room.edge_contains(&p) {
            area.terrain.insert(p, TerrainType::Wall);
        } else {
            area.terrain.insert(p, TerrainType::Floor);
        }
    }

    if room_type == Exit {
        area.terrain.insert(origin + V2(5, 5), TerrainType::Downstairs);
    }

    if room_type == Entrance {
        area.player_entrance = origin + V2(5, 5);
    }
}

fn connect_rooms<R: Rng>(area: &mut StaticArea<SpawnType>, rng: &mut R, wall: Wall) {
    // XXX: HACK STUPID VERSION
    // TODO: Pick random door positions.
    // TODO: Carve winding tunnel between doors.
    // TODO: Add doors.
    let p = wall.pos();
    let dir = wall.dir();
    for i in (0..CELL_SIZE) {
        area.terrain.insert(p + dir * i, TerrainType::Floor);
    }
}

// Nodes designate room locations. They have even coordinates.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Node(V2<i32>);

impl Node {
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

    /// Position in the center of the room behind the wall.
    pub fn pos(&self) -> V2<i32> {
        assert!(self._is_valid(), "Off-grid wall coordinates");
        match self.classify() {
            Vertical => V2(
                ((self.0).0 - 1) / 2 * CELL_SIZE + CELL_SIZE / 2,
                (self.0).1 / 2 * CELL_SIZE + CELL_SIZE / 2),
            Horizontal => V2(
                (self.0).0 / 2 * CELL_SIZE + CELL_SIZE / 2,
                ((self.0).1 - 1) / 2 * CELL_SIZE + CELL_SIZE / 2),
        }
    }

    /// Direction towards the front of the wall.
    pub fn dir(&self) -> V2<i32> {
        match self.classify() {
            Vertical => V2(1, 0),
            Horizontal => V2(0, 1),
        }
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
        if rng.one_chance_in(24 - clamp(0, 12, depth as u32 / 2)) {
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

        assert_eq!(V2(CELL_SIZE / 2, -1 * CELL_SIZE + CELL_SIZE / 2), horizontal.pos());
        assert_eq!(V2(-1 * CELL_SIZE + CELL_SIZE / 2, CELL_SIZE / 2), vertical.pos());
    }
}
