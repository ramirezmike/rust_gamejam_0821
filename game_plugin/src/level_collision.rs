use bevy::{prelude::*,};
use serde::Deserialize;
use bevy::reflect::{TypeUuid};
use crate::{asset_loader, camera, lerp};

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "40cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct LevelCollisionInfo {
    pub shapes: Vec::<CollisionShape>
}


#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "41cadc56-aa9c-4543-8640-a018b74b5052"]
pub enum CollisionShape {
    Rect((RectangleCollision, Option::<camera::CameraPosition>)),
    Stair(RectangleCollision)
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

pub fn fit_in_level(
    level_info: &asset_loader::LevelInfo,
    current: Vec3,
    target: Vec3,
) -> Vec3 {
    let mut current_shapes = vec!();
    for shape in level_info.collision_info.shapes.iter() {
        match shape {
            CollisionShape::Rect((r, _)) => {
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
                    current_shapes.push(r); 
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
                    current_shapes.push(r); 
                }
            }
        }
    }

    if !current_shapes.is_empty() {
        let first_shape = current_shapes[0];
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

        Vec3::new(x, first_shape.height, z)
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
    mut level_info_state: ResMut<asset_loader::LevelInfoState>, 
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
                for shape in level_asset.collision_info.shapes.iter() {
                    match shape {
                        CollisionShape::Rect((r, _)) | CollisionShape::Stair(r) => {
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
