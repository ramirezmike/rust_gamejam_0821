use bevy::{prelude::*,};
use crate::{credits, game_controller };

pub struct LevelOverEvent {}
pub struct LevelOverText {} // TODO: change this to like "BetweenLevelEntity" or something marker or something

pub fn setup_level_over_screen(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    asset_server: Res<AssetServer>,
) {
    let window = windows.get_primary_mut().unwrap();
    let width = window.width(); 
    let height = window.height(); 

    commands.spawn_bundle(UiCameraBundle::default())
            .insert(LevelOverText {});

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(height * 0.35),
                    left: Val::Px(width * 0.25),
                    ..Default::default()
                },
                max_size: Size {
                    width: Val::Px(width / 2.0),
                    height: Val::Undefined,
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::with_section(
                "".to_string(),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 80.0,
                    color: Color::WHITE,
                },
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    vertical: VerticalAlign::Center,
                },
            ),
            ..Default::default()
        })
        .insert(LevelOverText {});
        println!("Level over text made!");
}

