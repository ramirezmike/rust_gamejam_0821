use bevy::prelude::*;
use crate::{asset_loader, player, camera, level_collision, enemy, AppState, GameState,
            level_collision::CollisionShape, cutscene, cutscene::CutsceneSegment };

pub struct LevelReady(pub bool);
pub struct TheaterOutsidePlugin;
impl Plugin for TheaterOutsidePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .insert_resource(LevelReady(false))
            .init_resource::<TheaterMeshes>()
            .add_plugin(enemy::EnemyPlugin)
            .add_plugin(cutscene::CutscenePlugin)

            //.insert_resource(PathFinder::new())
            //.init_resource::<crate::pause::PauseButtonMaterials>()
            //.add_plugin(AudioPlugin)
            //.add_event::<holdable::LiftHoldableEvent>()
            .add_system_set(
               SystemSet::on_enter(crate::AppState::Loading)
                         .with_system(player::load_assets.system())
                         .with_system(load_assets.system())
            )
            .add_system_set(
                SystemSet::on_enter(crate::AppState::InGame)
                    .with_system(load_level.system().label("loading_level"))
                    .with_system(crate::camera::create_camera.system().after("loading_level"))
                    .with_system(set_clear_color.system().after("loading_level"))

            )
            .add_system_set(
                SystemSet::on_exit(crate::AppState::InGame)
                    .with_system(cleanup_environment.system())
            )
            .add_system_set(
                SystemSet::on_update(crate::AppState::ResetLevel)
                    .with_system(reset_level.system())
            )
            .add_system_set(
               SystemSet::on_update(crate::AppState::InGame)
                    .with_system(player::player_input.system())
                    .with_system(check_for_level_exit.system())
                    .with_system(player::player_movement_update.system())
                    .with_system(listen_for_level_reset.system())
               //.with_system(holdable::lift_holdable.system().label("handle_lift_events"))
            );
    }
}

#[derive(Default)]
pub struct TheaterMeshes {
    pub outside: Handle<Mesh>,
    pub lobby: Handle<Mesh>,
    pub lobby_railing: Handle<Mesh>,
    pub lobby_concession: Handle<Mesh>,
    pub lobby_desk: Handle<Mesh>,
    pub outside_material: Handle<StandardMaterial>,
    pub lobby_material: Handle<StandardMaterial>,
}

pub fn check_for_level_exit(
    player: Query<&Transform, With<player::Player>>,
    level_info_assets: Res<Assets<asset_loader::LevelInfo>>,
    level_info_state: Res<asset_loader::LevelInfoState>, 
    mut current_cutscene: ResMut<cutscene::CurrentCutscene>,
    game_state: Res<GameState>,
    mut state: ResMut<State<AppState>>,
) {
    let levels_asset = level_info_assets.get(&level_info_state.handle);
    if let Some(level_asset) = levels_asset  {
        for player in player.iter() {
            for (level, shape) in level_asset.collision_info.shapes.iter() {
                if *level == game_state.current_level {
                    match shape {
                        CollisionShape::LevelSwitch((r, _)) => {
                            if player.translation.x >= r.bottom_x 
                            && player.translation.x <= r.top_x 
                            && player.translation.z <= r.right_z
                            && player.translation.z >= r.left_z {
                                println!("Level switch triggered!");
                                current_cutscene.trigger(
                                    vec!(
                                        CutsceneSegment::CameraPosition((Vec3::ZERO, Quat::default(), 1.0)),
                                        CutsceneSegment::LevelSwitch(cutscene::Level::Lobby),
                                    )
                                );

                                state.push(AppState::Cutscene).unwrap();
                            }
                        }
                        _ => ()
                    }
                }
            }
        }
    }
}

fn load_assets(
//    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut theater_meshes: ResMut<TheaterMeshes>,
    mut level_info_state: ResMut<asset_loader::LevelInfoState>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut loading: ResMut<asset_loader::AssetsLoading>,
) {
    println!("Adding theater assets");
    theater_meshes.outside = asset_server.load("models/theater_outside.glb#Mesh0/Primitive0");
    theater_meshes.lobby = asset_server.load("models/lobby.glb#Mesh0/Primitive0");
    theater_meshes.lobby_railing = asset_server.load("models/lobby.glb#Mesh1/Primitive0");
    theater_meshes.lobby_concession = asset_server.load("models/lobby.glb#Mesh2/Primitive0");
    theater_meshes.lobby_desk = asset_server.load("models/lobby.glb#Mesh3/Primitive0");

    let texture_handle = asset_server.load("models/theater_outside.png");
    theater_meshes.outside_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        ..Default::default()
    });

    let texture_handle = asset_server.load("models/lobby.png");
    theater_meshes.lobby_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        ..Default::default()
    });

    loading.asset_handles.push(theater_meshes.outside.clone_untyped());
    loading.asset_handles.push(theater_meshes.lobby.clone_untyped());
    loading.asset_handles.push(theater_meshes.lobby_railing.clone_untyped());
    loading.asset_handles.push(theater_meshes.lobby_concession.clone_untyped());
    loading.asset_handles.push(theater_meshes.lobby_desk.clone_untyped());

    level_info_state.handle = asset_server.load("data/outside.lvl");
    asset_server.watch_for_changes().unwrap();
}

fn listen_for_level_reset(
    mut state: ResMut<State<crate::AppState>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::R) {
        state.set(crate::AppState::ResetLevel).unwrap();
    }
}
fn reset_level( 
    mut state: ResMut<State<crate::AppState>>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_seconds();

    if *timer > 1.0 {
        state.set(crate::AppState::InGame).unwrap();
        *timer = 0.0; 
    }
}

fn cleanup_environment(
    mut commands: Commands,
    level_mesh: Query<Entity, With<TheaterOutside>>,
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

    for entity in camera.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in collision_meshes.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

struct TheaterOutside { }
fn load_level( 
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut level_ready: ResMut<LevelReady>,
    mut theater_meshes: ResMut<TheaterMeshes>,
    enemy_meshes: Res<enemy::EnemyMeshes>,
    person_meshes: Res<player::PersonMeshes>,
    asset_server: Res<AssetServer>,
    mut level_info_state: ResMut<asset_loader::LevelInfoState>, 
    level_info_assets: ResMut<Assets<asset_loader::LevelInfo>>,
    mut game_state: ResMut<GameState>,
    mut state: ResMut<State<crate::AppState>>,

) {
    println!("loading level");
    println!("Starting to load level...");
    let levels_asset = level_info_assets.get(&level_info_state.handle);
    if let Some(level_asset) = levels_asset  {
        println!("Level loaded");
    } else {
        // try again later?
        println!("failed to load level");
        state.set(crate::AppState::Loading).unwrap();
        return;
    }

    game_state.current_level = cutscene::Level::Outside;
    let color = Color::hex("072AC8").unwrap(); 
    let mut transform = Transform::identity();
//  transform.apply_non_uniform_scale(Vec3::new(SCALE, SCALE, SCALE)); 
//  transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));

    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(TheaterOutside {})
            .with_children(|parent|  {
                parent.spawn_bundle(PbrBundle {
                    mesh: theater_meshes.outside.clone(),
                    material: theater_meshes.outside_material.clone(),
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

    level_ready.0 = true;

}

fn set_clear_color(
    mut clear_color: ResMut<ClearColor>,
) {
    clear_color.0 = Color::hex("555555").unwrap();
}

