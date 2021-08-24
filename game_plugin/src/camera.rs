use bevy::prelude::*;
use serde::Deserialize;
use bevy::reflect::{TypeUuid};
use bevy::render::camera::PerspectiveProjection;
use crate::{theater_outside::LevelReady, asset_loader, player::Player, level_collision::CollisionShape};

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

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "522adc56-aa9c-4543-8640-a018b74b5052"] // this needs to be actually generated
pub struct CameraPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    pub rotation_angle: f32,
    pub speed: f32,
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
    mut cameras: Query<(Entity, &mut MainCamera, &mut Transform), Without<Player>>,
    fly_camera: Query<&fly_camera::FlyCamera>,
    player: Query<&Transform, (With<Player>, Without<MainCamera>)>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    level_info_assets: Res<Assets<asset_loader::LevelInfo>>,
    level_info_state: Res<asset_loader::LevelInfoState>, 
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

    // this is for debugging. If we're flying, don't move the player
    if fly_camera.iter().count() > 0 {
        return;
    }

    let levels_asset = level_info_assets.get(&level_info_state.handle);
    if let Some(level_asset) = levels_asset  {
        for player in player.iter() {
            for shape in level_asset.collision_info.shapes.iter() {
                match shape {
                    CollisionShape::Rect((r, c)) => {
                        if let Some(c) = c {
                            if player.translation.x >= r.bottom_x 
                            && player.translation.x <= r.top_x 
                            && player.translation.z <= r.right_z
                            && player.translation.z >= r.left_z {

                                for (_e, _camera, mut transform) in cameras.iter_mut() {
                                    transform.translation.x += 
                                        (c.x - transform.translation.x) 
                                       * c.speed
                                       * time.delta_seconds();
                                    transform.translation.y += 
                                        (c.y - transform.translation.y) 
                                       * c.speed
                                       * time.delta_seconds();
                                    transform.translation.z += 
                                        (c.z - transform.translation.z) 
                                       * c.speed
                                       * time.delta_seconds();

                                    let end_rotation = Quat::from_axis_angle(Vec3::new(c.rotation_x, c.rotation_y, c.rotation_z), 
                                                                             c.rotation_angle);
                                    transform.rotation = transform.rotation.slerp(end_rotation, time.delta_seconds());
                                }
                            }
                        }
                    }
                    _ => ()
                }
            }
        }
    }
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
