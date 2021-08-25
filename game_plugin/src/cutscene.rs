use bevy::prelude::*;
use serde::Deserialize;
use bevy::reflect::{TypeUuid};
use crate::{player,asset_loader,AppState, game_controller, camera};

pub struct CutsceneEvent {
}
pub struct CharacterDisplayEvent {
    character_and_position: (Character, Position),
}

pub struct DebugCharacterMarker;
pub struct SpeechBox;
pub struct CutsceneTrashMarker;
pub struct SpeechText;
pub struct SpeechBoxEvent {
    text: Option::<String> // hide if empty
}

pub struct DebugCutsceneTriggerMesh;
pub struct CurrentCutscene {
    segment_index: usize,
    waiting: Option::<CutsceneWait>,
    cutscene: Option::<Cutscene>,
}

pub enum CutsceneWait {
    Time(f32),
    Interaction,
}

pub struct CutscenePlugin;
impl Plugin for CutscenePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
           .insert_resource(CurrentCutscene { 
               segment_index: 0,
               waiting: None,
               cutscene: None 
           })
           .add_event::<SpeechBoxEvent>()
           .add_event::<CharacterDisplayEvent>()
           .add_system_set(
               SystemSet::on_update(crate::AppState::InGame)
                   .with_system(check_for_cutscene.system())
                   .with_system(debug_draw_cutscene_triggers.system())

           )
           .add_system_set(
               SystemSet::on_enter(crate::AppState::Cutscene)
                   .with_system(setup_cutscene.system())
           )
           .add_system_set(
               SystemSet::on_exit(crate::AppState::Cutscene)
                   .with_system(cleanup_cutscene.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::Cutscene)
                   .with_system(handle_speechbox_event.system())
                   .with_system(handle_character_display_event.system())
                   .with_system(debug_move_character.system())
                   .with_system(update_cutscene.system())

           );
    }
}

pub fn update_cutscene(
    mut current_cutscene: ResMut<CurrentCutscene>,
    mut state: ResMut<State<AppState>>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    gamepad: Option<Res<game_controller::GameController>>,

    mut speechbox_event_writer: EventWriter<SpeechBoxEvent>, 
    mut character_display_event_writer: EventWriter<CharacterDisplayEvent>, 
    mut action_buffer: Local<Option::<u128>>,
) {
    let time_buffer = 100;
    let time_since_startup = time.time_since_startup().as_millis();
    if let Some(time_since_action) = *action_buffer {
        if time_since_startup - time_since_action > time_buffer {
            *action_buffer = None;
        }
    }

    match &mut current_cutscene.waiting {
        Some(waiting) => {
            match waiting {
                CutsceneWait::Time(wait_time) => {
                    *wait_time -=  time.delta_seconds();
                    if *wait_time < 0.0 {
                        current_cutscene.segment_index += 1;
                        current_cutscene.waiting = None;
                    } else {
                        return;
                    }
                },
                CutsceneWait::Interaction => {
                    let pressed_buttons = game_controller::get_pressed_buttons(&axes, &buttons, gamepad);

                    if (keyboard_input.just_pressed(KeyCode::Space) 
                    || keyboard_input.just_pressed(KeyCode::Return) 
                    || keyboard_input.just_pressed(KeyCode::J) 
                    || pressed_buttons.contains(&game_controller::GameButton::Action))
                    && action_buffer.is_none() {
                        *action_buffer = Some(time.time_since_startup().as_millis());

                        current_cutscene.segment_index += 1;
                        current_cutscene.waiting = None;
                    } else {
                        return;
                    }
                }
            }
        },
        None => ()
    }

    let current_index = current_cutscene.segment_index;
    println!("Current index: {}", current_index);
    match &mut current_cutscene.cutscene {
        Some(cutscene) =>  {
            if let Some(segment) = cutscene.segments.get(current_index) {
                match segment {
                    CutsceneSegment::Debug(text) => {
                        println!("{}", text);
                        current_cutscene.waiting = Some(CutsceneWait::Time(2.0));
                    },
                    CutsceneSegment::Textbox(text) => {
                        speechbox_event_writer.send(SpeechBoxEvent { text: Some(text.to_string()) });
                        current_cutscene.waiting = Some(CutsceneWait::Interaction);
                    },
                    CutsceneSegment::CharacterPosition(character, position) => {
                        character_display_event_writer.send(CharacterDisplayEvent {
                            character_and_position: (character.clone(), position.clone())
                        });
                        current_cutscene.waiting = Some(CutsceneWait::Time(0.0));
                    }
                    _ => ()
                }
            } else {
                println!("Cutscene is over!");
                // cutscene must be over
                current_cutscene.cutscene = None;
            }
        },
        None => {
            current_cutscene.segment_index = 0;
            state.pop().unwrap();
        }
    }
}

static SCALE: f32 = 0.35;
static LEFT_X: f32 = -0.9;
static LEFT_Y: f32 = -0.5;
static LEFT_Z: f32 = -2.1900003;
static LEFT_ROTATION_X: f32 = -0.07948004;
static LEFT_ROTATION_Y: f32 = 0.94565785;
static LEFT_ROTATION_Z: f32 = 0.315322;
static LEFT_ROTATION_ANGLE: f32 = 5.599498;

