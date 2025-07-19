use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::window::{PresentMode, PrimaryWindow, WindowResized, WindowResolution};
use place_marble::{
    DespawnGhostMarble, DespawnMarble, ShowMarbleSockets, despawn_ghost_marble,
    marble_placement_cursor_moved, mouseclick_place_marble, show_marble_sockets,
    spawn_ghost_marble,
};
use place_tile::{
    DespawnGhostTile, GhostTile, despawn_ghost_tile, mouseclick_delete_tile, mouseclick_place_tile,
    spawn_ghost_tile, tile_placement_cursor_moved,
};
use tile::Tile;
use ui::{
    UI_PANEL_HEIGHT, UiTileSelected, action_button_click, init_ui, marble_button_click,
    tile_button_click,
};

mod grid;
mod place_marble;
mod place_tile;
mod tile;
mod ui;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, States)]
enum SimState {
    #[default]
    Idle,
    /// Placing tiles.
    Placing,
    /// Deleting tiles.
    Deleting,
    /// Placing marbles.
    PlacingMarbles,
    /// Game is paused mid-simulation.
    Paused,
    /// Game simulation is running.
    Running,
}

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
        .add_event::<MouseClick>()
        .add_event::<UiTileSelected>()
        .add_event::<DespawnGhostTile>()
        .add_event::<DespawnMarble>()
        .add_event::<ShowMarbleSockets>()
        .init_state::<SimState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                tile_button_click,
                marble_button_click,
                action_button_click,
                on_resize_system,
                mouse_button_input,
            ),
        )
        .add_systems(
            Update,
            (
                placing_keyboard,
                tile_placement_cursor_moved,
                mouseclick_place_tile,
            )
                .run_if(in_state(SimState::Placing)),
        )
        .add_systems(
            Update,
            (marble_placement_cursor_moved, mouseclick_place_marble)
                .run_if(in_state(SimState::PlacingMarbles)),
        )
        .add_systems(
            Update,
            mouseclick_delete_tile.run_if(in_state(SimState::Deleting)),
        )
        .add_observer(spawn_ghost_tile)
        .add_observer(despawn_ghost_tile)
        .add_observer(show_marble_sockets)
        .add_observer(spawn_ghost_marble)
        .add_observer(despawn_ghost_marble)
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

#[derive(Clone, Copy, Debug, Event)]
struct MouseClick {
    world_pos: Vec2,
}

// Translate incoming mouse clicks into grid coordinates.
fn mouse_button_input(
    mut event_writer: EventWriter<MouseClick>,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
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

            debug!("left click, window coords {cursor} world coords {world_pos}",);
            event_writer.write(MouseClick { world_pos });
        }
    }
}

fn placing_keyboard(
    mut ghost: Query<(&mut Sprite, &Tile), With<GhostTile>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<SimState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        // FIXME: can I make an observer for "leaving tile placing mode"?
        commands.trigger(DespawnGhostTile);
        commands.trigger(DespawnGhostMarble);
        next_state.set(SimState::Idle);
        return;
    }
    if keyboard.just_pressed(KeyCode::Space) {
        let (_, &tile) = ghost.single().unwrap();
        commands.trigger(UiTileSelected(tile.next()));
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
