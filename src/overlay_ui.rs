use crate::{loading::FontAssets, GameState};
use bevy::prelude::*;

#[derive(Component)]
struct OverlayUi;

#[derive(Component)]
pub struct OverlayUiBodyInfo;

#[derive(Default, PartialEq)]
pub enum ViewingBody {
    Moon(usize),
    Satellite(usize),
    #[default]
    None,
}

#[derive(Default, Resource)]
pub struct OverylayUiControls {
    pub viewing_body: ViewingBody,
}

pub struct OverlayUiPlugin;

/// This plugin is responsible for the ui shown during game play
impl Plugin for OverlayUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OverylayUiControls>();
        app.add_systems(OnEnter(GameState::Playing), setup_overlayui)
            .add_systems(OnExit(GameState::Playing), cleanup_overlayui);
    }
}

fn setup_overlayui(
    mut commands: Commands,
    font_handles: Res<FontAssets>,
) {
    // help info
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::FlexStart,
                    top: Val::Px(2.),
                    left: Val::Px(2.),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0., 0., 0., 0.9)),
                ..default()
            },
            OverlayUi,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Controls:\ntab - next viewing body",
                TextStyle {
                    font_size: 16.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ));
        });

    // info on current focused object
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceAround,
                    bottom: Val::Px(8.),
                    width: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
            OverlayUi,
        ))
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(8.)),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(0., 0., 0., 0.9)),
                    ..default()
                },
            )).with_children(|parent| {
                parent.spawn((TextBundle::from_section(
                    "",
                    TextStyle {
                        font: font_handles.fira.clone(),
                        font_size: 16.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ), OverlayUiBodyInfo));
            });
        });
}

fn cleanup_overlayui(mut commands: Commands, overlayui: Query<Entity, With<OverlayUi>>) {
    for entity in overlayui.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
