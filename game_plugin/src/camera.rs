use bevy::prelude::*;
use serde::Deserialize;
use bevy::reflect::{TypeUuid};
use bevy::render::camera::PerspectiveProjection;
use crate::theater_outside::LevelReady;

pub mod fly_camera;

pub struct CameraTarget;

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "59cadc56-aa9c-4543-8640-a018b74b5052"] // this needs to be actually generated
pub enum CameraBehavior {
    Static,
    LockFollowX(f32, f32),
    LockFollowY(f32, f32, f32),
    FollowY(f32),
    FollowZ(f32),
    LooseFollowX(f32),
    MoveToX(f32),
}

use fly_camera::*;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app//.add_plugin(PickingPlugin)
           .add_system_set(
               SystemSet::on_update(crate::AppState::InGame)
                         .with_system(toggle_fly.system())
           )
           .add_system_set(
               SystemSet::on_enter(crate::AppState::InGame)
                         .with_system(reset_camera_on_enter_ingame.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::MainMenu)
                         .with_system(toggle_fly.system())
           )
           .add_plugin(FlyCameraPlugin)
           .add_system(update_camera.system());
    }
}

pub fn reset_camera_on_enter_ingame(
    mut main_camera: Query<&mut MainCamera>,
) {
    for mut camera in main_camera.iter_mut() {
        camera.current_followx_target = None;
        camera.current_followy_target = None;
    }
}

fn lerp(a: f32, b: f32, f: f32) -> f32 {
    return a + f * (b - a);
}
    
