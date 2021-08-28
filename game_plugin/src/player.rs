use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::{Direction, game_controller, game_settings, asset_loader, level_collision, 
            GameState, theater_outside, Kid, Mode, follow_text, enemy} ;

static DISTRACT_TEXT: &str = "[DISTRACT]";
static DISTRACT_DISTANCE: f32 = 2.0;
pub struct Player {
    pub kid: Kid,
    pub is_distracting: Option::<Entity>,
    pub movement: Option::<Direction>,
    pub velocity: Vec3,
}

pub static SCALE: f32 = 0.36;

#[derive(Default)]
pub struct PersonMeshes {
    pub person: Handle<Mesh>,
}

pub fn load_assets(
    asset_server: Res<AssetServer>,
    mut person_meshes: ResMut<PersonMeshes>,
    mut loading: ResMut<asset_loader::AssetsLoading>,
) {
    println!("Adding person assets");
    person_meshes.person = asset_server.load("models/dude.glb#Mesh0/Primitive0");

    loading.asset_handles.push(person_meshes.person.clone_untyped());
}

pub fn spawn_player(
    commands: &mut Commands, 
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    person_meshes: &Res<PersonMeshes>,
    theater_meshes: &ResMut<theater_outside::TheaterMeshes>,
    game_state: &ResMut<GameState>,

) {
    let kids = vec!(Kid::A, Kid::B, Kid::C, Kid::D);

    for (i, kid) in kids.iter().enumerate() {
        let position = game_state.last_positions[kid].unwrap();
        let mut transform = Transform::from_translation(position);
        transform.apply_non_uniform_scale(Vec3::new(SCALE, SCALE, SCALE)); 
        transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));

        let leg_color = Color::hex(game_state.kid_colors[&kid].legs.clone()).unwrap();
        let torso_color = Color::hex(game_state.kid_colors[&kid].torso.clone()).unwrap();
        let skin_color = Color::hex(game_state.kid_colors[&kid].skin.clone()).unwrap();
        let hair_color = Color::hex(game_state.kid_colors[&kid].hair.clone()).unwrap();

        let player_entity = 
        commands.spawn_bundle(PbrBundle {
                    transform,
                    ..Default::default()
                })
                .insert(Player {
                    kid: *kid,
                    movement: None,
                    is_distracting: None,
                    velocity: Vec3::default()
                })
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
                        mesh: if *kid == Kid::D {
                                theater_meshes.kid_hairtwo.clone()
                              } else {
                                  // omg this is gross
                                  let mut rng = rand::thread_rng();
                                  let mut nums: Vec<i32> = (0..1).collect();
                                  nums.shuffle(&mut rng);
                                  if *nums.last().unwrap() == 0{
                                      theater_meshes.kid_hairone.clone()
                                  } else {
                                      theater_meshes.kid_hairtwo.clone()
                                  }
                              },
                        material: materials.add(hair_color.into()),
                        ..Default::default()
                    });
                    parent.spawn_bundle(PbrBundle {
                        mesh: theater_meshes.kid_face.clone(),
                        material: theater_meshes.face_material.clone(),
                        ..Default::default()
                    });
                }).id();
    }
}

pub fn player_movement_update(
    mut player: Query<(&mut Player, &mut Transform)>,
    settings: Res<game_settings::GameSettings>,
    time: Res<Time>,
    game_state: ResMut<GameState>,
    level_info_assets: Res<Assets<asset_loader::LevelInfo>>,
    level_info_state: Res<asset_loader::LevelInfoState>, 
) {
    match game_state.mode {
        Mode::Follow => {
            let mut controlling_kid_position = Vec3::default();
            let mut move_away_from = vec!();
            for (player, transform) in player.iter_mut() {
                if player.kid ==  game_state.controlling {
                    controlling_kid_position = transform.translation;
                } else {
                    move_away_from.push((player.kid, transform.translation)); 
                }
            }

            for (mut player, mut transform) in player.iter_mut() {
                if player.kid ==  game_state.controlling {
                    move_player_controlled_kid(&mut player, &mut transform, &game_state, &settings, 
                                               &level_info_assets, &level_info_state, &time);
                } else {
                    move_non_player_controlled_kid(&mut player, &mut transform, &controlling_kid_position, 
                                                   &move_away_from, &game_state, &settings, 
                                                   &level_info_assets, &level_info_state, &time);
                }
            }
        },
        Mode::Switch => {
            for (mut player, mut transform) in player.iter_mut() {
                if player.kid ==  game_state.controlling {
                    move_player_controlled_kid(&mut player, &mut transform, &game_state, &settings, 
                                               &level_info_assets, &level_info_state, &time);
                }
            }
        }
    }
}

