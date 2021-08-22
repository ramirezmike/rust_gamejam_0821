use bevy::prelude::*;

use crate::{asset_loader, player, };

pub struct LevelReady(pub bool);
pub struct TheaterOutsidePlugin;
impl Plugin for TheaterOutsidePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .insert_resource(LevelReady(false))
            .init_resource::<TheaterMeshes>()

            //.insert_resource(PathFinder::new())
            //.init_resource::<crate::pause::PauseButtonMaterials>()
            //.add_plugin(AudioPlugin)
            //.add_event::<holdable::LiftHoldableEvent>()
            .add_system_set(
               SystemSet::on_enter(crate::AppState::Loading)
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
               SystemSet::on_update(crate::AppState::InGame)
                    .with_system(player::player_input.system())
                    .with_system(player::player_movement_update.system())
               //.with_system(holdable::lift_holdable.system().label("handle_lift_events"))
            );
    }
}

#[derive(Default)]
pub struct TheaterMeshes {
    pub outside: Handle<Mesh>,
}

fn load_assets(
//    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut theater_meshes: ResMut<TheaterMeshes>,
    mut loading: ResMut<asset_loader::AssetsLoading>,
) {
    println!("Adding theater assets");
    theater_meshes.outside = asset_server.load("models/theater_outside.glb#Mesh0/Primitive0");

    loading.asset_handles.push(theater_meshes.outside.clone_untyped());
}

fn cleanup_environment(
) {
}

struct TheaterOutside { }
fn load_level( 
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut level_ready: ResMut<LevelReady>,
    mut theater_meshes: ResMut<TheaterMeshes>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<State<crate::AppState>>,

) {
    println!("loading level");
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
                    material: materials.add(color.into()),
                    ..Default::default()
                });
            }).id();

    player::spawn_player(&mut commands, &mut materials, &mut meshes, 0, 1, 0);

    level_ready.0 = true;

}

fn set_clear_color(
    mut clear_color: ResMut<ClearColor>,
) {
    clear_color.0 = Color::hex("555555").unwrap();
}