pub fn handle_character_display_event(
    mut character_display_event_reader: EventReader<CharacterDisplayEvent>, 
    mut commands: Commands, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    person_meshes: Res<player::PersonMeshes>,
    main_camera: Query<Entity, With<camera::MainCamera>>,
) {
    for camera_entity in main_camera.iter() {
        for event in character_display_event_reader.iter() {
            let color = Color::hex("FCF300").unwrap(); 

            match event.character_and_position {
                (Character::Dude, Position::Left) => {
                    let mut transform = Transform::from_xyz(LEFT_X, LEFT_Y, LEFT_Z);
                    transform.apply_non_uniform_scale(Vec3::new(SCALE, SCALE, SCALE)); 
                    transform.rotate(Quat::from_axis_angle(Vec3::new(LEFT_ROTATION_X, LEFT_ROTATION_Y, LEFT_ROTATION_Z), LEFT_ROTATION_ANGLE));
                    let inner_mesh_vertical_offset = 1.0;
                    let character_entity = 
                        commands.spawn_bundle(PbrBundle {
                                    transform,
                                    ..Default::default()
                                })
                                .insert(CutsceneTrashMarker)
                                .insert(DebugCharacterMarker)
                                .with_children(|parent|  {
                                    parent.spawn_bundle(PbrBundle {
                                        mesh: person_meshes.person.clone(),
                                        material: materials.add(color.into()),
                                        transform: Transform::from_xyz(0.0, 0.5, 0.0),
                                        ..Default::default()
                                    });
                                }).id();
                            
                    commands.entity(camera_entity).push_children(&[character_entity]);
                },
                _ => ()
            }
        }
    }
}

pub fn debug_move_character(
    keyboard_input: Res<Input<KeyCode>>,
    mut characters: Query<&mut Transform, With<DebugCharacterMarker>>,
    time: Res<Time>,
) {
    for mut transform in characters.iter_mut() {
        let mut print = false;
        if keyboard_input.pressed(KeyCode::Left) {
            transform.translation.z -= 0.1; 
            print = true; 
        }
        if keyboard_input.pressed(KeyCode::Right) {
            transform.translation.z += 0.1; 
            print = true; 
        }
        if keyboard_input.pressed(KeyCode::Up) {
            transform.translation.x += 0.1; 
            print = true; 
        }
        if keyboard_input.pressed(KeyCode::Down) {
            transform.translation.x -= 0.1; 
            print = true; 
        }
        if keyboard_input.pressed(KeyCode::LShift) {
            transform.translation.y -= 0.1; 
            print = true; 
        }
        if keyboard_input.pressed(KeyCode::Space) {
            transform.translation.y += 0.1; 
            print = true; 
        }
        if keyboard_input.pressed(KeyCode::A) {
            transform.rotate(Quat::from_rotation_y(-time.delta_seconds()));
            print = true; 
        }
        if keyboard_input.pressed(KeyCode::D) {
            transform.rotate(Quat::from_rotation_y(time.delta_seconds()));
            print = true; 
        }
        if keyboard_input.pressed(KeyCode::W) {
            transform.rotate(Quat::from_rotation_z(time.delta_seconds()));
            print = true; 
        }
        if keyboard_input.pressed(KeyCode::S) {
            transform.rotate(Quat::from_rotation_z(-time.delta_seconds()));
            print = true; 
        }
        if keyboard_input.pressed(KeyCode::E) {
            transform.rotate(Quat::from_rotation_x(time.delta_seconds()));
            print = true; 
        }
        if keyboard_input.pressed(KeyCode::Q) {
            transform.rotate(Quat::from_rotation_x(-time.delta_seconds()));
            print = true; 
        }

        if print {
            let translation = transform.translation;
            let (rotation, axis) = transform.rotation.to_axis_angle();
            println!("camera_x: {:?},", translation.x); 
            println!("camera_y: {:?},", translation.y); 
            println!("camera_z: {:?},", translation.z); 
            println!("camera_rotation_x: {:?},", rotation.x); 
            println!("camera_rotation_y: {:?},", rotation.y); 
            println!("camera_rotation_z: {:?},", rotation.z); 
            println!("camera_rotation_angle: {:?},", axis); 
        }
    }
}

pub fn handle_speechbox_event(
    mut speechbox_event_reader: EventReader<SpeechBoxEvent>, 
    mut textbox_visibility: Query<&mut Visible, With<SpeechBox>>,
    mut textbox_text: Query<&mut Text, With<SpeechText>>,
) {
    for event in speechbox_event_reader.iter() {
        if let Some(text_to_display) = &event.text {
            println!("Got event to show textbox");
            for mut textbox_text in textbox_text.iter_mut() {
                textbox_text.sections[0].value = text_to_display.to_string();
            }
            for mut visibility in textbox_visibility.iter_mut() {
                visibility.is_visible = true;
            }
        } else {
            for mut textbox_text in textbox_text.iter_mut() {
                textbox_text.sections[0].value = "".to_string();
            }
            for mut visibility in textbox_visibility.iter_mut() {
                visibility.is_visible = false;
            }
        }
    }
}

