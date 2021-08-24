use bevy::prelude::*;
use crate::{player, asset_loader};
use bevy::render::pipeline::PrimitiveTopology;
use bevy::render::mesh::Indices;

#[derive(Default)]
pub struct EnemyMeshes {
    pub fov_cone: Handle<Mesh>,
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
                SystemSet::on_exit(crate::AppState::InGame)
                    .with_system(cleanup.system())
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

pub fn cleanup(
    mut commands: Commands,
    enemy: Query<Entity, With<Enemy>>,
) {
    for entity in enemy.iter() {
        commands.entity(entity).despawn_recursive();
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
pub fn spawn_enemy(
    commands: &mut Commands, 
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    enemy_meshes: &Res<EnemyMeshes>,

    x: usize,
    y: usize,
    z: usize,
) {
    let color = Color::hex("FF00FF").unwrap(); 
    let color2 = Color::hex("FFFF00").unwrap(); 

    let mut transform = Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32));
    transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));

    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(Enemy {
            })
            .with_children(|parent|  {
                parent.spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0})),
                    material: materials.add(color.into()),
                    transform: Transform::from_xyz(0.0, 0.5, 0.0),
                    ..Default::default()
                });

                let color = Color::rgba(color2.r(), color2.g(), color2.b(), 0.7);

                parent.spawn_bundle(PbrBundle {
                    mesh: enemy_meshes.fov_cone.clone(),
                    material: materials.add(color.into()),
                    visible: Visible {
                        is_visible: true,
                        is_transparent: true,
                    },
                    transform: {
                        let mut t = Transform::from_xyz(VIEW_DISTANCE / 2.0, 0.5, 0.0);
                        //t.scale = Vec3::new(0.5, 1.0, 2.0);
                        t.scale = Vec3::new(VIEW_DISTANCE * 0.8, VIEW_DISTANCE / 2.0, VIEW_DISTANCE * 0.8);
                        t.rotate(Quat::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), std::f32::consts::PI / 4.0));
                        t.rotate(Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), std::f32::consts::PI / 2.0));

                        t
                    },
                    ..Default::default()
                }).insert(Cone {});
            }).id();
}

pub fn update_enemy(
    mut enemies: Query<&mut Transform, With<Enemy>>, 
    time: Res<Time>,
) {
    for mut transform in enemies.iter_mut() {
        transform.rotate(Quat::from_rotation_y(time.delta_seconds()));
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

fn create_triangle() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, 
        vec![
            /*
f 1/1/1 5/2/1 2/3/1
f 2/3/2 5/2/2 3/4/2
f 2/5/3 4/6/3 1/7/3
f 3/4/4 5/2/4 4/8/4
f 4/8/5 5/2/5 1/1/5
f 2/5/3 3/9/3 4/6/3
            */
            [0.000000, -1.000000, -1.000000],
            [0.000000, 1.000000, 0.000000],
            [1.000000, -1.000000, 0.000000],

            [1.000000, -1.000000, 0.000000],
            [0.000000, 1.000000, 0.000000],
            [-0.000000, -1.000000, 1.000000],

            [1.000000, -1.000000, 0.000000],
            [-1.000000, -1.000000, -0.000000],
            [0.000000, -1.000000, -1.000000],

            [-0.000000, -1.000000, 1.000000],
            [0.000000, 1.000000, 0.000000],
            [-1.000000, -1.000000, -0.000000],

            [-1.000000, -1.000000, -0.000000],
            [0.000000, 1.000000, 0.000000],
            [0.000000, -1.000000, -1.000000],

            [1.000000, -1.000000, 0.000000],
            [-0.000000, -1.000000, 1.000000],
            [-1.000000, -1.000000, -0.000000],
        ]);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, 
        vec![
            [0.6667, 0.3333, -0.6667],
            [0.6667, 0.3333, -0.6667],
            [0.6667, 0.3333, -0.6667],

            [0.6667, 0.3333, 0.6667],
            [0.6667, 0.3333, 0.6667],
            [0.6667, 0.3333, 0.6667],

            [0.0000, -1.0000, 0.0000],
            [0.0000, -1.0000, 0.0000],
            [0.0000, -1.0000, 0.0000],

            [-0.6667, 0.3333, 0.6667],
            [-0.6667, 0.3333, 0.6667],
            [-0.6667, 0.3333, 0.6667],

            [-0.6667, 0.3333, -0.6667],
            [-0.6667, 0.3333, -0.6667],
            [-0.6667, 0.3333, -0.6667],

            [0.0000, -1.0000, 0.0000],
            [0.0000, -1.0000, 0.0000],
            [0.0000, -1.0000, 0.0000],

        ]);


    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, 
        vec![
            [0.000000, -1.000000, -1.000000],
            [0.000000, 1.000000, 0.000000],
            [1.000000, -1.000000, 0.000000],
            [1.000000, -1.000000, 0.000000],
            [0.000000, 1.000000, 0.000000],
            [-0.000000, -1.000000, 1.000000],
            [1.000000, -1.000000, 0.000000],
            [-1.000000, -1.000000, -0.000000],
            [0.000000, -1.000000, -1.000000],
            [-0.000000, -1.000000, 1.000000],
            [0.000000, 1.000000, 0.000000],
            [-1.000000, -1.000000, -0.000000],
            [-1.000000, -1.000000, -0.000000],
            [0.000000, 1.000000, 0.000000],
            [0.000000, -1.000000, -1.000000],
            [1.000000, -1.000000, 0.000000],
            [-0.000000, -1.000000, 1.000000],
            [-1.000000, -1.000000, -0.000000],

/*
    vt 0.250000 0.490000
    vt 0.250000 0.250000
    vt 0.490000 0.250000
    vt 0.250000 0.010000
    vt 0.990000 0.250000
    vt 0.510000 0.250000
    vt 0.750000 0.490000
    vt 0.010000 0.250000
    vt 0.750000 0.010000
*/
        ]);
    mesh.set_indices(Some(Indices::U32(
        vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9,
            10, 11, 12,
            13, 14, 15,
            16, 17, 18
        ]
    )));
    // v/t/n
