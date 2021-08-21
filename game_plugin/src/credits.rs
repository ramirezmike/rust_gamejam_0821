use bevy::prelude::*;

pub struct CreditsEvent {}
pub struct CreditsDelay(pub Timer);

pub fn setup_credits(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands.insert_resource(ClearColor(Color::hex(crate::COLOR_BLACK).unwrap()));
    let window = windows.get_primary_mut().unwrap();
    let width = window.width(); 
    let height = window.height(); 
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(height * 0.35),
                    left: Val::Px(width * 0.25),
                    ..Default::default()
                },
                max_size: Size {
                    width: Val::Px(width / 2.0),
                    height: Val::Undefined,
                    ..Default::default()
                },
                ..Default::default()
            },
            // Use the `Text::with_section` constructor
            text: Text::with_section(
                format!(" "),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    vertical: VerticalAlign::Center,
                },
            ),
            ..Default::default()
        })
        .insert(EndCredits(60.0));
}

pub struct EndCredits(f32);

pub fn update_credits(
    mut commands: Commands,
    mut end_credits: Query<(Entity, &mut EndCredits, &mut Style)>,
    time: Res<Time>,
    mut state: ResMut<State<crate::AppState>>,
    mut windows: ResMut<Windows>,
) {
    let window = windows.get_primary_mut().unwrap();
    let height = window.height(); 

    let mut end_credits_have_ended = false;
    for (_, mut end_credit, mut style) in end_credits.iter_mut() {
        end_credit.0 = end_credit.0 - (time.delta_seconds() * (8592.0 / height));
        style.position.top = Val::Percent(end_credit.0);

        println!("End Credit: {}",end_credit.0);
        if end_credit.0 < -60.0 * (8592.0 / height) {
            end_credits_have_ended = true; 
        }
    }

    if end_credits_have_ended {
        for (entity, _, _) in end_credits.iter_mut() {
            commands.entity(entity).despawn();
        }
        state.set(crate::AppState::MainMenu).unwrap();
    }
}

pub fn show_credits(
    mut credit_event: EventReader<CreditsEvent>,
    mut app_state: ResMut<State<crate::AppState>>
) { 
    if credit_event.iter().count() > 0 {
        app_state.set(crate::AppState::Credits).unwrap();
    }
}
