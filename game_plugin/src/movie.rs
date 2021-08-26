use bevy::prelude::*;
use crate::{enemy, cutscene, player, theater_outside, asset_loader, camera, level_collision, GameState};

/*
camera_x: -11.483573,
camera_y: 3.0450625,
camera_z: -0.30453277,
camera_rotation_x: 0.070242405,
camera_rotation_y: -0.99498117,
camera_rotation_z: 0.07126288,
camera_rotation_angle: 1.5902499,
*/

pub struct MoviePlugin;
impl Plugin for MoviePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_system_set(
                SystemSet::on_enter(crate::AppState::Movie)
                    .with_system(load_level.system().label("loading_level"))
                    .with_system(crate::camera::create_camera.system().after("loading_level"))
                    .with_system(set_clear_color.system().after("loading_level"))
            )
            .add_system_set(
                SystemSet::on_exit(crate::AppState::Movie)
                    .with_system(cleanup_environment.system())
            )
            .add_system_set(
                SystemSet::on_update(crate::AppState::ResetMovie)
                    .with_system(reset_level.system())
            )
            .add_system_set(
                SystemSet::on_update(crate::AppState::Movie)
                    .with_system(debug_in_movie.system())
                    .with_system(player::player_input.system())
                   // .with_system(check_for_level_exit.system())
                    .with_system(player::player_movement_update.system())
                    .with_system(listen_for_level_reset.system())
            );
    }
}

fn debug_in_movie(
) {
//    println!("In movie room");
}

fn listen_for_level_reset(
    mut state: ResMut<State<crate::AppState>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Key0) {
        state.set(crate::AppState::ResetMovie).unwrap();
    }
}
fn reset_level( 
    mut state: ResMut<State<crate::AppState>>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_seconds();

    if *timer > 1.0 {
        state.set(crate::AppState::Movie).unwrap();
        *timer = 0.0; 
    }
}

struct Movie {}
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
    println!("loading movie");
    let levels_asset = level_info_assets.get(&level_info_state.handle);
    if let Some(level_asset) = levels_asset  {
        println!("Level loaded");
    } else {
        // try again later?
        println!("failed to load movie room");
        return;
    }

    game_state.current_level = cutscene::Level::Movie;
    let color = Color::hex("072AC8").unwrap(); 
    let mut transform = Transform::identity();
//  transform.apply_non_uniform_scale(Vec3::new(SCALE, SCALE, SCALE)); 
//  transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));

    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(Movie {})
            .with_children(|parent|  {
                parent.spawn_bundle(PbrBundle {
                    mesh: theater_meshes.movie.clone(),
                    material: materials.add(color.into()),
                    //material: theater_meshes.movie_material.clone(),
                    transform: {
                        let mut t = Transform::default();
                        t.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));

                        t
                    },
                    ..Default::default()
                });
            }).id();


    player::spawn_player(&mut commands, &mut materials, &mut meshes, &person_meshes, 0, 1, 0);
    enemy::spawn_enemy(&mut commands, &mut materials, &mut meshes, &enemy_meshes, 5, 1, 0);
}

fn set_clear_color(
    mut clear_color: ResMut<ClearColor>,
) {
    clear_color.0 = Color::hex("222222").unwrap();
}

fn cleanup_environment(
    mut commands: Commands,
    level_mesh: Query<Entity, With<Movie>>,
    player: Query<Entity, With<player::Player>>,
    camera: Query<Entity, With<camera::MainCamera>>,
    collision_meshes: Query<Entity, With<level_collision::DebugLevelCollisionMesh>>, 
) {
    for entity in level_mesh.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in player.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // TODO (and also in lobby)
//  for entity in camera.iter() {
//      commands.entity(entity).despawn_recursive();
//  }

    for entity in collision_meshes.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