//  f 2//3 3//3 4//3


/*
             [0.000000, -1.000000, -1.000000],
             [1.000000, -1.000000, 0.000000],
             [-0.000000, -1.000000, 1.000000],
             [-1.000000, -1.000000, -0.000000],
             [0.000000, 1.000000, 0.000000],

             [-0.6667, 0.3333, 0.6667],
             [-0.6667, 0.3333, -0.6667],
             [0.6667, 0.3333, -0.6667],
             [0.6667, 0.3333, 0.6667],
             [0.0000, -1.0000, 0.0000],

            [0.000000, -1.000000, -1.000000],
            [0.000000, 1.000000, 0.000000],
            [1.000000, -1.000000, 0.000000],
            [1.000000, -1.000000, 0.000000],
            [0.000000, 1.000000, 0.000000],
            [-0.000000, -1.000000, 1.000000],
            [1.000000, -1.000000, 0.000000],
            [-1.000000, -1.000000, -0.000000],
            [0.000000, -1.000000, -1.000000],
            [-0.000000, -1.000000, 1.000000],
            [0.000000, 1.000000, 0.000000],
            [-1.000000, -1.000000, -0.000000],
            [-1.000000, -1.000000, -0.000000],
            [0.000000, 1.000000, 0.000000],
            [0.000000, -1.000000, -1.000000],
            [1.000000, -1.000000, 0.000000],
            [-0.000000, -1.000000, 1.000000],
            [-1.000000, -1.000000, -0.000000],



            [-0.6667, 0.3333, 0.6667],
            [-0.6667, 0.3333, 0.6667],
            [-0.6667, 0.3333, 0.6667],
            [-0.6667, 0.3333, -0.6667],
            [-0.6667, 0.3333, -0.6667],
            [-0.6667, 0.3333, -0.6667],
            [0.6667, 0.3333, -0.6667],
            [0.6667, 0.3333, -0.6667],
            [0.6667, 0.3333, -0.6667],
            [0.6667, 0.3333, 0.6667],
            [0.6667, 0.3333, 0.6667],
            [0.6667, 0.3333, 0.6667],
            [0.0000, -1.0000, 0.0000],
            [0.0000, -1.0000, 0.0000],
            [0.0000, -1.0000, 0.0000],
            [0.6667, 0.3333, -0.6667],
            [0.6667, 0.3333, -0.6667],
            [0.6667, 0.3333, -0.6667],




*/


    mesh
}

pub fn check_for_player(
    enemies: Query<(&Enemy, &Transform, &Children)>,
    mut cones: Query<&mut Handle<StandardMaterial>, With<Cone>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player: Query<&Transform, With<player::Player>>,
) {
    for (_, transform, children) in enemies.iter() {
        let (axis, mut angle) = transform.rotation.to_axis_angle();
        if axis.y >= -0.0 {
            angle = -angle;
        } 
        let left_angle = angle - VIEW_ANGLE;
        let right_angle = angle + VIEW_ANGLE;

        let left_vector = Vec2::new(left_angle.cos(), left_angle.sin()).normalize() * VIEW_DISTANCE;
        let right_vector = Vec2::new(right_angle.cos(), right_angle.sin()).normalize() * VIEW_DISTANCE;

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

                //println!("TRUE {:?} {:?}", player_position, triangle);
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

pub struct Enemy {}
