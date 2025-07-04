use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow, WindowResolution};
use grid::GridPosition;
use tile::{Offset, Tile};

mod grid;
mod tile;

const PRESENT_MODE: PresentMode = if cfg!(target_family = "wasm") {
    PresentMode::Fifo
} else {
    PresentMode::Mailbox
};

fn main() {
    bevy::log::info!("hello world");
    App::new()
        .add_plugins(
            DefaultPlugins
                // // Prevent asset .meta loading errors on web.
                // .set(AssetPlugin {
                //     meta_check: AssetMetaCheck::Never,
                //     ..default()
                // })
                // default_nearest() prevents blurring of pixel art
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        #[cfg(target_family = "wasm")]
                        canvas: Some("#roonsim-canvas".into()),
                        title: "Roon Simulator".into(),
                        resolution: WindowResolution::new(800.0, 800.0)
                            .with_scale_factor_override(4.0),
                        present_mode: PRESENT_MODE,
                        resizable: true,

                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .insert_resource(ClearColor(Color::srgb(0.3, 0.3, 0.3)))
        .add_event::<NewTileEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (keyboard_inputs, placement_cursor_moved, mouse_button_input),
        )
        .add_systems(Update, spawn_new_tile.after(mouse_button_input))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Add the MainCamera marker component.
    // FIXME: is this necessary?
    commands.spawn((Camera2d, MainCamera));
    spawn_ghost_tile(&mut commands, asset_server);
}

/// Used to help identify our main camera
// TODO: copied from https://bevy-cheatbook.github.io/cookbook/cursor2world.html
// not sure if this is necessary.
#[derive(Component)]
struct MainCamera;

/// User has requested placing a new tile.
#[derive(Event)]
pub struct NewTileEvent {
    position: GridPosition,
}

#[derive(Component)]
struct GhostTile;

/// Handle the mouse movement during tile placement
fn placement_cursor_moved(
    mut evr_cursor: EventReader<CursorMoved>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut ghost: Query<(&mut Transform, &mut Visibility, &Offset), With<GhostTile>>,
) {
    for cursor_moved in evr_cursor.read() {
        let cursor = cursor_moved.position;
        let (camera, camera_transform) = q_camera.single().unwrap();
        let world_pos = camera
            .viewport_to_world_2d(camera_transform, cursor)
            .unwrap();

        let (mut ghost_transform, mut ghost_visibility, &offset) = ghost.single_mut().unwrap();

        let grid_pos = GridPosition::from_world(world_pos, offset);

        let ghost_pos = grid_pos.to_world();
        let ghost_pos: Vec3 = ghost_pos.extend(0.0);

        ghost_transform.translation = ghost_pos;
        *ghost_visibility = Visibility::Visible;

        // TODO: draw an outline showing the grid position,
        // in the shape of the tile to be placed.

        //info!("New cursor position {cursor}, world coords {world_pos}, grid pos {grid_pos}");
    }
}

fn mouse_button_input(
    mut event_writer: EventWriter<NewTileEvent>,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    ghost: Query<&Offset, With<GhostTile>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(cursor) = window.cursor_position() {
            let (camera, camera_transform) = q_camera.single().unwrap();
            let world_pos = camera
                .viewport_to_world_2d(camera_transform, cursor)
                .unwrap();

            let &offset = ghost.single().unwrap();

            let grid_pos = GridPosition::from_world(world_pos, offset);

            info!("left click, window coords {cursor} world coords {world_pos}",);
            event_writer.write(NewTileEvent { position: grid_pos });
        } else {
            panic!("left button, can't find cursor position")
        }
    }
}

fn spawn_new_tile(
    mut event_reader: EventReader<NewTileEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ghost: Query<(&Sprite, &Tile), With<GhostTile>>,
) {
    for new_tile_event in event_reader.read() {
        let grid_position = new_tile_event.position;
        let position = grid_position.to_world();
        // Note z coordinate is > 0 so that it appears above the other tiles.
        let position: Vec3 = (position, -1.0).into();

        let (ghost_sprite, tile) = ghost.single_inner().unwrap();
        let mut sprite = tile.load_sprite(&asset_server);
        sprite.flip_x = ghost_sprite.flip_x;
        sprite.flip_y = ghost_sprite.flip_y;
        commands.spawn((sprite, Transform::from_translation(position), grid_position));
    }
}

/// Create a translucent tile showing where the next tile will be placed.
///
/// The sprite will have a `GhostTile` marker component.
///
/// This only needs to be done once.
fn spawn_ghost_tile(commands: &mut Commands, asset_server: Res<AssetServer>) {
    let tile = Tile::default();
    let offset = tile.offset();
    let mut sprite = tile.load_sprite(&asset_server);
    sprite.color = Color::linear_rgba(1.0, 1.0, 1.0, 0.3);
    commands.spawn((
        sprite,
        Transform::default(),
        Visibility::Hidden,
        tile,
        offset,
        GhostTile,
    ));
}

fn keyboard_inputs(
    mut ghost: Query<(&mut Sprite, &mut Tile, &mut Offset), With<GhostTile>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        // toggle the ghost image
        // FIXME: this is getting a little ridiculous; Maybe I should just despawn + respawn.
        let (mut sprite, mut tile, mut offset) = ghost.single_mut().unwrap();

        // Advance the ghost tile to the next tile type.
        // Copy the previous `color` into the new sprite so we keep the alpha.
        let next_tile = tile.next();
        let color = sprite.color;
        *sprite = next_tile.load_sprite(&asset_server);
        sprite.color = color;
        *offset = next_tile.offset();
        *tile = next_tile;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        let (mut sprite, _, _) = ghost.single_mut().unwrap();
        sprite.flip_x = !sprite.flip_x;
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        let (mut sprite, _, _) = ghost.single_mut().unwrap();
        sprite.flip_y = !sprite.flip_y;
    }
}