pub fn move_non_player_controlled_kid(
    player: &mut Player, 
    mut transform: &mut Transform, 
    controlling_kid_position: &Vec3,
    move_away_from: &Vec::<(Kid, Vec3)>,
    game_state: &ResMut<GameState>,
    settings: &Res<game_settings::GameSettings>,
    level_info_assets: &Res<Assets<asset_loader::LevelInfo>>,
    level_info_state: &Res<asset_loader::LevelInfoState>, 
    time: &Res<Time>,
) {
    let direction = *controlling_kid_position - transform.translation;
    let distance = direction.length();
    if distance > 3.0 {
        player.velocity += (direction * settings.player_speed) * time.delta_seconds();
    } else {
        player.velocity = Vec3::ZERO;
    }
    for (kid, translation) in move_away_from.iter() {
        if *kid != player.kid {
            let direction = transform.translation - *translation;
            let distance = direction.length();
            if distance < 1.5 {
                player.velocity += (direction * settings.player_speed) * time.delta_seconds();
            } 
        }
    }

    player.velocity = player.velocity.clamp_length_max(settings.player_speed * 0.1);
    let new_translation = transform.translation + player.velocity;

    let levels_asset = level_info_assets.get(&level_info_state.handle);
    if let Some(level_asset) = levels_asset  {
        let temp_new_translation = new_translation;
        let new_translation = level_collision::fit_in_level(&level_asset, game_state, transform.translation, new_translation);
        if temp_new_translation.x != new_translation.x {
            player.velocity.x = 0.0;
        }
        if temp_new_translation.y != new_translation.y {
            player.velocity.y = 0.0;
        }
        if temp_new_translation.z != new_translation.z {
            player.velocity.z = 0.0;
        }

        // wow, this actually works?
        let angle = (-(new_translation.z - transform.translation.z)).atan2(new_translation.x - transform.translation.x);
        let rotation = Quat::from_axis_angle(Vec3::Y, angle);

        if new_translation.length() > 0.001 {
            transform.translation = new_translation; 
        } 

        let new_rotation = transform.rotation.lerp(rotation, time.delta_seconds());

        // don't rotate if we're not moving or if uhh rotation isnt a number
        if !new_rotation.is_nan() && player.velocity.length() > 0.01 {
            transform.rotation = rotation;
        }
    }
}

pub fn move_player_controlled_kid(
    mut player: &mut Player, 
    mut transform: &mut Transform, 
    game_state: &ResMut<GameState>,
    settings: &Res<game_settings::GameSettings>,
    level_info_assets: &Res<Assets<asset_loader::LevelInfo>>,
    level_info_state: &Res<asset_loader::LevelInfoState>, 
    time: &Res<Time>,
) {
    if let Some(movement) = player.movement {
        let mut acceleration = Vec3::default();
        match movement {
            Direction::Up => {
                acceleration += Vec3::new(1.0, 0.0, 0.0);
            },
            Direction::Down => {
                acceleration += Vec3::new(-1.0, 0.0, 0.0);
            },
            Direction::Right => {
                acceleration += Vec3::new(0.0, 0.0, 1.0);
            },
            Direction::Left => {
                acceleration += Vec3::new(0.0, 0.0, -1.0);
            },
            _ => ()
        } 

        player.velocity += (acceleration * settings.player_speed) * time.delta_seconds();
        player.movement = None;
    } else {
        player.velocity *= settings.player_friction.powf(time.delta_seconds());
    }

    player.velocity = player.velocity.clamp_length_max(settings.player_speed);
    let new_translation = transform.translation + player.velocity;

    let levels_asset = level_info_assets.get(&level_info_state.handle);
    if let Some(level_asset) = levels_asset  {
        let temp_new_translation = new_translation;
        let new_translation = level_collision::fit_in_level(&level_asset, &game_state, transform.translation, new_translation);
        if temp_new_translation.x != new_translation.x {
            player.velocity.x = 0.0;
        }
        if temp_new_translation.y != new_translation.y {
            player.velocity.y = 0.0;
        }
        if temp_new_translation.z != new_translation.z {
            player.velocity.z = 0.0;
        }

        // wow, this actually works?
        let angle = (-(new_translation.z - transform.translation.z)).atan2(new_translation.x - transform.translation.x);
        let rotation = Quat::from_axis_angle(Vec3::Y, angle);
        transform.translation = new_translation; 

        let new_rotation = transform.rotation.lerp(rotation, time.delta_seconds());

        // don't rotate if we're not moving or if uhh rotation isnt a number
        if !new_rotation.is_nan() && player.velocity.length() > 0.0001 {
            transform.rotation = rotation;
        }
    }
}

