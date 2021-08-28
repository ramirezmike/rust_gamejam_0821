use bevy::{prelude::*,};
use serde::Deserialize;
use bevy::reflect::{TypeUuid};
use crate::{asset_loader, camera, lerp, cutscene, GameState, Kid, player, AppState, enemy};

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "40cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct LevelCollisionInfo {
    pub shapes: Vec::<(cutscene::Level, CollisionShape)>
}


#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "41cadc56-aa9c-4543-8640-a018b74b5052"]
pub enum CollisionShape {
    Rect((RectangleCollision, Option::<camera::CameraPosition>)),
    GetTicket((RectangleCollision, Option::<camera::CameraPosition>)),
    TicketCheck((RectangleCollision, Option::<camera::CameraPosition>)),
    Stair(RectangleCollision),
    DespawnPlayer((RectangleCollision, Vec3)),
    LevelSwitch((RectangleCollision, Option::<camera::CameraPosition>))
}

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "42cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct RectangleCollision {
    pub left_z: f32,
    pub right_z: f32,
    pub top_x: f32,
    pub bottom_x: f32,
    pub height: f32,
    pub base_height: f32,
}

pub fn ticket_checker(
    mut game_state: ResMut<GameState>,
    players: Query<(&Transform, &player::Player)>,
    enemies: Query<(&Transform, &enemy::Enemy)>,
    mut current_cutscene: ResMut<cutscene::CurrentCutscene>,
    mut state: ResMut<State<AppState>>,
    level_info_assets: Res<Assets<asset_loader::LevelInfo>>,
    level_info_state: Res<asset_loader::LevelInfoState>, 
) {
    if game_state.has_ticket.contains(&game_state.controlling) {
        return;
    }

    let levels_asset = level_info_assets.get(&level_info_state.handle);
    if let Some(level_info) = levels_asset {
        for (transform, player) in players.iter() {
            if player.kid != game_state.controlling { continue; }

            for (level, shape) in level_info.collision_info.shapes.iter() {
                if *level == game_state.current_level {
                    match shape {
                        CollisionShape::TicketCheck((r, _)) => {
                            for (enemy_transform, enemy) in enemies.iter() {

                                // enemy in this box isn't distracted
                                if !enemy.is_distracted
                                && enemy_transform.translation.x >= r.bottom_x 
                                && enemy_transform.translation.x <= r.top_x 
                                && enemy_transform.translation.z <= r.right_z
                                && enemy_transform.translation.z >= r.left_z {

                                    // player is in this box
                                    if transform.translation.x >= r.bottom_x 
                                    && transform.translation.x <= r.top_x 
                                    && transform.translation.z <= r.right_z
                                    && transform.translation.z >= r.left_z {
                                        current_cutscene.trigger(
                                            vec!(
                                                cutscene::CutsceneSegment::Textbox("Ahh I don't have a ticket".to_string()),
                                                cutscene::CutsceneSegment::LevelReset,
                                            ),
                                            game_state.current_level
                                        );
                                        state.push(AppState::Cutscene).unwrap();
                                    }
                                }
                            }
                        },
                        CollisionShape::GetTicket((r, _)) => {
                                if transform.translation.x >= r.bottom_x 
                                && transform.translation.x <= r.top_x 
                                && transform.translation.z <= r.right_z
                                && transform.translation.z >= r.left_z {
                                    game_state.has_ticket.push(player.kid.clone());
                                }
                            },
                        _ => ()
                    }
                }
            }
        }
    }
}

pub fn fit_in_level(
    level_info: &asset_loader::LevelInfo,
    game_state: &ResMut<GameState>,
    current: Vec3,
    target: Vec3,
) -> Vec3 {
                                                      // isStair
    let mut current_shapes: Vec::<(&RectangleCollision, bool)> = vec!();
    for (level, shape) in level_info.collision_info.shapes.iter() {
        if *level == game_state.current_level {
            match shape {
                CollisionShape::Rect((r, _)) 
              | CollisionShape::LevelSwitch((r, _)) 
              | CollisionShape::TicketCheck((r, _)) 
              | CollisionShape::GetTicket((r, _)) => {
                    if target.x >= r.bottom_x 
                    && target.x <= r.top_x 
                    && target.z <= r.right_z
                    && target.z >= r.left_z {
                        return Vec3::new(target.x, r.height, target.z);
                    }

                    if current.x >= r.bottom_x 
                    && current.x <= r.top_x 
                    && current.z <= r.right_z
                    && current.z >= r.left_z {
                        current_shapes.push((r, false)); 
                    }
                },
                CollisionShape::Stair(r) => {
                    if target.x >= r.bottom_x 
                    && target.x <= r.top_x 
                    && target.z <= r.right_z
                    && target.z >= r.left_z {
                        let height = lerp(r.base_height, r.height, (target.x - r.bottom_x) / (r.top_x - r.bottom_x));

                        return Vec3::new(target.x, height, target.z);
                    }

                    if current.x >= r.bottom_x 
                    && current.x <= r.top_x 
                    && current.z <= r.right_z
                    && current.z >= r.left_z {
                        current_shapes.push((r, true)); 
                    }
                },
                _ => ()
            }
        }
    }

    if !current_shapes.is_empty() {
        let (first_shape, is_stair) = current_shapes[0];
        let x = if target.x < first_shape.bottom_x {
                    first_shape.bottom_x
                } else if target.x > first_shape.top_x {
                    first_shape.top_x
                } else {
                    target.x
                };
        let z = if target.z < first_shape.left_z {
                    first_shape.left_z
                } else if target.z > first_shape.right_z {
                    first_shape.right_z
                } else {
                    target.z
                };

        Vec3::new(x, if is_stair { current.y } else { first_shape.height }, z)
    } else {
        current
    }
}

