use bevy::{prelude::*, sprite::Anchor};

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

    pub fn load_sprite(&self, asset_server: &Res<AssetServer>) -> Sprite {
        let mut sprite = Sprite::from_image(asset_server.load(self.sprite_filename()));
        // This anchor is imperfect as the pointer is always a bit right of center,
        // but it's close enough for now.
        sprite.anchor = Anchor::BottomCenter;
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
}

/// Which offset (horizontal alignment) a tile has.
#[derive(Copy, Clone, Debug, Component, PartialEq)]
pub enum Offset {
    Even,
    Odd,
}