fn update_camera(
    mut cameras: Query<(Entity, &mut MainCamera, &mut Transform)>,
    target: Query<&Transform, (With<CameraTarget>, Without<MainCamera>)>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    if keyboard_input.just_pressed(KeyCode::P) {
        for (_e, _camera, transform) in cameras.iter_mut() {
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

//  for (_, mut main_camera, mut camera_transform) in cameras.iter_mut() {
//      if let Ok(target_transform) = target.single() {
//          for behavior in level.camera_behaviors() {
//              match behavior {
//                  CameraBehavior::LockFollowX(min, max) => {
//                      match &mut main_camera.current_followx_target {
//                          Some(movement) => {
//                              movement.current_movement_time += time.delta_seconds();

//                              let new_translation = lerp(movement.starting_from, movement.target, 
//                                                         movement.current_movement_time / movement.finish_movement_time);
//                                                   
//                              if movement.current_movement_time > movement.finish_movement_time {
//                                  camera_transform.translation.x = movement.target;
//                                  main_camera.current_followx_target = None;
//                              } else if !new_translation.is_nan() {
//                                  camera_transform.translation.x = new_translation;
//                              }
//                          },
//                          None => {
//                              let x_distance = target_transform.translation.x - camera_transform.translation.x;
//                              if x_distance > *max || x_distance < *min {
//                                  main_camera.current_followx_target = Some(
//                                      CameraMovement {
//                                          target: target_transform.translation.x 
//                                                  - ((*max - *min) / 2.0)
//                                                  - *min,
//                                          starting_from: camera_transform.translation.x,
//                                          current_movement_time: 0.0,
//                                          finish_movement_time: 0.5,
//                                      }
//                                  );
//                              } 
//                          }
//                      }
//                  },
//                  CameraBehavior::LockFollowY(min, max, offset) => {
//                      match &mut main_camera.current_followy_target {
//                          Some(movement) => {
//                              movement.current_movement_time += time.delta_seconds();

//                              let new_translation = lerp(movement.starting_from, movement.target, 
//                                                         movement.current_movement_time / movement.finish_movement_time);
//                                                   
//                              if movement.current_movement_time > movement.finish_movement_time {
//                                  camera_transform.translation.y = movement.target;
//                                  main_camera.current_followy_target = None;
//                              } else if !new_translation.is_nan() {
//                                  camera_transform.translation.y = new_translation;
//                              }
//                          },
//                          None => {
//                              let y_distance = target_transform.translation.y - camera_transform.translation.y;
//                              if y_distance > *max || y_distance < *min {
//                                  main_camera.current_followy_target = Some(
//                                      CameraMovement {
//                                          target: target_transform.translation.y + offset,
//                                          starting_from: camera_transform.translation.y,
//                                          current_movement_time: 0.0,
//                                          finish_movement_time: 0.5,
//                                      }
//                                  );
//                              } 
//                          }
//                      }
//                  },
//                  CameraBehavior::FollowY(offset) => {
//                      camera_transform.translation.y += 
//                          (target_transform.translation.y - camera_transform.translation.y + offset) 
//                         * if is_menu { 0.4 } else { 0.8 } 
//                         * time.delta_seconds();
//                  },
//                  CameraBehavior::LooseFollowX(offset) => {
//                      camera_transform.translation.x += 
//                          (target_transform.translation.x - camera_transform.translation.x + offset) 
//                         * 1.8 
//                         * time.delta_seconds();
//                  },
//                  CameraBehavior::MoveToX(offset) => {
//                      camera_transform.translation.x += 
//                          (offset - camera_transform.translation.x) 
//                         * 1.8 
//                         * time.delta_seconds();
//                  },
//                  CameraBehavior::FollowZ(offset) => {
//                      camera_transform.translation.z += 
//                          (target_transform.translation.z - camera_transform.translation.z + offset) 
//                         * 0.8 
//                         * time.delta_seconds();
//                  },
//                  CameraBehavior::Static => (),
//              }
//          }
//      }
//  }
}

#[derive(Debug, PartialEq)]
pub enum MovementStep { Start, Middle, Loading, End }
impl Default for MovementStep {
    fn default() -> Self { MovementStep::Start }
}

#[derive(Default)]
pub struct CameraMouthMovement {
    moving: bool,
    current_movement_time: f32,
    current_movement_step: MovementStep,  
}

pub struct CameraMouth {
    start: Vec3,
    middle: Vec3,
    end: Vec3,
}

#[derive(Default)]
pub struct CameraBoltMovement {
    moving: bool,
    current_movement_time: f32,
    current_movement_step: MovementStep,  
}

pub struct CameraBolt {
    start: Vec3,
    middle: Vec3,
    end: Vec3,
}

#[derive(Default)]
pub struct CameraSpikeMovement {
    moving: bool,
    current_movement_time: f32,
    current_movement_step: MovementStep,  
}

pub struct CameraSpike {
    start: Vec3,
    middle: Vec3,
    end: Vec3,
}

pub fn create_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cameras: Query<&mut Transform, With<MainCamera>>,
    level_ready: Res<LevelReady>,
) {
    if !level_ready.0 {
        return; // level isn't loaded so we'll try again later
    }

    let mut transform = Transform::default();
    transform.translation = Vec3::new(-12.5, 10.5, 0.0);
    transform.rotation = Quat::from_axis_angle(Vec3::new(-0.20287918, -0.9580786, -0.20229985), 1.6107514);

    if let Ok(mut camera_transform) = cameras.single_mut() {
        *camera_transform = transform;
    } else {
        println!("Creating camera!");

        commands
            .spawn_bundle(PerspectiveCameraBundle {
                transform, 
                ..Default::default()
            })
            .insert(MainCamera {
                current_followx_target: None,
                current_followy_target: None
            });
    }
//  // destroy any existing main cameras
//  for camera in cameras.iter() {
//      println!("destroying camera");
//      commands.entity(camera).despawn_recursive();
//  }

}

pub struct CameraMovement {
    target: f32,
    starting_from: f32,
    current_movement_time: f32,
    finish_movement_time: f32,
}

pub struct MainCamera {
    pub current_followx_target: Option<CameraMovement>,
    pub current_followy_target: Option<CameraMovement>,
}

static DEFAULT_FOV: f32 = 0.7853982; 

fn toggle_fly(
    mut commands: Commands, 
    keys: Res<Input<KeyCode>>, 
    mut windows: ResMut<Windows>,
    mut camera: Query<(Entity, &mut MainCamera, Option<&FlyCamera>, &mut Transform)>,
    mut cooldown: Local<f32>,
    timer: Res<Time>,
) {
    *cooldown += timer.delta_seconds();

    if *cooldown < 2.0 {
        return;
    }

    if keys.just_pressed(KeyCode::F) {
        println!("PRESSED F");
        let window = windows.get_primary_mut().unwrap();
        for (e, _, f, mut t) in camera.iter_mut() {
            match f {
                Some(_) => {
                    commands.entity(e).remove::<FlyCamera>();
                    window.set_cursor_lock_mode(false);
                    window.set_cursor_visibility(true);
                },
                None => {
                    let mut fly_camera = FlyCamera::default();
                    fly_camera.key_forward = KeyCode::W; 
                    fly_camera.key_backward = KeyCode::S; 
                    fly_camera.key_left = KeyCode::A; 
                    fly_camera.key_right = KeyCode::D; 
                    commands.entity(e).insert(fly_camera);
                    window.set_cursor_lock_mode(true);
                    window.set_cursor_visibility(false);
                },
            }
        }

        *cooldown = 0.0;
    }
}
