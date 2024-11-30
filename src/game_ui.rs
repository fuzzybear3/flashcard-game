use bevy::app::App;
use bevy::prelude::*;

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
        Text::new("hello\nplayer!"),
        TextFont {
            // This font is loaded and will be used instead of the default font.
            // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font: asset_server.load("NotoSansJP-Regular.ttf"),
            font_size: 30.0,
            ..default()
        },
        // Set the justification of the Text
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
        TextFeedBack,
    ));

    commands.spawn((
        Text::new("default string"),
        TextFont {
            // This font is loaded and will be used instead of the default font.
            // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font: asset_server.load("NotoSansJP-Regular.ttf"),
            font_size: 50.0,
            ..default()
        },
        // Set the justification of the Text
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            right: Val::Px(5.0),
            ..default()
        },
        StreakCounter,
    ));
}

fn update_text_feedback(
    ui_interface: Res<UiInterface>,
    mut ui_query: Query<&mut Text, With<TextFeedBack>>,
) {
    let mut ui = ui_query.single_mut();
    **ui = ui_interface.text_output.clone();
}

fn update_streak_counter(
    ui_interface: Res<UiInterface>,
    mut ui_query: Query<&mut Text, With<StreakCounter>>,
) {
    let mut ui = ui_query.single_mut();
    **ui = ui_interface.streak.to_string();
}
