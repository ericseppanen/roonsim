//! Tools for handling the grid.
//!
//! Tiles can be placed with alignment 1 vertically, but 1/2 horizontally.
//! Additionally, there are some tiles that are always horizontally shifted
//! by 1/4. So the horizontal granularity is 4x finer than the vertical.
//!
//! In code we allow tiles to have any of the 1/4 horizontal alignments, and
//! we expect to need extra tools to insure that each tile gets the correct
//! alignment for its type.
//!
//! We use integers to store the grid locations, with a vertical unit being
//! 4x longer than the horizontal unit.

use std::fmt::Display;

use bevy::math::IVec2;
use bevy::prelude::*;

/// quarter-granularity horizontal grid units
const HORIZONTAL_GRID_PIXELS: i32 = 4;
const VERTICAL_GRID_PIXELS: i32 = 16;

#[derive(Clone, Copy, Debug, Component)]
pub struct GridPosition(pub IVec2);

impl GridPosition {
    /// Convert world coordinates to grid coordinates.
    pub fn from_world(pos: Vec2) -> Self {
        // FIXME: f32 -> i32 clamps large values; I would rather it panic.
        let x = (pos.x / (HORIZONTAL_GRID_PIXELS as f32)).floor() as i32;
        let y = (pos.y / (VERTICAL_GRID_PIXELS as f32)).floor() as i32;
        Self(IVec2::new(x, y))
    }

    /// Convert grid coordinates to world coordinates.
    pub fn to_world(&self) -> Vec2 {
        // FIXME: i32 -> f32 is lossy at high values
        // Careful not to do integer division as the rounding
        // is wrong for negative numbers (and div_floor is unstable)
        let x = (self.0.x * HORIZONTAL_GRID_PIXELS) as f32;
        let y = (self.0.y * VERTICAL_GRID_PIXELS) as f32;
        vec2(x, y)
    }
}

impl Display for GridPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // For printing, we assume a square tile has size 1.0 .
        // We scale up the horizontal size so the units match.
        let x = self.0.x as f32 / 4.0;
        let y = self.0.y;
        write!(f, "<{x}, {y}>")
    }
}