pub fn player_interact_check(
    players: Query<(Entity, &Transform, &Player), Without<enemy::Enemy>>,
    enemies: Query<(&Transform, &enemy::Enemy), Without<Player>>,
    game_state: Res<GameState>,
    mut follow_text_event_writer: EventWriter<follow_text::FollowTextEvent>,
    mut sleep: Local<f32>,
    time: Res<Time>, 
) {
    *sleep -= time.delta_seconds();
    if *sleep > 0.0 {
        return;
    }

    let mut text = None;
    let mut entity = None;

    // TODO: need to do something when there is only one player left?
    for (player_entity, player_transform, player) in players.iter() {
        if player.kid != game_state.controlling { continue; }

        entity = Some(player_entity);

        for (enemy_transform, enemy) in enemies.iter() {
            match enemy.enemy_spawn.enemy_type {
                enemy::EnemyType::Ticket(actually_checks) => {
                    if player_transform.translation.distance(enemy_transform.translation) <= DISTRACT_DISTANCE
                    && !enemy.is_distracted
                    && (!actually_checks || game_state.has_ticket.contains(&player.kid)) {
                        text = Some(DISTRACT_TEXT);
                    }
                },
                _ => ()
            }

        }
    }

    if let Some(entity) = entity {
        follow_text_event_writer.send(
            follow_text::FollowTextEvent {
                entity: entity,
                value: text.unwrap_or("").to_string(),
                is_player: true,
                force: true
            }
        );
    }

    *sleep = 1.0;
}

pub struct DistractEvent {
    is_starting: bool
}

pub fn handle_distract_event(
    mut distract_event_reader: EventReader<DistractEvent>,
    mut enemies: Query<(Entity, &mut enemy::Enemy, &Transform)>,
    mut players: Query<(&mut Player, &Transform)>,
    mut follow_text: ResMut<follow_text::FollowText>,
    mut game_state: ResMut<GameState>,
) {
    for event in distract_event_reader.iter() {
        println!("Got event");
        for (mut player, player_transform) in players.iter_mut() {
            if player.kid == game_state.controlling {
                println!("Got player");
                for (entity, mut enemy, enemy_transform) in enemies.iter_mut() {
                    println!("checking enemy");
                    if enemy_transform.translation.distance(player_transform.translation) < DISTRACT_DISTANCE {
                        println!("found enemy nearby");
                        if event.is_starting {
                            println!("Setting enemy distracted");
                            enemy.is_distracted = true;
                            player.is_distracting = Some(entity);

                            // SWITCH LOGIC oh my what have I done this is truly terrible

                            let remaining_kids =
                                game_state.last_positions
                                          .iter()
                                          .filter(|(_, position)| !position.is_none())
                                          .map(|(kid, _)| kid)
                                          .collect::<Vec::<_>>();
                            let current = remaining_kids.iter().position(|k| **k == game_state.controlling).unwrap();

                            let next_kid =
                                if let Some(k) = remaining_kids.get(current + 1) {
                                    k
                                } else {
                                    remaining_kids[0]
                                };
                                      
                            game_state.controlling = *next_kid;
                            player.velocity = Vec3::default();

                            follow_text.player_value = "".to_string();
                            player.movement = None;
                        } else {
                            println!("Setting enemy not distracted");
                            enemy.is_distracted = false;
                            player.is_distracting = None;
                        }
                    }
                }
            }
        }
    }
}

