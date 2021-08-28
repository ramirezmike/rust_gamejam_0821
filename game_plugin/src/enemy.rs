use bevy::prelude::*;
use crate::{player, asset_loader, theater_outside, GameState, level_collision, cutscene, AppState, follow_text::FollowTextEvent, get_colors};
use bevy::render::pipeline::PrimitiveTopology;
use serde::Deserialize;
use bevy::reflect::{TypeUuid};
use bevy::render::mesh::Indices;

#[derive(Default)]
pub struct EnemyMeshes {
    pub fov_cone: Handle<Mesh>,
}

pub static SCALE: f32 = 0.36;
pub static SPEED: f32 = 0.1;
pub static FRICTION: f32 = 0.1;

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "31c0df2d-8f17-4ed3-906f-e4e7ca870c2f"]
pub struct EnemySpawnPoint {
    pub level: cutscene::Level,
    pub location: Vec2,
    pub enemy_type: EnemyType,
    pub facing: crate::Direction,
}

#[derive(Debug, Clone, Deserialize, TypeUuid, PartialEq)]
#[uuid = "a3da668c-fa5c-402d-ab4f-edf62690827e"]
pub enum EnemyType {
    Ticket(bool),
    Patrol(Vec::<Vec2>),
    Mom,
    Camera,
    Dog
}

pub struct Cone { }
pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .init_resource::<EnemyMeshes>()
            .add_system_set(
               SystemSet::on_enter(crate::AppState::Loading)
                         .with_system(load_assets.system())
            )
            .add_system_set(
               SystemSet::on_update(crate::AppState::Lobby)
                    .with_system(print.system())
                    .with_system(update_enemy.system())
                    .with_system(scale_cone.system())
                    .with_system(check_for_player.system())
            )
            .add_system_set(
               SystemSet::on_update(crate::AppState::Movie)
                    .with_system(print.system())
                    .with_system(update_enemy.system())
                    .with_system(scale_cone.system())
                    .with_system(check_for_player.system())
            )
            .add_system_set(
               SystemSet::on_update(crate::AppState::InGame)
                    .with_system(print.system())
                    .with_system(update_enemy.system())
                    .with_system(scale_cone.system())
                    .with_system(check_for_player.system())
            );
    }
}

pub fn load_assets(
    asset_server: Res<AssetServer>,
    mut enemy_meshes: ResMut<EnemyMeshes>,
    mut loading: ResMut<asset_loader::AssetsLoading>,
) {
    println!("Adding enemy assets");
    enemy_meshes.fov_cone = asset_server.load("models/cone.glb#Mesh0/Primitive0");

    loading.asset_handles.push(enemy_meshes.fov_cone.clone_untyped());
}

static VIEW_DISTANCE :f32 = 5.7;
static VIEW_ANGLE: f32 = 0.5;

pub fn spawn_enemies(
    mut commands: Commands, 
    mut materials: ResMut<Assets<StandardMaterial>>,
//    meshes: ResMut<Assets<Mesh>>,
    enemy_meshes: Res<EnemyMeshes>,
    theater_meshes: ResMut<theater_outside::TheaterMeshes>,

    game_state: Res<GameState>,
    level_info_state: Res<asset_loader::LevelInfoState>, 
    level_info_assets: ResMut<Assets<asset_loader::LevelInfo>>,
) {
    let leg_color = Color::hex("293241").unwrap(); 
    let torso_color = Color::hex("e63946").unwrap(); 
    let hat_color = Color::hex("e63946").unwrap();
    let other_colors = get_colors();
    let vision_color = Color::hex("fdffb6").unwrap();

    if let Some(levels_asset) = level_info_assets.get(&level_info_state.handle) {
        for enemy_spawn in levels_asset.enemies.iter() {
            if enemy_spawn.level != game_state.current_level { continue; }

            let skin_color = Color::hex(other_colors.skin.to_string()).unwrap();

            let mut transform = Transform::from_translation(Vec3::new(enemy_spawn.location.x as f32, 
                                                                      0.0 as f32, 
                                                                      enemy_spawn.location.y as f32));
            transform.apply_non_uniform_scale(Vec3::new(SCALE, SCALE, SCALE)); 

            // do direction
            transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));

            commands.spawn_bundle(PbrBundle {
                        transform,
                        ..Default::default()
                    })
                    .insert(Enemy {
                        target_waypoint: 0,
                        velocity: Vec3::default(),
                        is_patroling: true,
                        is_distracted: false,
                        enemy_spawn: enemy_spawn.clone()
                    })
                    .with_children(|parent|  {
                        parent.spawn_bundle(PbrBundle {
                            mesh: theater_meshes.legs.clone(),
                            material: materials.add(leg_color.into()),
                            ..Default::default()
                        });
                        parent.spawn_bundle(PbrBundle {
                            mesh: theater_meshes.torso.clone(),
                            material: materials.add(torso_color.into()),
                            ..Default::default()
                        });
                        parent.spawn_bundle(PbrBundle {
                            mesh: theater_meshes.headhand.clone(),
                            material: materials.add(skin_color.into()),
                            ..Default::default()
                        });
                        parent.spawn_bundle(PbrBundle {
                            mesh: theater_meshes.hat.clone(),
                            material: materials.add(hat_color.into()),
                            ..Default::default()
                        });
                        parent.spawn_bundle(PbrBundle {
                            mesh: theater_meshes.face.clone(),
                            material: theater_meshes.face_material.clone(),
                            ..Default::default()
                        });

                        match enemy_spawn.enemy_type {
                            EnemyType::Patrol(_) => {
                                let color = Color::rgba(vision_color.r(), vision_color.g(), vision_color.b(), 0.7);

                                parent.spawn_bundle(PbrBundle {
                                    mesh: enemy_meshes.fov_cone.clone(),
                                    material: materials.add(color.into()),
                                    visible: Visible {
                                        is_visible: true,
                                        is_transparent: true,
                                    },
                                    transform: {
                                        let mut t = Transform::from_xyz(0.0, -2.2, 0.0);
                                        t.scale = Vec3::new(VIEW_DISTANCE, VIEW_DISTANCE, VIEW_DISTANCE);

                                        t
                                    },
                                    ..Default::default()
                                }).insert(Cone {});
                            },
                            _ => ()
                        }
                    }).id();
        }
    }
}

