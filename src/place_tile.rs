use bevy::prelude::*;

use crate::{
    MainCamera, MouseClick, SimState,
    grid::GridPosition,
    place_marble::place_marble_sockets,
    tile::{GridExtent, Offset, Tile},
    ui::UiTileSelected,
};

#[expect(clippy::type_complexity)]
pub fn mouseclick_delete_tile(
    mut event_reader: EventReader<MouseClick>,
    existing_tiles: Query<(Entity, &GridExtent), (With<Tile>, Without<GhostTile>)>,
    mut commands: Commands,
) {
    for mouse_click in event_reader.read() {
        // Search for a tile that intersects the click position.
        for (entity, extent) in existing_tiles {
            if extent.contains(mouse_click.world_pos) {
                debug!("deleting tile");
                commands.entity(entity).despawn();
                break;
            }
        }
    }
}

pub fn mouseclick_place_tile(
    mut event_reader: EventReader<MouseClick>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ghost: Query<(&Sprite, &Tile, &Offset), With<GhostTile>>,

    existing_tiles: Query<&GridExtent, (With<Tile>, Without<GhostTile>)>,
) {
    for mouse_click in event_reader.read() {
        // Compute the world position of the new sprite.
        let (ghost_sprite, &tile, &offset) = ghost.single_inner().unwrap();
        let grid_position = GridPosition::from_world_with_offset(mouse_click.world_pos, offset);
        let position = grid_position.to_world();

        // Compute the extent of the tile (its width in grid coordinates)
        let new_tile_extent = tile.extent(grid_position);

        // Check if the new tile collides with any existing tiles.
        for existing_extent in existing_tiles {
            if existing_extent.intersects(&new_tile_extent) {
                debug!("can't place tile due to collision");
                return;
            }
        }

        info!("spawn {tile:?}");

        // why -1.0 ?
        let position: Vec3 = (position, -1.0).into();

        let mut sprite = tile.load_sprite(&asset_server);
        sprite.flip_x = ghost_sprite.flip_x;
        sprite.flip_y = ghost_sprite.flip_y;
        commands.spawn((
            sprite,
            Transform::from_translation(position),
            tile,
            new_tile_extent,
        ));

        place_marble_sockets(
            &mut commands,
            &asset_server,
            tile,
            new_tile_extent,
            ghost_sprite.flip_x,
            ghost_sprite.flip_y,
        );
    }
}

#[derive(Component)]
pub struct GhostTile;

/// Handle the mouse movement during tile placement
pub fn tile_placement_cursor_moved(
    mut evr_cursor: EventReader<CursorMoved>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut ghost: Query<(&mut Transform, &Offset), With<GhostTile>>,
) {
    for cursor_moved in evr_cursor.read() {
        let cursor = cursor_moved.position;
        let (camera, camera_transform) = q_camera.single().unwrap();
        let world_pos = camera
            .viewport_to_world_2d(camera_transform, cursor)
            .unwrap();

        let (mut ghost_transform, &offset) = ghost.single_mut().unwrap();

        let grid_pos = GridPosition::from_world_with_offset(world_pos, offset);

        let ghost_pos = grid_pos.to_world();
        let ghost_pos: Vec3 = ghost_pos.extend(0.0);

        ghost_transform.translation = ghost_pos;

        // TODO: draw an outline showing the grid position,
        // in the shape of the tile to be placed.

        //info!("New cursor position {cursor}, world coords {world_pos}, grid pos {grid_pos}");
    }
}

pub fn spawn_ghost_tile(
    trigger: Trigger<UiTileSelected>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<SimState>>,
) {
    commands.trigger(DespawnGhostTile);

    let UiTileSelected(tile) = *trigger;

    let mut sprite = tile.load_sprite(&asset_server);
    let offset = tile.offset();
    // translucent tile to differentiate it from the already-placed tiles.
    sprite.color = Color::linear_rgba(1.0, 1.0, 1.0, 0.3);
    commands.spawn((
        sprite,
        // FIXME: the transform should be at the pointer location...
        Transform::default(),
        tile,
        offset,
        GhostTile,
    ));

    next_state.set(SimState::Placing);
}

#[derive(Event)]
pub struct DespawnGhostTile;

pub fn despawn_ghost_tile(
    _trigger: Trigger<DespawnGhostTile>,
    mut commands: Commands,
    mut ghost: Query<Entity, With<GhostTile>>,
) {
    // Despawn the previous ghost tile, if any.
    if let Ok(ghost_entity) = ghost.single_mut() {
        commands.entity(ghost_entity).despawn();
    }
}
