use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow, WindowResolution};

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
        .add_event::<NewTileEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, (mouse_button_input,))
        .add_systems(Update, spawn_new_tile.after(mouse_button_input))
        .run();
}

fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // Add the MainCamera marker component.
    // FIXME: is this necessary?
    commands.spawn((Camera2d, MainCamera));
}

/// Used to help identify our main camera
// TODO: copied from https://bevy-cheatbook.github.io/cookbook/cursor2world.html
// not sure if this is necessary.
#[derive(Component)]
struct MainCamera;

/// User has requested placing a new tile.
#[derive(Event)]
pub struct NewTileEvent {
    position: Vec2,
}

fn mouse_button_input(
    mut event_writer: EventWriter<NewTileEvent>,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(cursor) = window.cursor_position() {
            let (camera, camera_transform) = q_camera.single().unwrap();
            let world_pos = camera
                .viewport_to_world_2d(camera_transform, cursor)
                .unwrap();

            debug!("left click, window coords {cursor} world coords {world_pos}",);
            event_writer.write(NewTileEvent {
                position: world_pos,
            });
        } else {
            panic!("left button, can't find cursor position")
        }
    }
}

fn spawn_new_tile(
    mut event_reader: EventReader<NewTileEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for new_tile_event in event_reader.read() {
        commands.spawn((
            Sprite::from_image(asset_server.load("temp1.png")),
            Transform::from_translation((new_tile_event.position, 0.0).into()),
        ));
    }
}
