use bevy::{
    prelude::*,
    render::{camera::Viewport, view::RenderLayers},
};

use crate::tile::{ALL_TILES, Tile};

pub const UI_PANEL_HEIGHT: u32 = 64;

pub fn init_ui(asset_server: &AssetServer, commands: &mut Commands) {
    let viewport = Viewport {
        physical_position: UVec2::new(0, 0),
        physical_size: UVec2::new(600, UI_PANEL_HEIGHT),
        ..default()
    };
    let camera = Camera {
        viewport: Some(viewport),
        order: 1,
        ..default()
    };
    // Ensure we don't render the game sprites in this camera.
    let layers = RenderLayers::layer(2);
    let camera = commands.spawn((Camera2d, camera, layers)).id();

    // Set up UI
    commands
        .spawn((
            UiTargetCamera(camera),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(1.0),
                left: Val::Percent(1.0),
                width: Val::Percent(98.0),
                height: Val::Px(16.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Roons"),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(12.),
                    left: Val::Px(12.),
                    ..default()
                },
            ));
            buttons_panel(asset_server, parent);
        });
}

fn buttons_panel(asset_server: &AssetServer, parent: &mut ChildSpawnerCommands) {
    let bg_color = Color::srgb(0.5, 0.25, 0.25);
    let border_color = bg_color.darker(0.05);
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(0.5)),
                padding: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BorderColor(border_color),
            BackgroundColor(bg_color),
        ))
        .with_children(|parent| {
            for &tile in ALL_TILES {
                ui_tile_button(asset_server, parent, tile.name(), tile);
            }
        });
}

fn ui_tile_button(
    asset_server: &AssetServer,
    parent: &mut ChildSpawnerCommands,
    caption: &str,
    tile: Tile,
) {
    let image = asset_server.load(tile.sprite_filename());

    parent
        .spawn((
            UiPanelTile(tile),
            Button,
            Node {
                width: Val::Px(10.),
                height: Val::Px(10.),
                border: UiRect::all(Val::Px(0.5)),
                padding: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::WHITE),
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            ImageNode::new(image),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(caption),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 2.0,
                    ..default()
                },
            ));
        });
}

#[derive(Component)]
pub struct UiPanelTile(Tile);

pub fn button_system(
    interaction_query: Query<
        (&Interaction, &ComputedNodeTarget, &UiPanelTile),
        (Changed<Interaction>, With<Button>),
    >,
    mut event_writer: EventWriter<UiTileSelected>,
) {
    //info!("button_system");
    for (interaction, _computed_target, &UiPanelTile(tile)) in &interaction_query {
        if let Interaction::Pressed = *interaction {
            info!("enter tile spawning mode for {tile:?}");
            event_writer.write(UiTileSelected(tile));
        }
    }
}

/// User has selected a tile type for placement
#[derive(Event)]
pub struct UiTileSelected(pub Tile);
