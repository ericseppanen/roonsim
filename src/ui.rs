use bevy::{
    prelude::*,
    render::{camera::Viewport, view::RenderLayers},
};

use crate::{
    SimState,
    tile::{ALL_TILES, Marble, Tile},
};

pub const UI_PANEL_WIDTH: u32 = 780;
pub const UI_PANEL_HEIGHT: u32 = 64;

pub fn init_ui(asset_server: &AssetServer, commands: &mut Commands) {
    let viewport = Viewport {
        physical_position: UVec2::new(0, 0),
        physical_size: UVec2::new(UI_PANEL_WIDTH, UI_PANEL_HEIGHT),
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
            ui_marble_button(asset_server, parent);
            ui_action_button(asset_server, parent, "D", Action::Delete);
            ui_action_button(asset_server, parent, "<<", Action::Rewind);
            ui_action_button(asset_server, parent, ">", Action::Play);
            ui_action_button(asset_server, parent, "||", Action::Pause);
        });
}

#[derive(Copy, Clone, Debug, Component)]
pub enum Action {
    Delete,
    Rewind,
    Play,
    Pause,
}

fn ui_action_button(
    asset_server: &AssetServer,
    parent: &mut ChildSpawnerCommands,
    caption: &str,
    action: Action,
) {
    parent
        .spawn((
            action,
            Button,
            Node {
                width: Val::Px(10.),
                height: Val::Px(10.),
                border: UiRect::all(Val::Px(0.5)),
                padding: UiRect::all(Val::Px(1.0)),
                margin: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::WHITE),
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            //ImageNode::new(image),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(caption),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 3.0,
                    ..default()
                },
            ));
        });
}

/// Create a UI tile button.
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
                margin: UiRect::all(Val::Px(1.0)),
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

/// Create the UI marble button.
fn ui_marble_button(asset_server: &AssetServer, parent: &mut ChildSpawnerCommands) {
    let image = asset_server.load(Marble::sprite_filename());

    parent
        .spawn((
            UiPanelMarble,
            Button,
            Node {
                width: Val::Px(10.),
                height: Val::Px(10.),
                border: UiRect::all(Val::Px(0.5)),
                padding: UiRect::all(Val::Px(1.0)),
                margin: UiRect::all(Val::Px(1.0)),
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
                Text::new("marble"),
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

#[derive(Component)]
pub struct UiPanelMarble;

#[expect(clippy::type_complexity)]
pub fn tile_button_click(
    interaction_query: Query<
        (&Interaction, &ComputedNodeTarget, &UiPanelTile),
        (Changed<Interaction>, With<Button>),
    >,
    mut commands: Commands,
) {
    for (interaction, _computed_target, &UiPanelTile(tile)) in &interaction_query {
        if let Interaction::Pressed = *interaction {
            info!("enter tile spawning mode for {tile:?}");
            commands.trigger(UiTileSelected(tile));
        }
    }
}

#[expect(clippy::type_complexity)]
pub fn marble_button_click(
    interaction_query: Query<
        (&Interaction, &ComputedNodeTarget, &UiPanelMarble),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<SimState>>,
) {
    for (interaction, _computed_target, _) in &interaction_query {
        if let Interaction::Pressed = *interaction {
            info!("enter marble placing mode");
            next_state.set(SimState::PlacingMarbles);
        }
    }
}

#[expect(clippy::type_complexity)]
pub fn action_button_click(
    interaction_query: Query<
        (&Interaction, &ComputedNodeTarget, &Action),
        (Changed<Interaction>, With<Button>),
    >,
    mut _commands: Commands,
    mut next_state: ResMut<NextState<SimState>>,
) {
    for (interaction, _computed_target, &action) in &interaction_query {
        if let Interaction::Pressed = *interaction {
            info!("action button: {action:?}");
            let state = match action {
                Action::Delete => SimState::Deleting,
                Action::Rewind => SimState::Idle, // FIXME: needs work
                Action::Play => SimState::Running,
                Action::Pause => SimState::Paused,
            };
            next_state.set(state);
        }
    }
}

/// User has selected a tile type for placement
#[derive(Event)]
pub struct UiTileSelected(pub Tile);