pub fn update_enemy(
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy)>, 
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut follow_text_event_writer: EventWriter<FollowTextEvent>,
    level_info_assets: Res<Assets<asset_loader::LevelInfo>>,
    level_info_state: Res<asset_loader::LevelInfoState>, 
) {
    for (entity, mut transform, mut enemy) in enemies.iter_mut() {
        match &enemy.enemy_spawn.enemy_type {
            EnemyType::Patrol(waypoints) => {
                // this is pretty bad but it lets me end the game easier
                if enemy.target_waypoint >= waypoints.len() - 1
                && game_state.current_level == cutscene::Level::Movie
                && !game_state.has_avoided_movie_guard {
                    game_state.has_avoided_movie_guard = true;
                }

                if game_state.has_avoided_movie_guard { return; }

                let current_position = Vec2::new(transform.translation.x, transform.translation.z); 
                if let Some(point) = waypoints.get(enemy.target_waypoint) {
                    let distance = current_position.distance(*point);
                    if distance < 0.1 {
                        enemy.target_waypoint 
                            = if enemy.target_waypoint >= waypoints.len() - 1 {
                                  0
                              } else {
                                  enemy.target_waypoint + 1
                              };
                    } else {
                        //let angle = current_position.angle_between(*point);
                        let move_toward = (*point - current_position).normalize();
                        let move_toward = Vec3::new(move_toward.x, 0.0, move_toward.y);

                        enemy.velocity += (move_toward * SPEED) * time.delta_seconds();
                        enemy.velocity = enemy.velocity.clamp_length_max(SPEED);

                        if distance < 2.0 {
                            enemy.velocity *= FRICTION.powf(time.delta_seconds());
                        }

                        if game_state.current_level == cutscene::Level::Movie {
                            enemy.velocity = enemy.velocity.clamp_length_max(SPEED / 2.0);
                        }

                        let new_translation = transform.translation + enemy.velocity;

                        let levels_asset = level_info_assets.get(&level_info_state.handle);
                        if let Some(level_asset) = levels_asset  {
                            let temp_new_translation = new_translation;
                            let new_translation = level_collision::fit_in_level(&level_asset, &game_state, transform.translation, new_translation);
                            if temp_new_translation.x != new_translation.x {
                                enemy.velocity.x = 0.0;
                            }
                            if temp_new_translation.y != new_translation.y {
                                enemy.velocity.y = 0.0;
                            }
                            if temp_new_translation.z != new_translation.z {
                                enemy.velocity.z = 0.0;
                            }

                            // wow, this actually works?
                            let angle = (-(new_translation.z - transform.translation.z)).atan2(new_translation.x - transform.translation.x);
                            let rotation = Quat::from_axis_angle(Vec3::Y, angle);

                            transform.translation = new_translation; 

                            let new_rotation = transform.rotation.lerp(rotation, time.delta_seconds());

                            // don't rotate if we're not moving or if uhh rotation isnt a number
                            if !new_rotation.is_nan() && enemy.velocity.length() > 0.0001 {
                                transform.rotation = rotation;
                            }
                         }
                    }
                }
            },
            EnemyType::Ticket(_actually_checks) => {
                if enemy.is_distracted {
                    follow_text_event_writer.send(FollowTextEvent {
                        entity,
                        value: "I'm distracted!".to_string(),
                        is_player: false,
                        force: false,
                    });
                }
            },
            _ => ()
        }
    }
}

