use bevy::{prelude::*,};
use bevy::app::AppExit;
use bevy::app::Events;
//use bevy_prototype_debug_lines::*;
use bevy::reflect::{TypeUuid};
use bevy::window::WindowMode;
use serde::Deserialize;
use std::collections::HashMap;
use rand::seq::SliceRandom;

mod camera;
pub mod asset_loader;
pub mod level_over;
pub mod credits;
pub mod hud_pass;
pub mod cutscene;
pub mod game_controller;
pub mod pause;
pub mod movie;
pub mod player;
pub mod enemy;
pub mod game_settings;
pub mod level_collision;
pub mod lobby;
pub mod follow_text;
mod menu;
mod theater_outside; 

use camera::*;

pub static COLOR_BLACK: &str = "000000";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    Loading,
    Pause,
    Cutscene,
    InGame,
    Lobby,
    Movie,
    ScoreDisplay,
    LevelTitle,
    ChangingLevel,
    ResetLevel,
    ResetLobby,
    ResetMovie,
    RestartLevel,
    Credits,
}

pub struct GamePlugin;

pub fn get_colors(
) -> Colors { 
    let mut skin_colors = vec!("fefae0", "b08968", "e6ccb2", "7f5539", "c49e85");
    let mut leg_colors = vec!("788aa3", "b2c9ab", "8e9aaf", "ccd5ae");
    let mut torso_colors = vec!("e8e8e4", "efe9ae", "a8dadc", "caffbf", "43aa8b", "006d77", "ef476f");
    let mut hair_colors = vec!("ebcfb2", "424b54", "81583a", "bb4d73", "ff9100");

    let mut rng = rand::thread_rng();
    skin_colors.shuffle(&mut rng);
    leg_colors.shuffle(&mut rng);
    torso_colors.shuffle(&mut rng);
    hair_colors.shuffle(&mut rng);

    let mut rng = rand::thread_rng();
    let mut nums: Vec<i32> = (0..1).collect();
    nums.shuffle(&mut rng);
    let is_long_hair = *nums.last().unwrap() == 0;

    Colors {
        legs: leg_colors.last().unwrap().to_string(),
        torso: torso_colors.last().unwrap().to_string(),
        skin: skin_colors.last().unwrap().to_string(),
        hair: hair_colors.last().unwrap().to_string(),
        is_long_hair
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {


        app.add_plugins(DefaultPlugins)
//         .add_plugin(DebugLinesPlugin)
           .init_resource::<menu::ButtonMaterials>()
           .init_resource::<asset_loader::LevelInfoState>()
           .init_resource::<follow_text::FollowText>()
           .add_event::<ChangeStateEvent>()
           .add_event::<LevelResetEvent>()
           .add_event::<player::DistractEvent>()
           .add_event::<follow_text::FollowTextEvent>()

           .init_resource::<player::PersonMeshes>()
           .add_event::<credits::CreditsEvent>()

//           .add_state(AppState::MainMenu)
           .add_state(AppState::Loading)

           .add_system_set(SystemSet::on_update(AppState::Loading)
                   .with_system(asset_loader::check_assets_ready.system())
           )
           .add_system_set(SystemSet::on_exit(AppState::Loading)
                                //.with_system(fullscreen_app.system())
           )
           .add_system_set(SystemSet::on_enter(AppState::MainMenu))
           .add_system_set(
               SystemSet::on_enter(AppState::MainMenu)
                         .with_system(menu::setup_menu.system())
                         //.with_system(camera::create_camera.system())
           )
           .add_system_set(
               SystemSet::on_update(AppState::MainMenu)
                   .with_system(menu::menu.system())
                   .with_system(game_controller::gamepad_connections.system())
           )
           .add_system_set(
               SystemSet::on_exit(AppState::MainMenu)
                            .with_system(menu::cleanup_menu.system())
           )
           .add_system_set(
               SystemSet::on_enter(AppState::Credits)
               .with_system(credits::setup_credits.system())
           )
           .add_system_set(SystemSet::on_update(AppState::Credits).with_system(credits::update_credits.system()))
           .add_system_set(
               SystemSet::on_update(AppState::InGame)
                   .with_system(credits::show_credits.system())

                   // DEBUG stuff
                   .with_system(level_collision::debug_draw_level_colliders.system())
           )
           .add_system_set(
               SystemSet::on_update(AppState::Lobby)
                   // DEBUG stuff
                   .with_system(level_collision::debug_draw_level_colliders.system())
           )
           .add_system_set(
               SystemSet::on_update(AppState::Movie)
                   // DEBUG stuff
                   .with_system(level_collision::debug_draw_level_colliders.system())
           )
           .add_system(handle_change_state_event.system())
           .add_system(debug_move_entity.system())
           .add_system(handle_level_reset_event.system())
           .add_system(follow_text::update_follow_text.system())
           .add_system(follow_text::handle_follow_text_event.system())
           .add_system(player::player_interact_check.system())
           .add_plugin(theater_outside::TheaterOutsidePlugin)
           .add_plugin(lobby::LobbyPlugin)
           .add_plugin(movie::MoviePlugin)
           .add_plugin(camera::CameraPlugin)
           .add_plugin(game_settings::GameSettingsPlugin)

          //.add_startup_system(setup.system())
          //.add_system(print_on_load.system())

           .init_resource::<asset_loader::AssetsLoading>()
           .insert_resource(GameState {
               current_level: cutscene::Level::Outside,
               mode: Mode::Follow,
               controlling: Kid::A,
               last_positions: HashMap::new(),
               kid_colors: [
                   (Kid::A, get_colors()),
                   (Kid::B, get_colors()),
                   (Kid::C, get_colors()),
                   (Kid::D, get_colors()),
               ].iter().cloned().collect(),
               has_ticket: vec!(),
               has_seen_half_of_movie: false,
               has_avoided_movie_guard: false,
               game_is_done: false,
           })
           .add_asset::<asset_loader::LevelInfo>()
           .init_asset_loader::<asset_loader::LevelsAssetLoader>()

           .add_system(exit.system());
    }
}

pub struct LevelResetEvent;

pub struct GameState {
    pub current_level: cutscene::Level,
    pub mode: Mode, 
    pub controlling: Kid,
    pub last_positions: HashMap<Kid, Option::<Vec3>>,
    pub kid_colors: HashMap<Kid, Colors>,
    pub has_ticket: Vec::<Kid>,
    pub has_seen_half_of_movie: bool,
    pub has_avoided_movie_guard: bool,
    pub game_is_done: bool,
}

#[derive(Clone, PartialEq, Hash, Eq, Debug)]
pub struct Colors {
    pub legs: String,
    pub torso: String,
    pub skin: String,
    pub hair: String,
    pub is_long_hair: bool
}

#[derive(Copy, Clone, PartialEq, Hash, Eq, Debug)]
pub enum Kid {
    A,
    B,
    C,
    D,
}

pub enum Mode {
    Follow,
    Switch
}

fn exit(keys: Res<Input<KeyCode>>, mut exit: ResMut<Events<AppExit>>) {
    if keys.just_pressed(KeyCode::Q) {
        exit.send(AppExit);
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GameObject {
    pub entity: Entity,
    pub entity_type: EntityType
}

impl GameObject {
    pub fn new(entity: Entity, entity_type: EntityType) -> Self {
        GameObject { entity, entity_type }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum EntityType {
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, TypeUuid)]
#[uuid = "939adc56-aa9c-4543-8640-a018b74b5052"] // this needs to be actually generated
pub enum Direction {
    Up, Down, Left, Right, Beneath, Above
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, TypeUuid)]
#[uuid = "93cadc56-aa9c-4543-8640-a018b74b5052"] // this needs to be actually generated
pub struct Position { pub x: i32, pub y: i32, pub z: i32 }
impl Position {
    pub fn from_vec(v: Vec3) -> Position {
        Position {
            x: v.x as i32,
            y: v.y as i32,
            z: v.z as i32,
        }
    }
    pub fn to_vec(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
    pub fn update_from_vec(&mut self, v: Vec3) {
        self.x = v.x as i32;
        self.y = v.y as i32;
        self.z = v.z as i32;
    }
    pub fn matches(&self, v: Vec3) -> bool {
        v.x as i32 == self.x && v.y as i32 == self.y && v.z as i32 == self.z
    }
}

pub fn fullscreen_app(
    mut windows: ResMut<Windows>,
) {
    let window = windows.get_primary_mut().unwrap();
    println!("Setting fullscreen...");
    window.set_maximized(true);
    window.set_mode(WindowMode::BorderlessFullscreen);
}

pub fn lerp(a: f32, b: f32, f: f32) -> f32 {
    return a + f * (b - a);
}

pub struct ChangeStateEvent {
    target: AppState
}

// bevy was breaking when I ended up causing two state changes to happen in a row
// so this hopefully introduces a frame delay?
pub fn handle_change_state_event(
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    mut change_state_event_reader: EventReader<ChangeStateEvent>,
    mut queued_state_change: Local<Option::<AppState>>,
    time: Res<Time>,
    mut delay: Local<f32>,
    player: Query<Entity, With<player::Player>>,
) {
    if let Some(state_change) = &*queued_state_change {
        *delay -= time.delta_seconds();
        if *delay > 0.0 {
            return;
        }

        println!("Actually changing state now");
        state.set(state_change.clone()).unwrap();
        *queued_state_change = None; 
        return;
    }

    if let Some(event) = change_state_event_reader.iter().last() {
        println!("Popping cutscene state");
        state.pop().unwrap();

        // get rid of player to avoid triggering more state changes
        for entity in player.iter() {
            commands.entity(entity).despawn_recursive();
        }

        println!("Setting state change");
        *queued_state_change = Some(event.target.clone());
        *delay = 0.01;
    }
}

pub fn handle_level_reset_event(
    mut level_reset_event_reader: EventReader<LevelResetEvent>,
    mut players: Query<(&mut player::Player, &mut Transform), Without<enemy::Enemy>>,
    mut enemies: Query<(&mut enemy::Enemy, &mut Transform), Without<player::Player>>,
    mut follow_text: ResMut<follow_text::FollowText>,
    mut game_state: ResMut<GameState>,
) {
    for _ in level_reset_event_reader.iter() {
        for (mut player, mut transform) in players.iter_mut() {
            transform.translation = game_state.last_positions[&player.kid].unwrap_or(transform.translation);

            player.velocity = Vec3::default();
            player.movement = None;
            player.is_distracting = None;

            game_state.has_ticket = vec!();
            follow_text.lock = 0.0;
        }
        for (mut enemy, mut transform) in enemies.iter_mut() {
            enemy.is_distracted = false;
            if game_state.current_level == cutscene::Level::Movie {
                transform.translation.x = 0.0;
                transform.translation.z = -9.0;
                enemy.target_waypoint = 0;
            }
        }
    }
}

pub fn debug_move_entity(
    keyboard_input: Res<Input<KeyCode>>,
    mut entities: Query<&mut Transform, With<cutscene::DebugCharacterMarker>>,
    time: Res<Time>,
) {
      return;
    for mut transform in entities.iter_mut() {
        if keyboard_input.pressed(KeyCode::Left) {
            transform.translation.z -= 0.1; 
        }
        if keyboard_input.pressed(KeyCode::Right) {
            transform.translation.z += 0.1; 
        }
        if keyboard_input.pressed(KeyCode::Up) {
            transform.translation.x += 0.1; 
        }
        if keyboard_input.pressed(KeyCode::Down) {
            transform.translation.x -= 0.1; 
        }
        if keyboard_input.pressed(KeyCode::LShift) {
            transform.translation.y -= 0.1; 
        }
        if keyboard_input.pressed(KeyCode::Space) {
            transform.translation.y += 0.1; 
        }
        if keyboard_input.pressed(KeyCode::A) {
            transform.rotate(Quat::from_rotation_y(-time.delta_seconds()));
        }
        if keyboard_input.pressed(KeyCode::D) {
            transform.rotate(Quat::from_rotation_y(time.delta_seconds()));
        }
        if keyboard_input.pressed(KeyCode::W) {
            transform.rotate(Quat::from_rotation_z(time.delta_seconds()));
        }
        if keyboard_input.pressed(KeyCode::S) {
            transform.rotate(Quat::from_rotation_z(-time.delta_seconds()));
        }
        if keyboard_input.pressed(KeyCode::E) {
            transform.rotate(Quat::from_rotation_x(time.delta_seconds()));
        }
        if keyboard_input.pressed(KeyCode::Q) {
            transform.rotate(Quat::from_rotation_x(-time.delta_seconds()));
        }

        if keyboard_input.pressed(KeyCode::Z) {
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
}
