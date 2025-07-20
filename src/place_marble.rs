use bevy::prelude::*;

use crate::{
    MainCamera, MouseClick, SimState,
    grid::GridPosition,
    tile::{GridExtent, Marble, Tile},
};

pub struct MarblePlacePlugin;

impl Plugin for MarblePlacePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DespawnMarble>()
            .add_event::<ShowMarbleSockets>()
            .add_systems(
                Update,
                (marble_placement_cursor_moved, mouseclick_place_marble)
                    .run_if(in_state(SimState::PlacingMarbles)),
            )
            .add_systems(OnEnter(SimState::PlacingMarbles), spawn_ghost_marble)
            .add_systems(OnExit(SimState::PlacingMarbles), despawn_ghost_marble)
            .add_observer(show_marble_sockets);
    }
}

/// Handle the mouse movement during marble placement
pub fn marble_placement_cursor_moved(
    mut evr_cursor: EventReader<CursorMoved>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut ghost: Query<&mut Transform, With<GhostMarble>>,
) {
    for cursor_moved in evr_cursor.read() {
        let cursor = cursor_moved.position;
        let (camera, camera_transform) = q_camera.single().unwrap();
        let world_pos = camera
            .viewport_to_world_2d(camera_transform, cursor)
            .unwrap();

        let mut ghost_transform = ghost.single_mut().unwrap();

        let grid_pos = GridPosition::from_world_rounded(world_pos);

        let ghost_pos = grid_pos.to_world();
        let ghost_pos: Vec3 = ghost_pos.extend(0.0);

        ghost_transform.translation = ghost_pos;

        // TODO: draw an outline showing the grid position,
        // in the shape of the tile to be placed.

        //info!("New cursor position {cursor}, world coords {world_pos}, grid pos {grid_pos}");
    }
}

pub fn mouseclick_place_marble(
    mut event_reader: EventReader<MouseClick>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,

    _existing_marbles: Query<&GridExtent, (With<Marble>, Without<GhostMarble>)>,
) {
    for mouse_click in event_reader.read() {
        // Compute the world position of the new marble.
        let grid_pos = GridPosition::from_world_rounded(mouse_click.world_pos);
        let position = grid_pos.to_world();

        // FIXME: maybe click on an existing marble should delete it?

        // // Check if the new tile collides with any existing marbles.
        // for existing_extent in existing_marbles {
        //     if existing_extent.intersects(&new_tile_extent) {
        //         debug!("can't place tile due to collision");
        //         return;
        //     }
        // }

        debug!("spawn marble");

        // why -0.1 ? We need a bunch of constants for our Z heights.
        let position: Vec3 = (position, -0.1).into();

        let sprite = Marble::load_sprite(&asset_server);
        commands.spawn((sprite, Transform::from_translation(position), Marble));
    }
}

#[derive(Component)]
pub struct MarbleSocket;

/// Place MarbleSocket entities.
///
/// Marble sockets mark the places where it is legal to place marbles.
/// They are invisible (Disabled) unless we're in the marble placement
/// state.
pub fn place_marble_sockets(
    commands: &mut Commands,
    asset_server: &AssetServer,
    tile: Tile,
    extent: GridExtent,
    flip_x: bool,
    flip_y: bool,
) {
    // FIXME: needs a better name.
    let sprite = Sprite::from_image(asset_server.load("output.png"));

    // FIXME: this entity should be a child of the tile entity.
    for io_coord in tile.outputs() {
        let position = io_coord.to_world(extent, flip_x, flip_y);
        let position: Vec3 = (position, -0.5).into();
        commands.spawn((
            sprite.clone(),
            Transform::from_translation(position),
            MarbleSocket,
            // NOTE: bevy #18981 makes `Disabled` not work correctly if it's attached
            // to the entity at spawn time.
            Visibility::Hidden,
        ));
    }
}

#[derive(Event)]
pub struct ShowMarbleSockets(bool);

pub fn show_marble_sockets(
    trigger: Trigger<ShowMarbleSockets>,
    sockets: Query<&mut Visibility, With<MarbleSocket>>,
) {
    let ShowMarbleSockets(show) = *trigger;
    for mut socket_visibility in sockets {
        if show {
            *socket_visibility = Visibility::Visible;
        } else {
            *socket_visibility = Visibility::Hidden;
        }
    }
}

#[derive(Component)]
pub struct GhostMarble;

#[derive(Event)]
pub struct DespawnMarble;

pub fn spawn_ghost_marble(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Shouldn't be necessary; just being paranoid.
    commands.trigger(DespawnMarble);

    let mut sprite = Marble::load_sprite(&asset_server);
    // translucent tile to differentiate it from the already-placed tiles.
    sprite.color = Color::linear_rgba(1.0, 1.0, 1.0, 0.3);
    commands.spawn((
        sprite,
        // FIXME: the transform should be at the pointer location...
        Transform::default(),
        GhostMarble,
    ));

    // Show the marble sockets
    commands.trigger(ShowMarbleSockets(true));
}

pub fn despawn_ghost_marble(mut commands: Commands, mut ghost: Query<Entity, With<GhostMarble>>) {
    // Despawn the previous ghost marble, if any.
    if let Ok(ghost_entity) = ghost.single_mut() {
        commands.entity(ghost_entity).despawn();
    }
    // Hide the marble sockets
    commands.trigger(ShowMarbleSockets(false));
}
