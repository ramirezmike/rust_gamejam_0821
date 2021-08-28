use bevy::prelude::*;
use crate::{enemy, cutscene, player, theater_outside, asset_loader, camera, level_collision, GameState, AppState, Mode, Kid, };

pub struct LobbyPlugin;
impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_system_set(
                SystemSet::on_enter(crate::AppState::Lobby)
                    .with_system(load_level.system().label("loading_level"))
                    .with_system(crate::camera::create_camera.system().after("loading_level"))
                    .with_system(set_clear_color.system().after("loading_level"))
                    .with_system(enemy::spawn_enemies.system().after("loading_level"))
            )
            .add_system_set(
                SystemSet::on_exit(crate::AppState::Lobby)
                    .with_system(cleanup_environment.system())
            )
            .add_system_set(
                SystemSet::on_update(crate::AppState::ResetLobby)
                    .with_system(reset_level.system())
            )
            .add_system_set(
                SystemSet::on_update(crate::AppState::Lobby)
                    .with_system(debug_in_lobby.system())
                    .with_system(player::player_input.system())
                    .with_system(player::handle_distract_event.system())
                    .with_system(level_collision::ticket_checker.system())
                    .with_system(check_for_level_exit.system())
                    .with_system(player::player_movement_update.system())
                    .with_system(listen_for_level_reset.system())
            );
    }
}

fn debug_in_lobby(
) {

}

fn listen_for_level_reset(
    mut state: ResMut<State<crate::AppState>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Key0) {
        state.set(crate::AppState::ResetLobby).unwrap();
    }
}
fn reset_level( 
    mut state: ResMut<State<crate::AppState>>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_seconds();

    if *timer > 1.0 {
        state.set(crate::AppState::Lobby).unwrap();
        *timer = 0.0; 
    }
}

struct Lobby {}
fn load_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut theater_meshes: ResMut<theater_outside::TheaterMeshes>,
    enemy_meshes: Res<enemy::EnemyMeshes>,
    person_meshes: Res<player::PersonMeshes>,
    asset_server: Res<AssetServer>,
    mut level_info_state: ResMut<asset_loader::LevelInfoState>, 
    level_info_assets: ResMut<Assets<asset_loader::LevelInfo>>,
    mut game_state: ResMut<GameState>,

    mut state: ResMut<State<crate::AppState>>,

) {
    println!("loading lobby");
    let levels_asset = level_info_assets.get(&level_info_state.handle);
    if let Some(level_asset) = levels_asset  {
        println!("Level loaded");
    } else {
        // try again later?
        println!("failed to load lobby");
        return;
    }

    game_state.current_level = cutscene::Level::Lobby;
    game_state.mode = Mode::Switch;


    let color = Color::hex("072AC8").unwrap(); 
    let mut transform = Transform::identity();
//  transform.apply_non_uniform_scale(Vec3::new(SCALE, SCALE, SCALE)); 
//  transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));

    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(Lobby {})
            .with_children(|parent|  {
                parent.spawn_bundle(PbrBundle {
                    mesh: theater_meshes.lobby.clone(),
                    material: theater_meshes.lobby_material.clone(),
                    transform: {
                        let mut t = Transform::default();
                        t.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));

                        t
                    },
                    ..Default::default()
                });

                parent.spawn_bundle(PbrBundle {
                    mesh: theater_meshes.lobby_railing.clone(),
                    //material: theater_meshes.lobby_material.clone(),
                    transform: {
                        let mut t = Transform::default();
                        t.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));

                        t
                    },
                    ..Default::default()
                });

                parent.spawn_bundle(PbrBundle {
                    mesh: theater_meshes.lobby_railing.clone(),
                    material: theater_meshes.lobby_material.clone(),
                    transform: {
                        let mut t = Transform::default();
                        t.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));

                        t
                    },
                    ..Default::default()
                });

                parent.spawn_bundle(PbrBundle {
                    mesh: theater_meshes.lobby_concession.clone(),
                    material: theater_meshes.lobby_material.clone(),
                    transform: {
                        let mut t = Transform::default();
                        t.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));

                        t
                    },
                    ..Default::default()
                });

                parent.spawn_bundle(PbrBundle {
                    mesh: theater_meshes.lobby_desk.clone(),
                    material: theater_meshes.lobby_material.clone(),
                    transform: {
                        let mut t = Transform::default();
                        t.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));

                        t
                    },
                    ..Default::default()
                });
            }).id();

    game_state.last_positions = [
                (Kid::A, Some(Vec3::new(0.0, 0.0, 0.0))), 
                (Kid::B, Some(Vec3::new(0.0, 0.0, -1.0))),
                (Kid::C, Some(Vec3::new(0.0, 0.0, 0.5))), 
                (Kid::D, Some(Vec3::new(0.0, 0.0, -0.5))),
    ].iter().cloned().collect();
    player::spawn_player(&mut commands, &mut materials, &mut meshes, 
                         &person_meshes, &theater_meshes, &game_state);
}

