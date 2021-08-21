use bevy::{prelude::*,};
use bevy::app::AppExit;
use bevy::app::Events;
//use bevy_prototype_debug_lines::*;
use bevy::reflect::{TypeUuid};
use bevy::window::WindowMode;
use serde::Deserialize;

mod camera;
pub mod asset_loader;
pub mod level_over;
pub mod credits;
pub mod hud_pass;
pub mod game_controller;
pub mod pause;
mod menu;

use camera::*;

pub static COLOR_BLACK: &str = "000000";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    Loading,
    Pause,
    InGame,
    ScoreDisplay,
    LevelTitle,
    ChangingLevel,
    ResetLevel,
    RestartLevel,
    Credits,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugins(DefaultPlugins)
//         .add_plugin(DebugLinesPlugin)
           .init_resource::<menu::ButtonMaterials>()
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
                         .with_system(camera::create_camera.system())
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
           .add_system_set(SystemSet::on_update(AppState::InGame).with_system(credits::show_credits.system()))

          //.add_startup_system(setup.system())
          //.add_system(print_on_load.system())

           .init_resource::<asset_loader::AssetsLoading>()


           .add_system(exit.system());
    }
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
