use bevy::prelude::*;
use bevy::{app::App, asset::saver::AssetSaver};

#[derive(Component)]
struct TextFeedBack;

#[derive(Component)]
struct StreakCounter;

pub struct GameUI;

#[derive(Resource)]
pub struct UiInterface {
    pub text_output: String,
    pub streak: u32,
}

impl Plugin for GameUI {
    fn build(&self, app: &mut App) {
        // app.init_resource::<MyOtherResource>();
        // app.add_event::<MyEvent>();
        app.add_systems(Startup, plugin_init)
            .insert_resource(UiInterface {
                text_output: String::from("Hello"),
                streak: 0,
            });
        app.add_systems(Update, (update_text_feedback, update_streak_counter));
    }
}

fn plugin_init(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "hello\nplayer!",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font: asset_server.load("NotoSansJP-Regular.ttf"),
                font_size: 30.0,
                ..default()
            },
        ) // Set the justification of the Text
        .with_text_justify(JustifyText::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        }),
        TextFeedBack,
    ));

    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "default string",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 50.0,
                ..default()
            },
        ) // Set the justification of the Text
        .with_text_justify(JustifyText::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            right: Val::Px(5.0),
            ..default()
        }),
        StreakCounter,
    ));
}

fn update_text_feedback(
    ui_interface: Res<UiInterface>,
    mut ui_query: Query<&mut Text, With<TextFeedBack>>,
) {
    let mut ui = ui_query.single_mut();
    ui.sections[0].value = ui_interface.text_output.clone();
}

fn update_streak_counter(
    ui_interface: Res<UiInterface>,
    mut ui_query: Query<&mut Text, With<StreakCounter>>,
) {
    let mut ui = ui_query.single_mut();
    ui.sections[0].value = ui_interface.streak.to_string();
}
