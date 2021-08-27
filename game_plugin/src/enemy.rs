use bevy::prelude::*;
use crate::{player, asset_loader, theater_outside, GameState, level_collision, cutscene, AppState, follow_text::FollowTextEvent};
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
    level: cutscene::Level,
    location: Vec2,
    enemy_type: EnemyType,
    facing: crate::Direction,
}

#[derive(Debug, Clone, Deserialize, TypeUuid, PartialEq)]
#[uuid = "a3da668c-fa5c-402d-ab4f-edf62690827e"]
pub enum EnemyType {
    Ticket,
    Patrol(Vec::<Vec2>),
    Mom,
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
    let color = Color::hex("FF00FF").unwrap(); 
    let color2 = Color::hex("FFFF00").unwrap(); 

    if let Some(levels_asset) = level_info_assets.get(&level_info_state.handle) {
        for enemy_spawn in levels_asset.enemies.iter() {
            if enemy_spawn.level != game_state.current_level { continue; }

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
                        enemy_spawn: enemy_spawn.clone()
                    })
                    .with_children(|parent|  {
                        parent.spawn_bundle(PbrBundle {
                            mesh: theater_meshes.legs.clone(),
                            material: materials.add(color.into()),
                            ..Default::default()
                        });
                        parent.spawn_bundle(PbrBundle {
                            mesh: theater_meshes.torso.clone(),
                            material: materials.add(color.into()),
                            ..Default::default()
                        });
                        parent.spawn_bundle(PbrBundle {
                            mesh: theater_meshes.headhand.clone(),
                            material: materials.add(color.into()),
                            ..Default::default()
                        });
                        parent.spawn_bundle(PbrBundle {
                            mesh: theater_meshes.hat.clone(),
                            material: materials.add(color.into()),
                            ..Default::default()
                        });
                        parent.spawn_bundle(PbrBundle {
                            mesh: theater_meshes.face.clone(),
                            material: theater_meshes.face_material.clone(),
                            ..Default::default()
                        });

                        match enemy_spawn.enemy_type {
                            EnemyType::Patrol(_) => {
                                let color = Color::rgba(color2.r(), color2.g(), color2.b(), 0.7);

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
    mut enemies: Query<(&mut Transform, &mut Enemy)>, 
    time: Res<Time>,
    game_state: Res<GameState>,
    level_info_assets: Res<Assets<asset_loader::LevelInfo>>,
    level_info_state: Res<asset_loader::LevelInfoState>, 
) {
    for (mut transform, mut enemy) in enemies.iter_mut() {
        match &enemy.enemy_spawn.enemy_type {
            EnemyType::Patrol(waypoints) => {

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
                        enemy.velocity.clamp_length_max(SPEED);
                        if distance < 2.0 {
                            enemy.velocity *= FRICTION.powf(time.delta_seconds());
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
            }
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
                let left_angle = angle - VIEW_ANGLE;
                let right_angle = angle + VIEW_ANGLE;

                let left_vector = Vec2::new(left_angle.cos(), left_angle.sin()).normalize() * (VIEW_DISTANCE - 0.7);
                let right_vector = Vec2::new(right_angle.cos(), right_angle.sin()).normalize() * (VIEW_DISTANCE - 0.7);

                for p_transform in player.iter() {
                    let enemy_position = Vec2::new(transform.translation.x, transform.translation.z);
                    let player_position = Vec2::new(p_transform.translation.x, p_transform.translation.z);
                    let triangle: (Vec2, Vec2, Vec2) = (enemy_position, enemy_position + left_vector, enemy_position + right_vector); 
                    if point_in_triangle(player_position, triangle) {
                        let color2 = Color::hex("FF0000").unwrap(); 
                        let color = Color::rgba(color2.r(), color2.g(), color2.b(), 0.7);
                        for child in children.iter() {
                            if let Ok(mut cone_material) = cones.get_mut(*child) {
                                *cone_material = materials.add(color.into());
                            }
                        }

                        follow_text_event_writer.send(FollowTextEvent {
                            entity,
                            value: "Hey!".to_string(),
                            is_player: false,
                        });

                        current_cutscene.trigger(
                            vec!(
                                cutscene::CutsceneSegment::Textbox("Ahh I was seen".to_string()),
                                cutscene::CutsceneSegment::LevelReset,
                            ),
                            game_state.current_level
                        );
                        state.push(AppState::Cutscene).unwrap();
                            
//                        println!("TRUE {:?} {:?}", player_position, triangle);
                    } else {
                        let color2 = Color::hex("FFFF00").unwrap(); 
                        let color = Color::rgba(color2.r(), color2.g(), color2.b(), 0.7);
                        for child in children.iter() {
                            if let Ok(mut cone_material) = cones.get_mut(*child) {
                                *cone_material = materials.add(color.into());
                            }
                        }
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
    enemy_spawn: EnemySpawnPoint,
    target_waypoint: usize,
    is_patroling: bool,
    velocity: Vec3,
}
