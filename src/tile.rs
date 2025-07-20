use bevy::{prelude::*, sprite::Anchor};

use crate::grid::{GRID_UNITS_PER_TILE, GridPosition};

/// The coordinates of marble locations within a tile.
///
/// Inputs are places where marbles may enter from an adjacent tile. Outputs are
/// locations where marbles may exit the tile. Sticky points are places where marbles
/// may reside until perturbed by another marble.
#[expect(dead_code)]
struct Io {
    /// Places where marbles may enter.
    pub inputs: &'static [IoCoord],
    /// Places where marbles may leave.
    pub outputs: &'static [IoCoord],
    /// Places where marbles may stay put for a while.
    pub sticky: &'static [IoCoord],
}

/// The locations of inputs and outputs for a specific tile type.
#[derive(Copy, Clone, Debug)]
pub struct IoCoord {
    /// The X coordinate, in grid units.
    ///
    /// For a 1x1 tile, the allowed values are 1, 2, or 3. 0 and 4 are the corners,
    /// which is not allowed.
    x: u8,
    /// The Y coordinate is always 0 (bottom side), 1 (middle), or 2 (top side).
    y: MarbleY,
}

/// Marble Y locations.
///
/// As a `u8` these values are in grid unit, i.e.
/// `Bottom` is 25% from the bottom edge.
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum MarbleY {
    Bottom = 1,
    #[expect(dead_code)]
    Middle = 2,
    Top = 3,
}

impl MarbleY {
    /// Convert the marble Y location to a grid offset.
    fn to_grid(self) -> i32 {
        self as i32
    }
}

impl IoCoord {
    /// Create an `IoCoord` on the bottom edge of a tile.
    const fn bottom(x: u8) -> Self {
        Self {
            x,
            y: MarbleY::Bottom,
        }
    }

    /// Create an `IoCoord` on the top edge of a tile.
    const fn top(x: u8) -> Self {
        Self { x, y: MarbleY::Top }
    }

    /// Convert to world coordinates, given a tile location.
    ///
    /// These coordinates will be inside the tile such that a ball 1/2 the
    /// tile size will fit inside the tile perimeter.
    pub fn to_world(self, tile_pos: GridExtent, flip_x: bool, flip_y: bool) -> Vec2 {
        // first, compute the grid position for the tile origin, along with direction vectors
        // ( +1 or -1 ) that indicate which direction to move the Io positions.
        let mut x = tile_pos.origin.0.x;
        let mut y = tile_pos.origin.0.y;
        let mut x_direction = 1;
        let mut y_direction = 1;

        if flip_x {
            x += tile_pos.width;
            x_direction = -1;
        }
        if flip_y {
            y += 1;
            y_direction = -1;
        }

        // Add the offset for the IoCoord, so that the tile doesn't move when flipped.
        x += x_direction * i32::from(self.x);
        y += y_direction * self.y.to_grid();

        GridPosition(ivec2(x, y)).to_world()
    }
}

static CANUTE_IO: Io = Io {
    inputs: &[],
    outputs: &[IoCoord::bottom(2), IoCoord::top(4), IoCoord::top(6)],
    sticky: &[],
};

static SHIMMY_IO: Io = Io {
    inputs: &[],
    outputs: &[IoCoord::top(3)],
    sticky: &[],
};

static SWITCH_IO: Io = Io {
    inputs: &[],
    outputs: &[IoCoord::top(2), IoCoord::top(4), IoCoord::top(6)],
    sticky: &[],
};

static TURN_IO: Io = Io {
    inputs: &[],
    outputs: &[IoCoord::bottom(2), IoCoord::bottom(6)],
    sticky: &[],
};

static DISTRIBUTOR_IO: Io = Io {
    inputs: &[],
    outputs: &[IoCoord::top(2), IoCoord::top(6), IoCoord::top(10)],
    sticky: &[],
};

static LONG_TURN_IO: Io = Io {
    inputs: &[],
    outputs: &[IoCoord::bottom(2), IoCoord::bottom(6), IoCoord::bottom(10)],
    sticky: &[],
};

static PATH_IO: Io = Io {
    inputs: &[IoCoord::bottom(2)],
    outputs: &[IoCoord::top(2)],
    sticky: &[],
};

static SWAP_IO: Io = Io {
    inputs: &[],
    outputs: &[IoCoord::top(2), IoCoord::top(6)],
    sticky: &[],
};

static TRAP_IO: Io = Io {
    inputs: &[],
    outputs: &[IoCoord::top(2), IoCoord::top(6), IoCoord::top(8)],
    sticky: &[],
};

static XOR_IO: Io = Io {
    inputs: &[],
    outputs: &[IoCoord::top(2), IoCoord::top(4), IoCoord::top(6)],
    sticky: &[],
};

#[derive(Debug, Copy, Clone, Default, Component)]
pub enum Tile {
    Canute,
    Shimmy,
    Switch,
    Turn,
    Distributor,
    LongTurn,
    #[default]
    Path,
    Swap,
    Trap,
    Xor,
}