pub fn player_input(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>, 
    mut player: Query<&mut Player>,
    camera: Query<&crate::camera::fly_camera::FlyCamera>,
    mut action_buffer: Local<Option::<u128>>,
    mut up_buffer: Local<Option::<u128>>,
    mut down_buffer: Local<Option::<u128>>,
    mut right_buffer: Local<Option::<u128>>,
    mut left_buffer: Local<Option::<u128>>,
    mut game_state: ResMut<GameState>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    gamepad: Option<Res<game_controller::GameController>>,
    mut follow_text: ResMut<follow_text::FollowText>,
    mut distract_event_writer: EventWriter<DistractEvent>,
) {
    let time_buffer = 100;

    // this is for debugging. If we're flying, don't move the player
    if camera.iter().count() > 0 {
        return;
    }

    let time_since_startup = time.time_since_startup().as_millis();
    if let Some(time_since_up) = *up_buffer {
        if time_since_startup - time_since_up > time_buffer {
            *up_buffer = None;
        }
    }
    if let Some(time_since_down) = *down_buffer {
        if time_since_startup - time_since_down > time_buffer {
            *down_buffer = None;
        }
    }
    if let Some(time_since_left) = *left_buffer {
        if time_since_startup - time_since_left > time_buffer {
            *left_buffer = None;
        }
    }
    if let Some(time_since_right) = *right_buffer {
        if time_since_startup - time_since_right > time_buffer {
            *right_buffer = None;
        }
    }
    if let Some(time_since_action) = *action_buffer {
        if time_since_startup - time_since_action > time_buffer {
            *action_buffer = None;
        }
    }

    let pressed_buttons = game_controller::get_pressed_buttons(&axes, &buttons, gamepad);
    for mut player in player.iter_mut() {
        if player.kid != game_state.controlling { continue; }

        if !action_buffer.is_none() {
            continue;
        }

        if (keyboard_input.just_pressed(KeyCode::Space) 
        || keyboard_input.just_pressed(KeyCode::Return) 
        || keyboard_input.just_pressed(KeyCode::J) 
        || keyboard_input.just_pressed(KeyCode::K) 
        || pressed_buttons.contains(&game_controller::GameButton::Action)
        || pressed_buttons.contains(&game_controller::GameButton::Switch))
        && action_buffer.is_none() {
            *action_buffer = Some(time.time_since_startup().as_millis());
        }

        if keyboard_input.just_pressed(KeyCode::K) || pressed_buttons.contains(&game_controller::GameButton::Switch) {
                            // SWITCH LOGIC oh my what have I done this is truly terrible
                            let remaining_kids =
                                game_state.last_positions
                                          .iter()
                                          .filter(|(_, position)| !position.is_none())
                                          .map(|(kid, _)| kid)
                                          .collect::<Vec::<_>>();
                            let current = remaining_kids.iter().position(|k| **k == game_state.controlling).unwrap();

                            let next_kid =
                                if let Some(k) = remaining_kids.get(current + 1) {
                                    k
                                } else {
                                    remaining_kids[0]
                                };
                                      
                            game_state.controlling = *next_kid;
            player.velocity = Vec3::default();

            follow_text.player_value = "".to_string();
            player.movement = None;


            distract_event_writer.send(DistractEvent {
                is_starting: false,
            });
        }

        if keyboard_input.just_pressed(KeyCode::J) || pressed_buttons.contains(&game_controller::GameButton::Action) {
            if follow_text.player_value == DISTRACT_TEXT {
                distract_event_writer.send(DistractEvent {
                    is_starting: true,
                });
            }
        }

        let mut move_dir = None;
        if (keyboard_input.pressed(KeyCode::W) 
         || keyboard_input.pressed(KeyCode::Up) 
         || pressed_buttons.contains(&game_controller::GameButton::Up))
           && up_buffer.is_none() {
            move_dir = Some(Direction::Up); 
            *up_buffer = Some(time.time_since_startup().as_millis());
        }
        if (keyboard_input.pressed(KeyCode::S) 
           || keyboard_input.pressed(KeyCode::Down) 
           || pressed_buttons.contains(&game_controller::GameButton::Down))
           && down_buffer.is_none() {
            move_dir = Some(Direction::Down); 
            *down_buffer = Some(time.time_since_startup().as_millis());
        }
        if (keyboard_input.pressed(KeyCode::A) 
           || keyboard_input.pressed(KeyCode::Left) 
           || pressed_buttons.contains(&game_controller::GameButton::Left))
           && left_buffer.is_none() {
            move_dir = Some(Direction::Left); 
            *left_buffer = Some(time.time_since_startup().as_millis());
        }
        if (keyboard_input.pressed(KeyCode::D) 
           || keyboard_input.pressed(KeyCode::Right) 
           || pressed_buttons.contains(&game_controller::GameButton::Right))
           && right_buffer.is_none() {
            move_dir = Some(Direction::Right); 
            *right_buffer= Some(time.time_since_startup().as_millis());
        }

        player.movement = move_dir;
    }
}
