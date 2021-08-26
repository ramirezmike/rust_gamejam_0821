use bevy::prelude::*;

use crate::{Direction, game_controller, game_settings, asset_loader, level_collision, GameState};

pub struct Player {
    movement: Option::<Direction>,
    velocity: Vec3,
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

    x: usize,
    y: usize,
    z: usize,
) {
    let color = Color::hex("FCF300").unwrap(); 

    let mut transform = Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32));
    transform.apply_non_uniform_scale(Vec3::new(SCALE, SCALE, SCALE)); 
    transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));
    let inner_mesh_vertical_offset = 1.0;
    let player_entity = 
    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(Player {
                movement: None,
                velocity: Vec3::default()
            })
            .with_children(|parent|  {
                parent.spawn_bundle(PbrBundle {
                    mesh: person_meshes.person.clone(),
                    material: materials.add(color.into()),
                    transform: Transform::from_xyz(0.0, 0.5, 0.0),
                    ..Default::default()
                });
            }).id();
}

pub fn player_movement_update(
    mut player: Query<(&mut Player, &mut Transform)>,
    settings: Res<game_settings::GameSettings>,
    time: Res<Time>,
    game_state: Res<GameState>,
    level_info_assets: Res<Assets<asset_loader::LevelInfo>>,
    level_info_state: Res<asset_loader::LevelInfoState>, 
) {
    for (mut player, mut transform) in player.iter_mut() {
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
            if !new_rotation.is_nan() {
                transform.rotation = rotation;
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
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    gamepad: Option<Res<game_controller::GameController>>,
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
        if (keyboard_input.just_pressed(KeyCode::Space) 
        || keyboard_input.just_pressed(KeyCode::Return) 
        || keyboard_input.just_pressed(KeyCode::J) 
        || pressed_buttons.contains(&game_controller::GameButton::Action))
        && action_buffer.is_none() {
            *action_buffer = Some(time.time_since_startup().as_millis());
            continue;
        }

        if !action_buffer.is_none() {
            continue;
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

//      if movement_got_set {
//          squash_queue.squashes.clear();

//          // squashes are done in reverse
//          squash_queue.squashes.push(Squash {
//              start_scale: Vec3::new(0.7, 1.4, 1.0),
//              target_scale: Vec3::new(1.0, 1.0, 1.0),
//              start_vertical: 2.5,
//              target_vertical: 1.0,
//              start_horizontal: 0.0,
//              target_horizontal: 0.0,
//              current_scale_time: 0.0,
//              finish_scale_time: 0.20,
//          });
//          squash_queue.squashes.push(Squash {
//              start_scale: Vec3::new(1.0, 1.0, 1.0),
//              target_scale: Vec3::new(0.7, 1.4, 1.0),
//              start_vertical: 1.0,
//              target_vertical: 2.5,
//              start_horizontal: 0.0,
//              target_horizontal: 0.0,
//              current_scale_time: 0.0,
//              finish_scale_time: 0.05,
//          });

//          create_dust_event_writer.send(dust::CreateDustEvent { 
//              position: Position::from_vec(transform.translation),
//              move_away_from: move_dir,
//          });
//      }
    }
}
