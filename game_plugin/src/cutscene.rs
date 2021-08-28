use bevy::prelude::*;
use serde::Deserialize;
use bevy::reflect::{TypeUuid};
use crate::{player,asset_loader,AppState, game_controller, camera, ChangeStateEvent, GameState, LevelResetEvent, Kid, theater_outside};

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

impl CurrentCutscene {
    pub fn trigger(&mut self, segments: Vec::<CutsceneSegment>, level: Level) {
        self.segment_index = 0;
        self.waiting = None;
        self.cutscene = Some(Cutscene {
                            location: (Vec2::ZERO, 0.0), 
                            level,
                            has_been_triggered: false,
                            segments
                        });
    }
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
               SystemSet::on_update(crate::AppState::Lobby)
                   .with_system(check_for_cutscene.system())
                   .with_system(debug_draw_cutscene_triggers.system())

           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::Movie)
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
                   .with_system(make_talk.system())
                   //.with_system(debug_move_character.system())
                   .with_system(update_cutscene.system())

           );
    }
}

pub fn update_cutscene(
    mut game_state: ResMut<GameState>,
    mut current_cutscene: ResMut<CurrentCutscene>,
    mut state: ResMut<State<AppState>>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    gamepad: Option<Res<game_controller::GameController>>,
    mut speechbox_event_writer: EventWriter<SpeechBoxEvent>, 
    mut character_display_event_writer: EventWriter<CharacterDisplayEvent>, 
    mut level_reset_event_writer: EventWriter<LevelResetEvent>,
    mut change_state_event_writer: EventWriter<ChangeStateEvent>,
    mut cameras: Query<&mut Transform, With<camera::MainCamera>>,
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
                println!("Segment: {:?}", segment);
                match segment {
                    CutsceneSegment::Debug(text) => {
                        println!("{}", text);
                        current_cutscene.waiting = Some(CutsceneWait::Time(2.0));
                    },
                    CutsceneSegment::Textbox(text) => {
                        if text.len() == 0 {
                            speechbox_event_writer.send(SpeechBoxEvent { text: None });
                            current_cutscene.waiting = Some(CutsceneWait::Time(0.0));
                        } else {
                            speechbox_event_writer.send(SpeechBoxEvent { text: Some(text.to_string()) });
                            current_cutscene.waiting = Some(CutsceneWait::Interaction);
                        }
                    },
                    CutsceneSegment::LevelReset => {
                        level_reset_event_writer.send(LevelResetEvent);
                        current_cutscene.waiting = Some(CutsceneWait::Time(0.0));
                    },
                    CutsceneSegment::SetTalking(character) => {
                        game_state.currently_talking = Some(*character);
                        current_cutscene.waiting = Some(CutsceneWait::Time(0.0));
                    },
                    CutsceneSegment::CharacterPosition(character, position) => {
                        character_display_event_writer.send(CharacterDisplayEvent {
                            character_and_position: (character.clone(), position.clone())
                        });
                        current_cutscene.waiting = Some(CutsceneWait::Time(0.0));
                    },
                    CutsceneSegment::LevelSwitch(level) => {
                        match level {
                            Level::Lobby => {
                                change_state_event_writer.send(ChangeStateEvent { target: AppState::Lobby });
                            },
                            Level::Movie => {
                                change_state_event_writer.send(ChangeStateEvent { target: AppState::Movie });
                            },
                            Level::Outside => {
                                change_state_event_writer.send(ChangeStateEvent { target: AppState::InGame });
                            },
                        }
                    },
                    CutsceneSegment::SetHalfwayMovie => {
                        game_state.has_seen_half_of_movie = true;
                        game_state.last_positions.insert(Kid::A, Some(Vec3::new(21.0, 16.0, 0.0)));
                        game_state.last_positions.insert(Kid::B, Some(Vec3::new(21.0, 16.0, -1.0)));
                        game_state.last_positions.insert(Kid::C, Some(Vec3::new(21.0, 16.0, -0.5)));
                        game_state.last_positions.insert(Kid::D, Some(Vec3::new(21.0, 16.0, 0.5)));

                        level_reset_event_writer.send(LevelResetEvent);
                        current_cutscene.waiting = Some(CutsceneWait::Time(0.0));
                    },
                    CutsceneSegment::SetGameIsDone => {
                        game_state.game_is_done = true;
                        current_cutscene.waiting = Some(CutsceneWait::Time(0.0));
                    },
                    CutsceneSegment::CameraPosition(x,y,z, rx, ry, rz, rw, speed) => {
                        let mut reached_target = false;
                        for mut transform in cameras.iter_mut() {
                            transform.translation.x += 
                                (x - transform.translation.x) 
                               * speed
                               * time.delta_seconds();
                            transform.translation.y += 
                                (y - transform.translation.y) 
                               * speed
                               * time.delta_seconds();
                            transform.translation.z += 
                                (z - transform.translation.z) 
                               * speed
                               * time.delta_seconds();
                            let rotation = Quat::from_axis_angle(Vec3::new(*rx, *ry, *rz), *rw);
                            transform.rotation = transform.rotation.slerp(rotation, time.delta_seconds());

                            let translation = Vec3::new(*x, *y, *z);
                            if transform.translation.distance(translation) < 0.5 {
                                reached_target = true; 
                            }
                        }

                        if reached_target {
                            current_cutscene.waiting = Some(CutsceneWait::Time(0.0));
                        }
                    },
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
static LEFT_X: f32 = -1.1;
static LEFT_Y: f32 = -0.70000005;
static LEFT_Z: f32 = -2.39;
static LEFT_ROTATION_X: f32 = 0.026847154;
static LEFT_ROTATION_Y: f32 = 0.99964094;
static LEFT_ROTATION_Z: f32 = 0.0030628047;
static LEFT_ROTATION_ANGLE: f32 = 5.635683;

static RIGHT_X: f32 = 1.1000001;
static RIGHT_Y: f32 = -0.70000005;
static RIGHT_Z: f32 = -2.39;
static RIGHT_ROTATION_X: f32 = 0.0062407814;
static RIGHT_ROTATION_Y: f32 = 0.9999601;
static RIGHT_ROTATION_Z: f32 = -0.0065717786;
static RIGHT_ROTATION_ANGLE: f32 = 3.7862408;

static CENTER_RIGHT_X: f32 = 0.6000001;
static CENTER_RIGHT_Y: f32 = -0.70000005;
static CENTER_RIGHT_Z: f32 = -2.9899995;
static CENTER_RIGHT_ROTATION_X: f32 = 0.015168801;
static CENTER_RIGHT_ROTATION_Y: f32 = 0.99987656;
static CENTER_RIGHT_ROTATION_Z: f32 = -0.0043785768;
static CENTER_RIGHT_ROTATION_ANGLE: f32 = 4.1550827;

static CENTER_LEFT_X: f32 = -0.8;
static CENTER_LEFT_Y: f32 = -0.70000005;
static CENTER_LEFT_Z: f32 = -2.9899995;
static CENTER_LEFT_ROTATION_X: f32 = 0.028549988;
static CENTER_LEFT_ROTATION_Y: f32 = 0.99955666;
static CENTER_LEFT_ROTATION_Z: f32 = 0.008944312;
static CENTER_LEFT_ROTATION_ANGLE: f32 = 5.3240614;

pub fn handle_character_display_event(
    mut character_display_event_reader: EventReader<CharacterDisplayEvent>, 
    mut commands: Commands, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    characters: Query<(Entity, &CharacterTracker)>,
    game_state: Res<GameState>,
    person_meshes: Res<player::PersonMeshes>,
    theater_meshes: ResMut<theater_outside::TheaterMeshes>,
    main_camera: Query<Entity, With<camera::MainCamera>>,
) {
    for camera_entity in main_camera.iter() {
        for event in character_display_event_reader.iter() {
            println!("Received display character event");
            let color = Color::hex("FCF300").unwrap(); 
            let kid = match &event.character_and_position.0 {
                         Character::Dude => Kid::A,
                         Character::A => Kid::A,
                         Character::B => Kid::B,
                         Character::C => Kid::C,
                         Character::D => Kid::D,
                      };

            match event.character_and_position {
                (_, Position::Left) => {
                    let mut transform = Transform::from_xyz(LEFT_X, LEFT_Y, LEFT_Z);
                    transform.apply_non_uniform_scale(Vec3::new(SCALE, SCALE, SCALE)); 
                    transform.rotate(Quat::from_axis_angle(Vec3::new(LEFT_ROTATION_X, LEFT_ROTATION_Y, LEFT_ROTATION_Z), LEFT_ROTATION_ANGLE));
                    let character_entity = spawn_character(&mut commands, &game_state, transform, 
                                                           &theater_meshes, &mut materials, kid, event.character_and_position.0);

                    commands.entity(camera_entity).push_children(&[character_entity]);
                },
                (_, Position::Right) => {
                    let mut transform = Transform::from_xyz(RIGHT_X, RIGHT_Y, RIGHT_Z);
                    transform.apply_non_uniform_scale(Vec3::new(SCALE, SCALE, SCALE)); 
                    transform.rotate(Quat::from_axis_angle(Vec3::new(RIGHT_ROTATION_X, RIGHT_ROTATION_Y, RIGHT_ROTATION_Z), RIGHT_ROTATION_ANGLE));
                    let character_entity = spawn_character(&mut commands, &game_state, transform, 
                                                           &theater_meshes, &mut materials, kid, event.character_and_position.0);

                    commands.entity(camera_entity).push_children(&[character_entity]);
                },
                (_, Position::Center_Left) => {
                    let mut transform = Transform::from_xyz(CENTER_LEFT_X, CENTER_LEFT_Y, CENTER_LEFT_Z);
                    transform.apply_non_uniform_scale(Vec3::new(SCALE, SCALE, SCALE)); 
                    transform.rotate(Quat::from_axis_angle(Vec3::new(CENTER_LEFT_ROTATION_X, CENTER_LEFT_ROTATION_Y, CENTER_LEFT_ROTATION_Z), CENTER_LEFT_ROTATION_ANGLE));
                    let character_entity = spawn_character(&mut commands, &game_state, transform, 
                                                           &theater_meshes, &mut materials, kid, event.character_and_position.0);

                    commands.entity(camera_entity).push_children(&[character_entity]);
                },
                (_, Position::Center_Right) => {
                    let mut transform = Transform::from_xyz(CENTER_RIGHT_X, CENTER_RIGHT_Y, CENTER_RIGHT_Z);
                    transform.apply_non_uniform_scale(Vec3::new(SCALE, SCALE, SCALE)); 
                    transform.rotate(Quat::from_axis_angle(Vec3::new(CENTER_RIGHT_ROTATION_X, CENTER_RIGHT_ROTATION_Y, CENTER_RIGHT_ROTATION_Z), CENTER_RIGHT_ROTATION_ANGLE));
                    let character_entity = spawn_character(&mut commands, &game_state, transform, 
                                                           &theater_meshes, &mut materials, kid, event.character_and_position.0);

                    commands.entity(camera_entity).push_children(&[character_entity]);
                },
                (c, Position::Clear) => {
                    for (entity, character) in characters.iter() {
                        if c == character.0 {
                            commands.entity(entity).despawn_recursive();
                        }
                    }
                },
                _ => ()
            }
        }
    }
}

pub fn make_talk(
    game_state: Res<GameState>,
    mut characters: Query<(&CharacterTracker, &Children)>,
    mut faces: Query<(Entity, &mut Visible), (With<EyesMaterial>, Without<MouthMaterial>)>,
    mut mouths: Query<(Entity, &mut Visible), (With<MouthMaterial>, Without<EyesMaterial>)>,
    mut mouth_open: Local<bool>,
    mut sleep: Local<f32>,
    time: Res<Time>,
) {
    *sleep -= time.delta_seconds();

    if *sleep <= 0.0 {
        *sleep = 0.3;
        *mouth_open = !*mouth_open;
    }

    for _ in characters.iter_mut() {
        for (_, mut face) in faces.iter_mut() {
            face.is_visible = true;
        }
        for (_, mut mouth) in mouths.iter_mut() {
            mouth.is_visible = false;
        }
    }
    match game_state.currently_talking {
        Some(currently_talking) => {
            if *mouth_open {
                for (c, children) in characters.iter_mut() {
                    if currently_talking == c.0 {
                        for child_entity in children.iter() {
                            for (e, mut mouth) in mouths.iter_mut() {
                                if e == *child_entity {
                                    mouth.is_visible = true;
                                }
                            }

                            for (e, mut face) in faces.iter_mut() {
                                if e == *child_entity {
                                    face.is_visible = false;
                                }
                            }
                        }
                    } 
                }
            }
        },
        None => ()
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
    game_state: Res<GameState>,
    mut current_cutscene: ResMut<CurrentCutscene>,
) {
    if let Some(levels_asset) = level_info_assets.get_mut(&level_info_state.handle) {
        for player in player.iter() {
            let player_position = Vec2::new(player.translation.x, player.translation.z);
            for mut cutscene in levels_asset.cutscenes.cutscenes.iter_mut() {
                if cutscene.level != game_state.current_level || cutscene.has_been_triggered { continue; }

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
                                    max_size: Size {
                                        width: Val::Px(1280.0),
                                        height: Val::Undefined,
                                    },
                                    ..Default::default()
                                },
                                text: Text::with_section(
                                    "",
                                    TextStyle {
                                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                        font_size: 50.0,
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
    game_state: Res<GameState>,

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

                    if cutscene.level != game_state.current_level { continue; }

                    let (location, _distance) = cutscene.location;
                    let color = Color::hex("5F0550").unwrap(); 
                    let color = Color::rgba(color.r(), color.g(), color.b(), 0.5);
                    commands.spawn_bundle(PbrBundle {
                                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                                material: materials.add(color.into()),
                                transform: Transform::from_xyz(location.x, 1.5, location.y),
                                visible: Visible {
                                    is_visible: true,
                                    is_transparent: !cutscene.has_been_triggered,
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
    level: Level,
    has_been_triggered: bool,
    segments: Vec::<CutsceneSegment>
}

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "49cbdf56-aa9c-3543-8640-bbbbb74b5052"]
pub enum CutsceneSegment {
    CameraPosition(f32, f32, f32, f32, f32, f32, f32, f32), // position, rotation, speed
    Textbox(String),
    CharacterPosition(Character, Position),
    LevelSwitch(Level),
    SetTalking(Character),
    Speech(String, Character),
    Clear(Character),
    LevelReset,
    SetHalfwayMovie,
    SetGameIsDone, 
    Debug(String),
    Delay(f32),
}

#[derive(Debug, Copy, Clone, Deserialize, TypeUuid, PartialEq)]
#[uuid = "21cbdf56-aa9c-3543-8640-bbbbb74b5052"]
pub enum Level {
    Outside,
    Lobby,
    Movie,
}

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "4bbbdf56-aa9c-3543-8640-bbbbb74b5052"]
pub enum Position {
    Left,
    Right,
    Center_Left,
    Center_Right,
    Clear,
}

#[derive(Debug, Copy, Clone, Deserialize, TypeUuid, PartialEq)]
#[uuid = "4abadf56-ab9c-3543-8640-bbbbb74b5052"]
pub enum Character {
    Dude,
    A,
    B,
    C,
    D,
}

fn spawn_character(
    commands: &mut Commands, 
    game_state: &Res<GameState>,
    transform: Transform,
    theater_meshes: &ResMut<theater_outside::TheaterMeshes>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    kid: Kid,
    character: Character,
) -> Entity {
    let leg_color = Color::hex(game_state.kid_colors[&kid].legs.clone()).unwrap();
    let torso_color = Color::hex(game_state.kid_colors[&kid].torso.clone()).unwrap();
    let skin_color = Color::hex(game_state.kid_colors[&kid].skin.clone()).unwrap();
    let hair_color = Color::hex(game_state.kid_colors[&kid].hair.clone()).unwrap();
    let is_long_hair = game_state.kid_colors[&kid].is_long_hair;

    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(CharacterTracker(character))
            .insert(CutsceneTrashMarker)
            .insert(DebugCharacterMarker)
            .with_children(|parent|  {
                parent.spawn_bundle(PbrBundle {
                    mesh: theater_meshes.kid_legs.clone(),
                    material: materials.add(leg_color.into()),
                    ..Default::default()
                });
                parent.spawn_bundle(PbrBundle {
                    mesh: theater_meshes.kid_torso.clone(),
                    material: materials.add(torso_color.into()),
                    ..Default::default()
                });
                parent.spawn_bundle(PbrBundle {
                    mesh: theater_meshes.kid_headhand.clone(),
                    material: materials.add(skin_color.into()),
                    ..Default::default()
                });
                parent.spawn_bundle(PbrBundle {
                    mesh: if kid == Kid::D {
                            theater_meshes.kid_hairtwo.clone()
                          } else {
                              if is_long_hair {
                                  theater_meshes.kid_hairtwo.clone()
                              } else {
                                  println!("hair one!");
                                  theater_meshes.kid_hairone.clone()
                              }
                          },
                    material: materials.add(hair_color.into()),
                    ..Default::default()
                });
                parent.spawn_bundle(PbrBundle {
                    mesh: theater_meshes.kid_face.clone(),
                    material: theater_meshes.face_material.clone(),
                    ..Default::default()
                })
                .insert(EyesMaterial);

                parent.spawn_bundle(PbrBundle {
                    mesh: theater_meshes.kid_face.clone(),
                    material: theater_meshes.talk_material.clone(),
                    ..Default::default()
                })
                .insert(MouthMaterial);
            }).id()
}

pub struct EyesMaterial;
pub struct MouthMaterial;
pub struct CharacterTracker(Character);
