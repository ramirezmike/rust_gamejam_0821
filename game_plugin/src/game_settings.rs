use bevy::prelude::*;

pub struct GameSettings {
    pub player_speed: f32,
    pub player_friction: f32,
}

pub struct GameSettingsPlugin;
impl Plugin for GameSettingsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(GameSettings {
               player_speed: 3.0,
               player_friction: 0.15,
           })
           .add_system_set(
              SystemSet::on_update(crate::AppState::InGame)
                   .with_system(update_settings.system())
           );
    }
}

fn update_settings(
    mut settings: ResMut<GameSettings>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::M) {
        settings.player_speed += 1.0;
        println!("player_speed {:?}", settings.player_speed);
    }
    if keyboard_input.just_pressed(KeyCode::N) {
        settings.player_speed -= 1.0;
        println!("player_speed {:?}", settings.player_speed);
    }
    if keyboard_input.just_pressed(KeyCode::B) {
        settings.player_friction += 0.1;
        println!("player_friction {:?}", settings.player_friction);
    }
    if keyboard_input.just_pressed(KeyCode::V) {
        settings.player_friction -= 0.1;
        println!("player_friction {:?}", settings.player_friction);
    }
}

