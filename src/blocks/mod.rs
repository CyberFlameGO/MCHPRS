mod redstone;

use crate::items::{ActionResult, UseOnBlockContext};
use crate::plot::Plot;
use log::error;
use redstone::*;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct BlockPos {
    pub x: i32,
    pub y: u32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: u32, z: i32) -> BlockPos {
        BlockPos { x, y, z }
    }

    pub fn offset(&self, face: BlockFace) -> BlockPos {
        match face {
            BlockFace::Bottom => BlockPos::new(self.x, self.y.saturating_sub(1), self.z),
            BlockFace::Top => BlockPos::new(self.x, self.y + 1, self.z),
            BlockFace::North => BlockPos::new(self.x, self.y, self.z - 1),
            BlockFace::South => BlockPos::new(self.x, self.y, self.z + 1),
            BlockFace::West => BlockPos::new(self.x - 1, self.y, self.z),
            BlockFace::East => BlockPos::new(self.x + 1, self.y, self.z),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BlockDirection {
    North,
    South,
    East,
    West,
}

impl BlockDirection {
    fn opposite(self) -> BlockDirection {
        use BlockDirection::*;
        match self {
            North => South,
            South => North,
            East => West,
            West => East,
        }
    }

    pub fn block_face(self) -> BlockFace {
        use BlockDirection::*;
        match self {
            North => BlockFace::North,
            South => BlockFace::South,
            East => BlockFace::East,
            West => BlockFace::West,
        }
    }

    fn values() -> [BlockDirection; 4] {
        use BlockDirection::*;
        [North, South, East, West]
    }

    pub fn from_id(id: u32) -> BlockDirection {
        match id {
            0 => BlockDirection::North,
            1 => BlockDirection::South,
            2 => BlockDirection::West,
            3 => BlockDirection::East,
            _ => panic!("Invalid BlockDirection"),
        }
    }

    fn get_id(self) -> u32 {
        match self {
            BlockDirection::North => 0,
            BlockDirection::South => 1,
            BlockDirection::West => 2,
            BlockDirection::East => 3,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BlockFace {
    Bottom,
    Top,
    North,
    South,
    West,
    East,
}

impl BlockFace {
    pub fn from_id(id: u32) -> BlockFace {
        match id {
            0 => BlockFace::Bottom,
            1 => BlockFace::Top,
            2 => BlockFace::North,
            3 => BlockFace::South,
            4 => BlockFace::West,
            5 => BlockFace::East,
            _ => panic!("Invalid BlockFace"),
        }
    }
}

impl BlockFace {
    pub fn values() -> [BlockFace; 6] {
        use BlockFace::*;
        [Top, Bottom, North, South, East, West]
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Block {
    Air,
    RedstoneWire(RedstoneWire),
    RedstoneRepeater(RedstoneRepeater),
    RedstoneComparator(RedstoneComparator),
    RedstoneTorch(bool),
    RedstoneWallTorch(bool, BlockDirection),
    RedstoneLamp(bool),
    RedstoneBlock,
    Solid(u32),
    Transparent(u32),
}

impl Block {
    fn is_transparent(&self) -> bool {
        if let Block::Transparent(_) = self {
            true
        } else {
            false
        }
    }

    fn is_solid(&self) -> bool {
        match self {
            Block::RedstoneLamp(_) | Block::RedstoneBlock | Block::Solid(_) => true,
            _ => false,
        }
    }

    pub fn can_place_block_in(&self) -> bool {
        match self.get_id() {
            0 => true,           // Air
            9129..=9130 => true, // Void and Cave air
            34..=49 => true,     // Water
            50..=65 => true,     // Lava
            1341 => true,        // Grass
            1342 => true,        // Fern
            1343 => true,        // Dead bush
            1344 => true,        // Seagrass
            1345..=1346 => true, // Tall Seagrass
            7357..=7358 => true, // Tall Grass
            7359..=7360 => true, // Tall Fern
            _ => false,
        }
    }

    pub fn from_block_state(id: u32) -> Block {
        match id {
            0 => Block::Air,
            // Redstone Wire
            2056..=3351 => {
                let id = id - 2056;
                let west = RedstoneWireSide::from_id(id % 3);
                let south = RedstoneWireSide::from_id(id % 9 / 3);
                let power = id % 144 / 9;
                let north = RedstoneWireSide::from_id(id % 432 / 144);
                let east = RedstoneWireSide::from_id(id / 432);
                Block::RedstoneWire(RedstoneWire::new(north, south, east, west, power as u8))
            }
            // Redstone Torch
            3885 => Block::RedstoneTorch(true),
            3886 => Block::RedstoneTorch(false),
            // Redstone Wall Torch
            3887..=3894 => {
                let id = id - 3887;
                let lit = (id & 1) == 0;
                let facing = BlockDirection::from_id(id >> 1);
                Block::RedstoneWallTorch(lit, facing)
            }
            // Redstone Repeater
            4017..=4080 => {
                let id = id - 4017;
                let powered = (id & 1) == 0;
                let locked = ((id >> 1) & 1) == 0;
                let facing = BlockDirection::from_id((id >> 2) & 3);
                let delay = (id >> 4) as u8 + 1;
                Block::RedstoneRepeater(RedstoneRepeater::new(delay, facing, locked, powered))
            }
            // Redstone Lamp
            5140 => Block::RedstoneLamp(true),
            5141 => Block::RedstoneLamp(false),
            // Redstone Comparator
            6142..=6157 => {
                let id = id - 6142;
                let powered = (id & 1) == 0;
                let mode = ComparatorMode::from_id((id >> 1) & 1);
                let facing = BlockDirection::from_id(id >> 2);
                Block::RedstoneComparator(RedstoneComparator::new(facing, mode, powered))
            }
            6190 => Block::RedstoneBlock,
            _ => Block::Solid(id),
        }
    }

    pub fn get_id(self) -> u32 {
        match self {
            Block::Air => 0,
            Block::RedstoneWire(wire) => {
                wire.east.get_id() * 432
                    + wire.north.get_id() * 144
                    + wire.power as u32 * 9
                    + wire.south.get_id() * 3
                    + wire.west.get_id()
                    + 2056
            }
            Block::RedstoneTorch(true) => 3885,
            Block::RedstoneTorch(false) => 3886,
            Block::RedstoneWallTorch(lit, facing) => (facing.get_id() << 1) + (!lit as u32) + 3887,
            Block::RedstoneRepeater(repeater) => {
                (repeater.delay as u32 - 1) * 16
                    + repeater.facing.get_id() * 4
                    + !repeater.locked as u32 * 2
                    + !repeater.powered as u32
                    + 4017
            }
            Block::RedstoneLamp(true) => 5140,
            Block::RedstoneLamp(false) => 5141,
            Block::RedstoneComparator(comparator) => {
                comparator.facing.get_id() * 4
                    + comparator.mode.get_id() * 2
                    + !comparator.powered as u32
                    + 6142
            }
            Block::RedstoneBlock => 6190,
            Block::Solid(id) => id,
            Block::Transparent(id) => id,
        }
    }

    pub fn from_name(name: &str) -> Option<Block> {
        match name {
            "air" => Some(Block::Air),
            "glass" => Some(Block::Transparent(230)),
            "sandstone" => Some(Block::Solid(245)),
            "stone_bricks" => Some(Block::Solid(4481)),
            _ => None,
        }
    }

    pub fn on_use(&self, plot: &mut Plot, pos: &BlockPos) -> ActionResult {
        match self {
            Block::RedstoneRepeater(repeater) => {
                let mut repeater = repeater.clone();
                repeater.delay += 1;
                if repeater.delay > 4 {
                    repeater.delay -= 4;
                }
                plot.set_block(&pos, Block::RedstoneRepeater(repeater));
                ActionResult::Success
            }
            Block::RedstoneComparator(comparator) => {
                let mut comparator = comparator.clone();
                comparator.mode = comparator.mode.flip();
                plot.set_block(&pos, Block::RedstoneComparator(comparator));
                ActionResult::Success
            }
            _ => ActionResult::Pass,
        }
    }

    pub fn get_state_for_placement(
        plot: &Plot,
        pos: &BlockPos,
        item_id: u32,
        context: &UseOnBlockContext,
    ) -> Block {
        let block = match item_id {
            // Glass
            64 => Block::Transparent(230),
            // Sandstone
            68 => Block::Solid(245),
            // Wool
            82..=97 => Block::Solid(item_id + 1301),
            // Redstone Torch
            173 => match context.block_face {
                BlockFace::Top => Block::RedstoneTorch(true),
                BlockFace::Bottom => Block::RedstoneTorch(true),
                BlockFace::North => Block::RedstoneWallTorch(true, BlockDirection::North),
                BlockFace::South => Block::RedstoneWallTorch(true, BlockDirection::South),
                BlockFace::East => Block::RedstoneWallTorch(true, BlockDirection::East),
                BlockFace::West => Block::RedstoneWallTorch(true, BlockDirection::West),
            },
            // Concrete
            413..=428 => Block::Solid(item_id + 8489),
            // Redstone Repeater
            513 => Block::RedstoneRepeater(RedstoneRepeater {
                delay: 1,
                facing: context.player_direction.opposite(),
                locked: false,
                powered: false,
            }),
            // Redstone Comparator
            514 => Block::RedstoneComparator(RedstoneComparator {
                mode: ComparatorMode::Compare,
                facing: context.player_direction.opposite(),
                powered: false,
            }),
            // Redstone Wire
            600 => Block::RedstoneWire(RedstoneWire::get_state_for_placement(plot, pos)),
            _ => {
                error!("Tried to place block which wasnt a block!");
                Block::Solid(245)
            }
        };
        if block.is_valid_position(plot, pos) {
            block
        } else {
            Block::Air
        }
    }

    pub fn place_in_plot(self, plot: &mut Plot, pos: &BlockPos) {
        match self {
            Block::RedstoneRepeater(_) => {
                // TODO: Queue repeater tick
                plot.set_block(pos, self);
            }
            _ => {
                plot.set_block(pos, self);
            }
        }
        Block::change_surrounding_blocks(plot, pos);
        Block::update_surrounding_blocks(plot, pos);
    }

    pub fn destroy(self, plot: &mut Plot, pos: &BlockPos) {
        plot.set_block(&pos, Block::Air);
        Block::change_surrounding_blocks(plot, pos);
        Block::update_surrounding_blocks(plot, pos);
    }

    fn update(self, plot: &mut Plot, pos: &BlockPos) {
        match self {
            Block::RedstoneWire(wire) => {
                wire.on_neighbor_updated(plot, pos);
            }
            _ => {}
        }
    }

    pub fn is_cube(self) -> bool {
        match self {
            Block::Solid(_) | Block::Transparent(_) | Block::RedstoneBlock => true,
            _ => false,
        }
    }

    pub fn is_valid_position(self, plot: &Plot, pos: &BlockPos) -> bool {
        match self {
            Block::RedstoneWire(_)
            | Block::RedstoneComparator(_)
            | Block::RedstoneRepeater(_)
            | Block::RedstoneTorch(_) => {
                let bottom_block = plot.get_block(&pos.offset(BlockFace::Bottom));
                bottom_block.is_cube()
            }
            Block::RedstoneWallTorch(_, direction) => {
                let parent_block = plot.get_block(&pos.offset(direction.opposite().block_face()));
                parent_block.is_cube()
            }
            _ => true,
        }
    }

    fn change(self, plot: &mut Plot, pos: &BlockPos, direction: &BlockFace) {
        if !self.is_valid_position(plot, pos) {
            self.destroy(plot, pos);
            return;
        }
        match self {
            Block::RedstoneWire(wire) => {
                let new_state = wire.on_neighbor_changed(plot, pos, direction);
                plot.set_block(pos, Block::RedstoneWire(new_state));
            }
            _ => {}
        }
    }

    fn update_surrounding_blocks(plot: &mut Plot, pos: &BlockPos) {
        for direction in &BlockFace::values() {
            let neighbor_pos = &pos.offset(*direction);
            let block = plot.get_block(neighbor_pos);
            block.update(plot, neighbor_pos);
        }
    }

    fn change_surrounding_blocks(plot: &mut Plot, pos: &BlockPos) {
        for direction in &BlockFace::values() {
            let neighbor_pos = &pos.offset(*direction);
            let block = plot.get_block(neighbor_pos);
            block.change(plot, neighbor_pos, direction);

            // Also change diagonal blocks

            let up_pos = &neighbor_pos.offset(BlockFace::Top);
            let up_block = plot.get_block(&up_pos);
            up_block.change(plot, up_pos, direction);

            let down_pos = &neighbor_pos.offset(BlockFace::Bottom);
            let down_block = plot.get_block(&down_pos);
            down_block.change(plot, down_pos, direction);
        }
    }
}

#[test]
fn repeater_id_test() {
    let original =
        Block::RedstoneRepeater(RedstoneRepeater::new(3, BlockDirection::West, true, false));
    let id = original.get_id();
    assert_eq!(id, 4058);
    let new = Block::from_block_state(id);
    assert_eq!(new, original);
}

#[test]
fn comparator_id_test() {
    let original = Block::RedstoneComparator(RedstoneComparator::new(
        BlockDirection::West,
        ComparatorMode::Subtract,
        false,
    ));
    let id = original.get_id();
    assert_eq!(id, 6153);
    let new = Block::from_block_state(id);
    assert_eq!(new, original);
}