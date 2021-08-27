use bevy::prelude::*;
use bevy::render::camera::Camera;

use crate::{camera, player, GameState};

pub struct FollowTextEvent {
    pub entity: Entity,
    pub value: String,
    pub is_player: bool
}

#[derive(Default)]
pub struct FollowText {
    pub entity: Option::<Entity>,
    pub player_value: String,
    pub value: String,
    pub lock: f32,
}

pub struct FollowTextMarker;
pub struct FollowTextPlayerMarker;

pub fn handle_follow_text_event(
    mut follow_text: ResMut<FollowText>,
    mut follow_text_event_reader: EventReader<FollowTextEvent>
) {
    for event in follow_text_event_reader.iter() {
        if event.is_player {
            follow_text.player_value = event.value.clone();
        } else {
            follow_text.entity = Some(event.entity);
            follow_text.value = event.value.clone();
            follow_text.lock = 3.0;
        }
    }
}

pub fn update_follow_text(
    windows: Res<Windows>,
    mut follow_text: ResMut<FollowText>,
    players: Query<(&Transform, &player::Player)>,
    game_state: ResMut<GameState>,
    time: Res<Time>,
    mut player_text_query: Query<(&mut Style, &CalculatedSize, &mut Text,), (With<FollowTextPlayerMarker>, Without<FollowTextMarker>)>,
    mut text_query: Query<(&mut Style, &CalculatedSize, &mut Text,), (With<FollowTextMarker>, Without<FollowTextPlayerMarker>)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<camera::MainCamera>>,
    mesh_query: Query<&Transform, Without<player::Player>>,
) {
    follow_text.lock -= time.delta_seconds();
    for (camera, camera_transform) in camera_query.iter() {
        for (mut style, calculated, mut text) in player_text_query.iter_mut() {
            for (transform, player) in players.iter() {
                if player.kid == game_state.controlling {
                    text.sections[0].value = follow_text.player_value.clone();

                    match camera.world_to_screen(&windows, camera_transform, transform.translation)
                    {
                        Some(coords) => {
                            style.position.left = Val::Px(coords.x + 100.0 - calculated.size.width / 2.0);
                            style.position.bottom = Val::Px(coords.y - calculated.size.height / 2.0);
                        }
                        None => {
                            // A hack to hide the text when the mesh is behind the camera
                            style.position.bottom = Val::Px(-1000.0);
                        }
                    }
                }
            }
        }

        if follow_text.lock <= 0.0 {
            follow_text.value = "".to_string();
            follow_text.entity = None; 
            follow_text.lock = 0.0;
        }

        for (mut style, calculated, mut text) in text_query.iter_mut() {
            text.sections[0].value = follow_text.value.clone();

            if let Some(entity) = follow_text.entity {
                if let Ok(mesh) = mesh_query.get(entity) {
                    match camera.world_to_screen(&windows, camera_transform, mesh.translation) {
                        Some(coords) => {
                            style.position.left = Val::Px(coords.x + - calculated.size.width / 2.0);
                            style.position.bottom = Val::Px(coords.y - calculated.size.height / 2.0);
                        }
                        None => {
                            // A hack to hide the text when the mesh is behind the camera
                            style.position.bottom = Val::Px(-1000.0);
                        }
                    }
                }
            }
        }
    }
}

pub fn create_follow_text(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
) {
    // Set up UI labels for clarity
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                size: Size {
                    //width: Val::Px(200.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::with_section(
                "".to_string(),
                TextStyle {
                    font: font.clone(),
                    font_size: 50.0,
                    color: Color::WHITE,
                },
                TextAlignment {
                    ..Default::default()
                }
            ),
            ..Default::default()
        })
        .insert(FollowTextMarker);

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                size: Size {
                    //width: Val::Px(200.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::with_section(
                "".to_string(),
                TextStyle {
                    font: font.clone(),
                    font_size: 50.0,
                    color: Color::WHITE,
                },
                TextAlignment {
                    ..Default::default()
                }
            ),
            ..Default::default()
        })
        .insert(FollowTextPlayerMarker);
}