fn set_clear_color(
    mut clear_color: ResMut<ClearColor>,
) {
    clear_color.0 = Color::hex("222222").unwrap();
}

fn cleanup_environment(
    mut commands: Commands,
    level_mesh: Query<Entity, With<Lobby>>,
    player: Query<Entity, With<player::Player>>,
    enemy: Query<Entity, With<enemy::Enemy>>,
    camera: Query<Entity, With<camera::MainCamera>>,
    collision_meshes: Query<Entity, With<level_collision::DebugLevelCollisionMesh>>, 
) {
    for entity in level_mesh.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in player.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in enemy.iter() {
        commands.entity(entity).despawn_recursive();
    }

//  for entity in camera.iter() {
//      commands.entity(entity).despawn_recursive();
//  }

    for entity in collision_meshes.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn check_for_level_exit(
    mut commands: Commands,
    players: Query<(Entity, &Transform, &player::Player)>,
    level_info_assets: Res<Assets<asset_loader::LevelInfo>>,
    level_info_state: Res<asset_loader::LevelInfoState>, 
    mut current_cutscene: ResMut<cutscene::CurrentCutscene>,
    mut game_state: ResMut<GameState>,
    mut state: ResMut<State<AppState>>,
) {
    let levels_asset = level_info_assets.get(&level_info_state.handle);
    if let Some(level_asset) = levels_asset  {
        let number_of_players = players.iter().len();
        for (entity, transform, player) in players.iter() {
            for (level, shape) in level_asset.collision_info.shapes.iter() {
                if *level == game_state.current_level {
                    match shape {
                        level_collision::CollisionShape::DespawnPlayer((r, cancel_position)) => {
                            if transform.translation.x >= r.bottom_x 
                            && transform.translation.x <= r.top_x 
                            && transform.translation.z <= r.right_z
                            && transform.translation.z >= r.left_z {
                                if number_of_players <= 1 {
                                    println!("Level switch triggered!");
                                    current_cutscene.trigger(
                                        vec!(
                                            //cutscene::CutsceneSegment::CameraPosition((Vec3::ZERO, Quat::default(), 1.0)),
                                            cutscene::CutsceneSegment::LevelSwitch(cutscene::Level::Movie),
                                        ),
                                        cutscene::Level::Lobby
                                    );

                                    state.push(AppState::Cutscene).unwrap();
                                } else {
                                    commands.entity(entity).despawn_recursive();
                                    game_state.last_positions.insert(player.kid, None);
                                    game_state.controlling = game_state.last_positions
                                                                       .iter()
                                                                       .filter(|(_key, value)| !value.is_none())
                                                                       .map(|(key, _)| key)
                                                                       .last()
                                                                       .unwrap()
                                                                       .clone();
                                }

                            }
                        }
                        _ => ()
                    }
                }
            }
        }
    }
}