pub const ALL_TILES: &[Tile] = &[
    Tile::Canute,
    Tile::Shimmy,
    Tile::Switch,
    Tile::Turn,
    Tile::Distributor,
    Tile::LongTurn,
    Tile::Path,
    Tile::Swap,
    Tile::Trap,
    Tile::Xor,
];

impl Tile {
    pub fn name(&self) -> &'static str {
        match self {
            Tile::Canute => "canute",
            Tile::Shimmy => "shimmy",
            Tile::Switch => "switch",
            Tile::Turn => "turn",
            Tile::Distributor => "distributor",
            Tile::LongTurn => "long_turn",
            Tile::Path => "path",
            Tile::Swap => "swap",
            Tile::Trap => "trap",
            Tile::Xor => "xor",
        }
    }

    pub fn sprite_filename(&self) -> String {
        format!("{}.png", self.name())
    }

    pub fn grid_width(&self) -> i32 {
        let squares = match self {
            Tile::Path | Tile::Shimmy => 1,
            Tile::Canute | Tile::Swap | Tile::Switch | Tile::Turn | Tile::Xor => 2,
            Tile::Distributor | Tile::LongTurn | Tile::Trap => 3,
        };
        GRID_UNITS_PER_TILE * squares
    }

    pub fn load_sprite(&self, asset_server: &AssetServer) -> Sprite {
        let mut sprite = Sprite::from_image(asset_server.load(self.sprite_filename()));
        // This anchor is imperfect as the pointer is always a bit right of center,
        // but it's close enough for now.
        sprite.anchor = Anchor::BottomLeft;
        sprite
    }

    pub fn next(&self) -> Self {
        // FIXME: use proc macros for this.
        match self {
            Tile::Canute => Tile::Shimmy,
            Tile::Shimmy => Tile::Switch,
            Tile::Switch => Tile::Turn,
            Tile::Turn => Tile::Distributor,
            Tile::Distributor => Tile::LongTurn,
            Tile::LongTurn => Tile::Path,
            Tile::Path => Tile::Swap,
            Tile::Swap => Tile::Trap,
            Tile::Trap => Tile::Xor,
            Tile::Xor => Tile::Canute,
        }
    }

    // Check if this is an "even" tile (horizontal alignment 0.0 or 0.5)
    // or an "odd" tile (0.25 or 0.75)
    pub fn offset(&self) -> Offset {
        match self {
            Tile::Shimmy => Offset::Odd,
            _ => Offset::Even,
        }
    }

    pub fn extent(&self, origin: GridPosition) -> GridExtent {
        GridExtent {
            origin,
            width: self.grid_width(),
        }
    }

    /// Return a list of input coordinates for this tile.
    pub fn _inputs(&self) -> &'static [IoCoord] {
        todo!();
    }

    /// Return a list of output coordinates for this tile.
    pub fn outputs(&self) -> &'static [IoCoord] {
        self.io().outputs
    }

    /// Get access to the `Io` struct for this tile.
    fn io(&self) -> &'static Io {
        match self {
            Tile::Canute => &CANUTE_IO,
            Tile::Shimmy => &SHIMMY_IO,
            Tile::Switch => &SWITCH_IO,
            Tile::Turn => &TURN_IO,
            Tile::Distributor => &DISTRIBUTOR_IO,
            Tile::LongTurn => &LONG_TURN_IO,
            Tile::Path => &PATH_IO,
            Tile::Swap => &SWAP_IO,
            Tile::Trap => &TRAP_IO,
            Tile::Xor => &XOR_IO,
        }
    }
}

/// Which offset (horizontal alignment) a tile has.
#[derive(Copy, Clone, Debug, Component)]
pub enum Offset {
    Even,
    Odd,
}

/// The grid area covered by a tile.
#[derive(Copy, Clone, Debug, Component)]
pub struct GridExtent {
    origin: GridPosition,
    width: i32,
}

impl GridExtent {
    /// Check if this extent contains a grid position.
    pub fn contains(&self, world_pos: Vec2) -> bool {
        let grid_pos = GridPosition::from_world_snap_row(world_pos);

        // wrong row
        if self.origin.0.y != grid_pos.0.y {
            return false;
        }
        // position is left of extent.
        if grid_pos.0.x < self.origin.0.x {
            return false;
        }
        // position is right of extent.
        if grid_pos.0.x >= self.origin.0.x + self.width {
            return false;
        }
        // position is within extent.
        true
    }

    /// Check if this extent intersects another extent.
    pub fn intersects(&self, other: &GridExtent) -> bool {
        debug!("intersects? {self:?} -- {other:?}");

        // wrong row
        if self.origin.0.y != other.origin.0.y {
            return false;
        }
        // self is entirely left of other
        if self.origin.0.x + self.width <= other.origin.0.x {
            return false;
        }
        // other is entirely left of self
        if other.origin.0.x + other.width <= self.origin.0.x {
            return false;
        }
        // extents overlap
        true
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Marble;

impl Marble {
    pub fn sprite_filename() -> &'static str {
        "marble.png"
    }

    pub fn load_sprite(asset_server: &AssetServer) -> Sprite {
        let mut sprite = Sprite::from_image(asset_server.load(Self::sprite_filename()));
        sprite.anchor = Anchor::Center;
        sprite
    }
}
