use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::window::{PresentMode, PrimaryWindow, WindowResized, WindowResolution};
use grid::GridPosition;
use tile::{GridExtent, Offset, Tile};
use ui::{UI_PANEL_HEIGHT, UiTileSelected, button_system, init_ui};

mod grid;
mod tile;
mod ui;

const PRESENT_MODE: PresentMode = if cfg!(target_family = "wasm") {
    PresentMode::Fifo
} else {
    PresentMode::Mailbox
};

fn main() {
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
        .add_event::<UiTileSelected>()
        .add_systems(Startup, setup)
        .add_systems(Update, (button_system, on_resize_system))
        .add_systems(
            Update,
            (
                keyboard_inputs,
                placement_cursor_moved,
                mouse_button_input,
                change_ghost_tile,
            ),
        )
        .add_systems(Update, spawn_new_tile.after(mouse_button_input))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // FIXME: unify this code with the window resize code.
    let viewport = Viewport {
        physical_position: UVec2::new(0, UI_PANEL_HEIGHT),
        physical_size: UVec2::new(800, 700),
        ..default()
    };
    let camera = Camera {
        viewport: Some(viewport),
        ..default()
    };
    commands.spawn((Camera2d, camera, MainCamera));
    init_ui(&asset_server, &mut commands);
    spawn_ghost_tile(&mut commands, asset_server);
}

/// On window resize, recompute the camera viewport.
fn on_resize_system(
    mut resize_reader: EventReader<WindowResized>,
    mut camera: Single<&mut Camera, With<MainCamera>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    for event in resize_reader.read() {
        // Our window is 800x800 with scale factor 4.0
        // This event gives us 200x200 (the logical size, I think?)
        info!("window resize: {:.1} x {:.1}", event.width, event.height);

        let scale = window.scale_factor();
        let width = (event.width * scale) as u32;
        let height = (event.height * scale) as u32;
        let height = height.saturating_sub(UI_PANEL_HEIGHT);

        let viewport = camera.viewport.as_mut().unwrap();
        viewport.physical_size = UVec2::new(width, height);
    }
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

            let viewport_rect = camera.logical_viewport_rect().unwrap();
            if !viewport_rect.contains(cursor) {
                // click is outside viewport.
                // It seems a bit silly that viewport_to_world_2d doesn't
                // handle this.
                return;
            }

            let world_pos = camera
                .viewport_to_world_2d(camera_transform, cursor)
                .unwrap();

            let &offset = ghost.single().unwrap();
            let grid_pos = GridPosition::from_world(world_pos, offset);

            debug!("left click, window coords {cursor} world coords {world_pos}",);
            event_writer.write(NewTileEvent { position: grid_pos });
        }
    }
}

fn spawn_new_tile(
    mut event_reader: EventReader<NewTileEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ghost: Query<(&Sprite, &Tile), With<GhostTile>>,
    existing_tiles: Query<&GridExtent, (With<Tile>, Without<GhostTile>)>,
) {
    for new_tile_event in event_reader.read() {
        // Compute the world position of the new sprite.
        let grid_position = new_tile_event.position;
        let position = grid_position.to_world();
        let (ghost_sprite, &tile) = ghost.single_inner().unwrap();

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

        // Note z coordinate is > 0 so that it appears above the other tiles.
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
    mut ghost: Query<(&mut Sprite, &Tile), With<GhostTile>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut event_writer: EventWriter<UiTileSelected>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let (_, &tile) = ghost.single().unwrap();
        event_writer.write(UiTileSelected(tile.next()));
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        let (mut sprite, _) = ghost.single_mut().unwrap();
        sprite.flip_x = !sprite.flip_x;
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        let (mut sprite, _) = ghost.single_mut().unwrap();
        sprite.flip_y = !sprite.flip_y;
    }
}

fn change_ghost_tile(
    mut event_reader: EventReader<UiTileSelected>,
    mut ghost: Query<(&mut Sprite, &mut Tile, &mut Offset), With<GhostTile>>,
    asset_server: Res<AssetServer>,
) {
    // If multiple events were queued, we only care about the last one.
    let Some(&UiTileSelected(tile)) = event_reader.read().last() else {
        return;
    };

    let (mut sprite, mut ghost_tile, mut offset) = ghost.single_mut().unwrap();

    // Copy the previous `color` into the new sprite so we keep the alpha.
    let color = sprite.color;
    *sprite = tile.load_sprite(&asset_server);
    sprite.color = color;
    *offset = tile.offset();
    *ghost_tile = tile;
}