pub fn cleanup_cutscene(
    trash: Query<Entity, With<CutsceneTrashMarker>>,
    mut commands: Commands,
) {
    for entity in trash.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn check_for_cutscene(
    player: Query<&Transform, With<player::Player>>,
    level_info_state: Res<asset_loader::LevelInfoState>, 
    mut level_info_assets: ResMut<Assets<asset_loader::LevelInfo>>,
    mut state: ResMut<State<AppState>>,
    mut current_cutscene: ResMut<CurrentCutscene>,
) {
    if let Some(levels_asset) = level_info_assets.get_mut(&level_info_state.handle) {
        for player in player.iter() {
            let player_position = Vec2::new(player.translation.x, player.translation.z);
            for mut cutscene in levels_asset.cutscenes.cutscenes.iter_mut() {
                if cutscene.has_been_triggered { continue; }

                let (location, distance) = cutscene.location;
                if player_position.distance(location) < distance {
                    println!("Cutscene hit");
                    cutscene.has_been_triggered = true;
                    current_cutscene.cutscene = Some(cutscene.clone());
                    current_cutscene.segment_index = 0;
                    current_cutscene.waiting = None;
                    state.push(AppState::Cutscene).unwrap();
                }
            }
        }
    }
}

pub fn setup_cutscene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(UiCameraBundle::default())
            .insert(CutsceneTrashMarker);

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(30.0)),
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            visible: Visible {
                is_visible: false,
                is_transparent: false,
            },
            ..Default::default()
        })
        .insert(SpeechBox)
        .insert(CutsceneTrashMarker)
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        border: Rect::all(Val::Px(2.0)),
                        ..Default::default()
                    },
                    visible: Visible {
                        is_visible: false,
                        is_transparent: false,
                    },
                    material: materials.add(Color::rgb(0.65, 0.65, 0.65).into()),
                    ..Default::default()
                })
                .insert(SpeechBox)
                .with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                align_items: AlignItems::FlexEnd,
                                ..Default::default()
                            },
                            material: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
                            visible: Visible {
                                is_visible: false,
                                is_transparent: false,
                            },
                            ..Default::default()
                        })
                        .insert(SpeechBox)
                        .with_children(|parent| {
                            parent.spawn_bundle(TextBundle {
                                style: Style {
                                    margin: Rect::all(Val::Px(5.0)),
                                    ..Default::default()
                                },
                                text: Text::with_section(
                                    "",
                                    TextStyle {
                                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                        font_size: 30.0,
                                        color: Color::WHITE,
                                    },
                                    Default::default(),
                                ),
                                ..Default::default()
                            })
                            .insert(SpeechText);
                        });
                });
        });
}

pub fn debug_draw_cutscene_triggers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut is_drawing: Local<bool>,
    mut cooldown: Local<usize>,
    level_info_state: Res<asset_loader::LevelInfoState>, 
    level_info_assets: ResMut<Assets<asset_loader::LevelInfo>>,

    trigger_meshes: Query<Entity, With<DebugCutsceneTriggerMesh>>, 
) {
    if *cooldown != 0 {
        *cooldown -= 1; 
        return;
    }

    if keyboard_input.just_pressed(KeyCode::C) {
        *is_drawing = !*is_drawing;
        *cooldown = 10;

        if *is_drawing {
            if let Some(levels_asset) = level_info_assets.get(&level_info_state.handle) {
                for cutscene in levels_asset.cutscenes.cutscenes.iter() {
                    let (location, _distance) = cutscene.location;
                    let color = Color::hex("5F0550").unwrap(); 
                    let color = Color::rgba(color.r(), color.g(), color.b(), 0.5);
                    commands.spawn_bundle(PbrBundle {
                                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                                material: materials.add(color.into()),
                                transform: Transform::from_xyz(location.x, 1.5, location.y),
                                visible: Visible {
                                    is_visible: true,
                                    is_transparent: true,
                                },
                                ..Default::default()
                            })
                            .insert(DebugCutsceneTriggerMesh {});
                }
            }
        } else {
            for entity in trigger_meshes.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "498bdc56-8a9c-8543-8640-8018b74b5052"]
pub struct Cutscenes {
    cutscenes: Vec::<Cutscene>
}

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "49cbdc56-aa9c-3543-8640-a018b74b5052"]
pub struct Cutscene {
    location: (Vec2, f32), // X,Z and distance to trigger
    has_been_triggered: bool,
    segments: Vec::<CutsceneSegment>
}

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "49cbdf56-aa9c-3543-8640-bbbbb74b5052"]
pub enum CutsceneSegment {
    CameraPosition,
    Textbox(String),
    CharacterPosition(Character, Position),
    Speech(String, Character),
    Clear(Character),
    Debug(String),
    Delay(f32),
}

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "4bbbdf56-aa9c-3543-8640-bbbbb74b5052"]
pub enum Position {
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "4abadf56-ab9c-3543-8640-bbbbb74b5052"]
pub enum Character {
    Dude
}
