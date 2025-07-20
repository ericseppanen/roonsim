//! Tools for handling the grid.
//!
//! The grid is based on units of 1/4 tile.
//!
//! Tiles can be placed with alignment 4 vertically, but 2 horizontally.
//! Additionally, there are some tiles that are always horizontally shifted
//! by 1 grid unit (1/4 tile).
//!
//! Marbles may be placed at locations defined by "marble sockets", that are
//! usually offset by 1 grid unit vertically from the tile edge, but sometimes
//! 2 (halfway between the tile top and bottom). Marble entrances and exits
//! are at even-numbered horizontal grid locations. This shifts to odd numbers
//! from the perspective of a tile that uses a 1-unit horizontal shift, so
//! that the world location is still even.

use std::fmt::Display;

use bevy::math::IVec2;
use bevy::prelude::*;

use crate::tile::Offset;

pub const GRID_UNITS_PER_TILE: i32 = 4;
pub const PIXELS_PER_GRID_UNIT: i32 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub struct GridPosition(pub IVec2);

impl GridPosition {
    /// Convert world coordinates to grid coordinates, snapping to legal tile positions.
    pub fn from_world_with_offset(pos: Vec2, offset: Offset) -> Self {
        let GridPosition(IVec2 { mut x, y }) = Self::from_world_snap_row(pos);
        match offset {
            Offset::Even => {
                if (x & 1) == 1 {
                    x -= 1;
                }
            }
            Offset::Odd => {
                if (x & 1) == 0 {
                    x += 1;
                }
            }
        }
        Self(IVec2::new(x, y))
    }

    /// Convert world coordinates to grid coordinates, rounding towards nearest `GridPosition`.
    pub fn from_world(pos: Vec2) -> Self {
        let x = (pos.x / (PIXELS_PER_GRID_UNIT as f32)).round() as i32;
        let y = (pos.y / (PIXELS_PER_GRID_UNIT as f32)).round() as i32;
        Self(IVec2::new(x, y))
    }

    /// Convert world coordinates to grid coordinates, snapping to a the most positive tile row.
    pub fn from_world_snap_row(pos: Vec2) -> Self {
        let x = (pos.x / (PIXELS_PER_GRID_UNIT as f32)).floor() as i32;
        let y = (pos.y / (PIXELS_PER_GRID_UNIT as f32)).floor() as i32;
        let y = (y / GRID_UNITS_PER_TILE) * GRID_UNITS_PER_TILE;
        Self(IVec2::new(x, y))
    }

    /// Convert grid coordinates to world coordinates.
    pub fn to_world(self) -> Vec2 {
        let x = (self.0.x * PIXELS_PER_GRID_UNIT) as f32;
        let y = (self.0.y * PIXELS_PER_GRID_UNIT) as f32;
        vec2(x, y)
    }

    // Give the absolute X and Y distance to another grid position.
    pub fn distance_to(self, other: GridPosition) -> UVec2 {
        let x_dist = self.0.x - other.0.x;
        let y_dist = self.0.y - other.0.y;
        let x_dist = x_dist.abs().try_into().unwrap();
        let y_dist = y_dist.abs().try_into().unwrap();
        uvec2(x_dist, y_dist)
    }
}

impl Display for GridPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}, {}>", self.0.x, self.0.y)
    }
}
