use bevy::app::App;
use bevy::prelude::*;

#[derive(Component)]
pub struct TextUi;

pub struct GameUI;

#[derive(Resource)]
pub struct TextOutput(pub String);

// Add the resource to the app
// app.insert_resource(Score(0));

impl Plugin for GameUI {
    fn build(&self, app: &mut App) {
        // app.init_resource::<MyOtherResource>();
        // app.add_event::<MyEvent>();
        app.add_systems(Startup, plugin_init)
            .insert_resource(TextOutput(String::new()));
        app.add_systems(Update, update_ui);
    }
}

fn plugin_init(mut commands: Commands) {
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "hello\nplayer!",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
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
        TextUi,
    ));
}

fn update_ui(text_output: Res<TextOutput>, mut ui_query: Query<&mut Text, With<TextUi>>) {
    let mut ui = ui_query.single_mut();
    ui.sections[0].value = text_output.0.clone();
}
