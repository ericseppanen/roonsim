use bevy::{prelude::*, sprite::Anchor};

use crate::grid::GridPosition;

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

impl Tile {
    pub fn sprite_filename(&self) -> &'static str {
        match self {
            Tile::Canute => "canute.png",
            Tile::Shimmy => "shimmy.png",
            Tile::Switch => "switch.png",
            Tile::Turn => "turn.png",
            Tile::Distributor => "distributor.png",
            Tile::LongTurn => "long_turn.png",
            Tile::Path => "path.png",
            Tile::Swap => "swap.png",
            Tile::Trap => "trap.png",
            Tile::Xor => "xor.png",
        }
    }

    pub fn grid_width(&self) -> i32 {
        const HORIZONTAL_GRID_UNITS_PER_SQUARE: i32 = 4;
        let squares = match self {
            Tile::Path | Tile::Shimmy => 1,
            Tile::Canute | Tile::Swap | Tile::Switch | Tile::Turn | Tile::Xor => 2,
            Tile::Distributor | Tile::LongTurn | Tile::Trap => 3,
        };
        HORIZONTAL_GRID_UNITS_PER_SQUARE * squares
    }

    pub fn load_sprite(&self, asset_server: &Res<AssetServer>) -> Sprite {
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
    pub fn intersects(&self, other: &GridExtent) -> bool {
        info!("intersects? {self:?} -- {other:?}");

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