pub fn debug_draw_level_colliders(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut is_drawing: Local<bool>,
    mut cooldown: Local<usize>,
    game_state: Res<GameState>,
    level_info_state: Res<asset_loader::LevelInfoState>, 
    level_info_assets: ResMut<Assets<asset_loader::LevelInfo>>,

    collision_meshes: Query<Entity, With<DebugLevelCollisionMesh>>, 
) {
    if *cooldown != 0 {
        *cooldown -= 1; 
        return;
    }

    if keyboard_input.just_pressed(KeyCode::O) {
        *is_drawing = !*is_drawing;
        *cooldown = 10;

        if *is_drawing {
            let levels_asset = level_info_assets.get(&level_info_state.handle);
            if let Some(level_asset) = levels_asset {
                for (level, shape) in level_asset.collision_info.shapes.iter() {
                    if *level == game_state.current_level {
                        match shape {
                            CollisionShape::Rect((r, _)) 
                            | CollisionShape::Stair(r) 
                            | CollisionShape::LevelSwitch((r, _)) 
                            | CollisionShape::TicketCheck((r, _)) 
                            | CollisionShape::GetTicket((r, _)) => {
                                let color = Color::hex("FF0000").unwrap(); 
                                let color = Color::rgba(color.r(), color.g(), color.b(), 0.5);

                                // left side
                                commands.spawn_bundle(PbrBundle {
                                            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                                            material: materials.add(color.into()),
                                            transform: {
                                                let mut transform = Transform::from_xyz(r.bottom_x + ((r.top_x - r.bottom_x) / 2.0), r.height, r.left_z);
                                                transform.apply_non_uniform_scale(Vec3::new(r.top_x - r.bottom_x, 1.0, 1.0)); 

                                                transform
                                            },
                                            visible: Visible {
                                                is_visible: true,
                                                is_transparent: true,
                                            },
                                            ..Default::default()
                                        })
                                        .insert(DebugLevelCollisionMesh {});

                                // right side
                                commands.spawn_bundle(PbrBundle {
                                            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                                            material: materials.add(color.into()),
                                            transform: {
                                                let mut transform = Transform::from_xyz(r.bottom_x + ((r.top_x - r.bottom_x) / 2.0), r.height, r.right_z);
                                                transform.apply_non_uniform_scale(Vec3::new(r.top_x - r.bottom_x, 1.0, 1.0)); 

                                                transform
                                            },
                                            visible: Visible {
                                                is_visible: true,
                                                is_transparent: true,
                                            },
                                            ..Default::default()
                                        })
                                        .insert(DebugLevelCollisionMesh {});

                                // top side
                                commands.spawn_bundle(PbrBundle {
                                            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                                            material: materials.add(color.into()),
                                            transform: {
                                                let mut transform = Transform::from_xyz(r.top_x, r.height, r.left_z + (r.right_z - r.left_z) / 2.0);
                                                transform.apply_non_uniform_scale(Vec3::new(1.0, 1.0, r.right_z - r.left_z)); 

                                                transform
                                            },
                                            visible: Visible {
                                                is_visible: true,
                                                is_transparent: true,
                                            },
                                            ..Default::default()
                                        })
                                        .insert(DebugLevelCollisionMesh {});

                                // bottom side
                                commands.spawn_bundle(PbrBundle {
                                            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                                            material: materials.add(color.into()),
                                            transform: {
                                                let mut transform = Transform::from_xyz(r.bottom_x, r.height, r.left_z + (r.right_z - r.left_z) / 2.0);
                                                transform.apply_non_uniform_scale(Vec3::new(1.0, 1.0, r.right_z - r.left_z)); 

                                                transform
                                            },
                                            visible: Visible {
                                                is_visible: true,
                                                is_transparent: true,
                                            },
                                            ..Default::default()
                                        })
                                        .insert(DebugLevelCollisionMesh {});
                            },
                            _ => ()
                        }
                    }
                }
            }
        } else {
            for entity in collision_meshes.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

pub struct DebugLevelCollisionMesh { }