fn scale_cone(
    keyboard_input: Res<Input<KeyCode>>,
    mut cones: Query<&mut Transform, With<Cone>>,
) {
    for mut t in cones.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::Y) {
            t.scale.z += 0.1;
            t.scale.x += 0.1;
        }
        if keyboard_input.just_pressed(KeyCode::H) {
            t.scale.z -= 0.1;
            t.scale.x -= 0.1;
        }
        if keyboard_input.just_pressed(KeyCode::T) {
            t.translation.x += 0.1;
        }
        if keyboard_input.just_pressed(KeyCode::G) {
            t.translation.x -= 0.1;
        }
    }
}

pub fn check_for_player(
    enemies: Query<(Entity, &Enemy, &Transform, &Children)>,
    mut cones: Query<&mut Handle<StandardMaterial>, With<Cone>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut current_cutscene: ResMut<cutscene::CurrentCutscene>,
    mut state: ResMut<State<AppState>>,
    mut follow_text_event_writer: EventWriter<FollowTextEvent>,
    game_state: Res<GameState>,
    player: Query<&Transform, With<player::Player>>,
) {
    for (entity, enemy, transform, children) in enemies.iter() {
        match enemy.enemy_spawn.enemy_type {
            EnemyType::Patrol(_) => {
                let (axis, mut angle) = transform.rotation.to_axis_angle();
                if axis.y >= -0.0 {
                    angle = -angle;
                } 
                let view_angle = if game_state.current_level == cutscene::Level::Movie {
                                    0.1
                                } else {
                                    VIEW_ANGLE
                                };
                let left_angle = angle - view_angle;
                let right_angle = angle + view_angle;

                let view_distance = if game_state.current_level == cutscene::Level::Movie {
                                        VIEW_DISTANCE - 2.7
                                    } else {
                                        VIEW_DISTANCE - 0.7
                                    };
                let left_vector = Vec2::new(left_angle.cos(), left_angle.sin()).normalize() * (view_distance);
                let right_vector = Vec2::new(right_angle.cos(), right_angle.sin()).normalize() * (view_distance);

                for p_transform in player.iter() {
                    let enemy_position = Vec2::new(transform.translation.x, transform.translation.z);
                    let player_position = Vec2::new(p_transform.translation.x, p_transform.translation.z);
                    let triangle: (Vec2, Vec2, Vec2) = (enemy_position, enemy_position + left_vector, enemy_position + right_vector); 
                    if point_in_triangle(player_position, triangle) {

                        follow_text_event_writer.send(FollowTextEvent {
                            entity,
                            value: "Hey!".to_string(),
                            is_player: false,
                            force: true,
                        });

                        current_cutscene.trigger(
                            level_collision::random_death_two(&game_state),
                            game_state.current_level
                        );
                        state.push(AppState::Cutscene).unwrap();
                            
//                        println!("TRUE {:?} {:?}", player_position, triangle);
                    } else {
                    }
                }
                //println!("Angle: {} {}", axis, angle);
            },
            _ => ()
        }
    }
}

fn point_in_triangle(
    p: Vec2,
    t: (Vec2, Vec2, Vec2)
) -> bool {
    // The point p is inside the triangle if 0 <= s <= 1 and 0 <= t <= 1 and s + t <= 1.
    // s,t and 1 - s - t are called the barycentric coordinates of the point p.
    let a = 0.5 * (-t.1.y * t.2.x + t.0.y * (-t.1.x + t.2.x) + t.0.x * (t.1.y - t.2.y) + t.1.x * t.2.y);
    let sign = if a < 0.0 { -1.0 } else { 1.0 };
    let s = (t.0.y * t.2.x - t.0.x * t.2.y + (t.2.y - t.0.y) * p.x + (t.0.x - t.2.x) * p.y) * sign;
    let t = (t.0.x * t.1.y - t.0.y * t.1.x + (t.0.y - t.1.y) * p.x + (t.1.x - t.0.x) * p.y) * sign;
    s > 0.0 && t > 0.0 && (s + t) < 2.0 * a * sign
}

pub fn print(
    enemies: Query<(&Enemy, &Transform)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::I) {
        for (_, transform) in enemies.iter() {
            let (rotation, axis) = transform.rotation.to_axis_angle();

            println!("Axis: {} {} {} Angle: {},", rotation.x, rotation.y, rotation.z, axis); 
        }
    }
}

pub struct Enemy {
    pub enemy_spawn: EnemySpawnPoint,
    pub target_waypoint: usize,
    pub is_patroling: bool,
    pub velocity: Vec3,
    pub is_distracted: bool,
}
