use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::window::{PresentMode, PrimaryWindow, WindowResized, WindowResolution};
use place_marble::MarblePlacePlugin;
use place_tile::TilePlacePlugin;
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
        .add_plugins((TilePlacePlugin, MarblePlacePlugin))
        .insert_resource(ClearColor(Color::srgb(0.3, 0.3, 0.3)))
        .add_event::<MouseClick>()
        .add_event::<UiTileSelected>()
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
