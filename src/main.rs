use bevy::render::{
    camera::RenderTarget,
    // render_resource::{
    //     Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    // },
};
use bevy::{color::palettes::css::*, pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    },
    text::FontSmoothing,
    window::PrimaryWindow,
};
#[allow(unused_imports)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::f32::consts::PI;

use serde::Deserialize;
use std::fs;

use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

mod game_ui;
use game_ui::*;

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Sign {
    image_handle: Handle<Image>,
    ui_id: Entity,
}

enum GateState {
    Passed,
    Unpass,
}
#[derive(PartialEq, Debug)]
enum CorrectSide {
    Left,
    Right,
}

#[derive(Component)]
struct Gate {
    word: Word,
    gate_state: GateState,
    correct_side: CorrectSide,
    material_handle: Handle<StandardMaterial>,
}

#[derive(Component)]
struct DistanceTracker {
    distance_traveled: f32,
    _distance_from_last_sign: f32,
}

const ADVANCE_AMOUNT_PER_STEP: f32 = 0.1;
const SIGN_SPACING_DISTANCE: f32 = 25.;
const NUMBER_OF_SIGNS: u32 = 4;

#[derive(Debug, Deserialize, Clone)]
enum Category {
    Adjective,
    Adverb,
    Noun,
    Pronoun, // Add this line
    Verb,
    Time,
    Question,
    Response,
    Conjunction,       // e.g., "and," "but"
    Interjection,      // e.g., "yes," "no," "thank you"
    PreNounAdjectival, // modifiers before nouns, like "that"
    SuruVerb,          // nouns that can be used with "suru" to make verbs (e.g., 勉強する)
}

// enum Category {
//     Adjective,
//     Noun,
//     Expression,
//     Verb,
//     Time,
//     Question,
//     Response,
// }

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
struct FullWord {
    japanese_word: String,
    english_translation: String,
    category: Category,
    romaji: String,
}

#[derive(Debug, Deserialize, Resource)]
struct Vocabulary {
    translations: Vec<FullWord>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
struct Hiragana {
    character: String,
    romaji: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Resource)]
struct HiraganaList {
    hiragana: Vec<Hiragana>,
}

#[derive(Debug, Clone)]
struct Word {
    word: String,
    translation: String,
}

#[derive(Debug, Resource)]
struct WordList {
    words: Vec<Word>,
}

impl WordList {
    #[allow(dead_code)]
    fn ramdom_word(&self) -> &Word {
        // Create a random number generator
        let mut rng = thread_rng();

        // Choose a random element from the vector
        self.words.choose(&mut rng).expect("vec is empty")
    }

    fn ramdom_word_pair(&self) -> (&Word, &Word) {
        let mut rng = thread_rng();

        // Choose two random elements from the vector
        let chosen: Vec<&Word> = self.words.choose_multiple(&mut rng, 2).collect();

        // Unwrap the first two chosen elements or panic if not enough elements
        match chosen.as_slice() {
            [first, second] => (*first, *second),
            _ => panic!("Not enough elements in translations vector"),
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GameUI)
        // .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                enabled: true,
                text_color: Color::srgb(0.0, 1.0, 0.0),
                text_config: TextFont {
                    // Here we define size of our overlay
                    font_size: 42.0,
                    // If we want, we can use a custom font
                    font: default(),
                    // We could also disable font smoothing,
                    font_smoothing: FontSmoothing::default(),
                },
            },
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_player,
                move_distance_marker,
                sign_spawn_manager,
                close_on_esc,
                gate_pass_checker,
                // resource_debug_system,
            ),
        )
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut asset_server: Res<AssetServer>,
) {
    let mut vocabulary = read_translation_file("dictionary/N5_transtlations.toml");
    // let vocabulary = read_translation_file("N5_transtlations.toml");

    let extra_vocabulary = read_translation_file("dictionary/translations.toml");
    vocabulary
        .translations
        .extend(extra_vocabulary.translations);

    let hiragana_list = read_hiragana_file("dictionary/hiragana.toml");

    let mut new_list = WordList { words: Vec::new() };

    // for translation in vocabulary.translations {
    //     new_list.words.push(Word {
    //         word: translation.english_translation.clone(),
    //         translation: translation.romaji.clone(),
    //     });
    // }
    //
    for hiragana in hiragana_list.hiragana {
        new_list.words.push(Word {
            word: hiragana.character.clone(),
            translation: hiragana.romaji.clone(),
        });
    }

    // Chessboard Planetrasnlations
    let black_material = materials.add(Color::BLACK);
    let white_material = materials.add(Color::WHITE);

    let plane_mesh = meshes.add(Plane3d::default().mesh().size(2.0, 2.0));

    for x in -3..150 {
        // for x in -3..1500 {
        for z in -2..3 {
            commands.spawn((
                Mesh3d(plane_mesh.clone()),
                MeshMaterial3d(if (x + z) % 2 == 0 {
                    black_material.clone()
                } else {
                    white_material.clone()
                }),
                Transform::from_xyz(x as f32 * 2.0, -1.0, z as f32 * 2.0),
            ));
        }
    }

    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.8, 0.5, 0.6))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Person,
        DistanceTracker {
            distance_traveled: 0.,
            _distance_from_last_sign: 0.,
        },
    ));

    // light

    //sun
    commands.spawn((
        DirectionalLight {
            // illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            illuminance: 6_000.,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 5.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 2.5),

            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .build(),
        Name::new("sun"),
    ));

    commands.spawn((
        DirectionalLight {
            // illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            illuminance: 2_000.,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 5.0, 0.0),
            rotation: Quat::from_rotation_x(PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .build(),
        Name::new("fill"),
    ));

    commands.spawn((
        DirectionalLight {
            // illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            illuminance: 2_000.,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 5.0, 0.0),
            // rotation: Quat::from_rotation_z(PI / 4.),
            rotation: Quat::from_rotation_y(5.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .build(),
        Name::new("forward_light"),
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-10., 1.5, 0.0).looking_at(Vec3::new(0., 2., 0.), Vec3::Y),
        Person,
    ));

    // spawn first sign
    for i in 0..3 {
        let (main_translation, off_translation) = new_list.ramdom_word_pair();
        spawn_gate(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut images,
            main_translation,
            off_translation,
            25. + i as f32 * SIGN_SPACING_DISTANCE,
            &mut asset_server,
        );
    }

    commands.insert_resource(new_list);
}

fn move_player(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<&mut Transform, With<Person>>,
) {
    for mut transform in &mut query {
        transform.translation += Vec3 {
            x: ADVANCE_AMOUNT_PER_STEP,
            y: 0.,
            z: 0.,
        };
        // let mouse_pos = cursor_moved_events.iter().last();
        let window = q_windows.single();
        if let Some(position) = window.cursor_position() {
            let motion_width = 8.;
            let z = position.x / window.width() * motion_width - motion_width / 2.;
            transform.translation.z = z;
        }
    }
}

fn move_distance_marker(mut query: Query<&mut DistanceTracker>) {
    for mut distnace_tracker in &mut query {
        distnace_tracker.distance_traveled += ADVANCE_AMOUNT_PER_STEP;
    }
}

fn sign_spawn_manager(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    query: Query<&DistanceTracker>,
    // signs_query: Query<(Entity, &Transform, &Sign)>,
    signs_query: Query<&Sign>,
    gate_query: Query<(Entity, &Transform, &Children), With<Gate>>,
    vocabulary: Res<WordList>,
    mut asset_server: Res<AssetServer>,
) {
    let distance_traveled = query.single().distance_traveled;

    // Despawn Gate
    for (entity, gate_transform, children) in &gate_query {
        if distance_traveled - gate_transform.translation.x > 10. {
            for child in children {
                if let Ok(sign) = signs_query.get(*child) {
                    images.remove(sign.image_handle.id());
                    commands.entity(sign.ui_id).despawn_recursive();
                }
            }
            commands.entity(entity).despawn_recursive();
            let (main_translation, off_translation) = vocabulary.ramdom_word_pair();
            let spawn_distance =
                distance_traveled + SIGN_SPACING_DISTANCE * (NUMBER_OF_SIGNS - 1) as f32;
            spawn_gate(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut images,
                main_translation,
                off_translation,
                spawn_distance,
                &mut asset_server,
            );
        }
    }
}

fn create_sign(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    text_content: &str,
    transform: Transform,
    meshes: &mut ResMut<Assets<Mesh>>,
    asset_server: &mut Res<AssetServer>,
    gate_id: &Entity,
) -> Entity {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // let mut image = Image {
    //     texture_descriptor: TextureDescriptor {
    //         label: None,
    //         size,
    //         dimension: TextureDimension::D2,
    //         format: TextureFormat::Bgra8UnormSrgb,
    //         mip_level_count: 1,
    //         sample_count: 1,
    //         usage: TextureUsages::TEXTURE_BINDING
    //             | TextureUsages::COPY_DST
    //             | TextureUsages::RENDER_ATTACHMENT,
    //         view_formats: &[],
    //     },
    //     ..default()
    // };

    // This is the texture that will be rendered to.
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );

    // You need to set these texture usage flags in order to use the image as a render target
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    // // Fill image data with zeroes
    // image.resize(size);

    let image_handle = images.add(image);

    // Create a unique camera for rendering each texture with different text
    let texture_camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: -1,
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
        ))
        .id();

    // Set up the UI text for the texture
    // let font = asset_server.load("MesloLGS NF Regular.ttf");
    let font = asset_server.load("NotoSansJP-Regular.ttf");
    let ui = commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(GOLD.into()),
            TargetCamera(texture_camera),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text_content),
                TextFont {
                    font,
                    font_size: 100.0,
                    ..default()
                },
                TextColor::BLACK,
            ));
        })
        .id();

    // This material has the texture that has been rendered.
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle.clone()),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    // commands.entity(parent_entity).add_child(ui);
    let sign_mesh = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 4.0, 2.5))),
            // material: materials.add(Color::srgb_u8(124, 144, 255)),
            MeshMaterial3d(material_handle),
            transform,
            Name::new("sign"),
        ))
        .id();

    // commands.entity(sign_mesh).add_child(ui);
    commands.entity(sign_mesh).insert(Sign {
        image_handle: image_handle.clone(),
        ui_id: ui,
    });
    commands.entity(sign_mesh).add_child(texture_camera);
    // commands.entity(sign_mesh).add_child(ui);

    // todo I don't know why but if I add ui as a child of sign_mesh, the signs are black.... so
    // for a work around I am storing the id in the Sign component and despawning it manually.
    // commands
    //     .entity(sign_mesh)
    //     .push_children(&[texture_camera, ui]);

    commands.entity(*gate_id).add_child(sign_mesh);
    // commands.entity(*gate_id).add_child(sign_mesh);

    sign_mesh
}

fn spawn_gate(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    word: &Word,
    other_word: &Word,
    distance: f32,
    asset_server: &mut Res<AssetServer>,
) {
    const SIGN_DISTANCE_FROM_CENTER: f32 = 4.;
    let sign_distance_from_gate = 3.;

    let mut rng = rand::thread_rng();
    let correct_side = if rng.gen_bool(0.5) {
        // 50% chance for each side
        CorrectSide::Left
    } else {
        CorrectSide::Right
    };

    let (left_translation, right_translation) = match correct_side {
        CorrectSide::Left => (word, other_word),
        CorrectSide::Right => (other_word, word),
    };

    let gate_material_handle = materials.add(Color::srgb_u8(50, 50, 50));
    let gate_id = commands
        .spawn((
            Mesh3d(meshes.add(Cylinder::new(0.2, 2.5))),
            MeshMaterial3d(gate_material_handle.clone()),
            Transform::from_xyz(distance, -0.5, 0.0),
            Gate {
                word: word.to_owned(),
                gate_state: GateState::Unpass,
                correct_side,
                material_handle: gate_material_handle,
            },
        ))
        .id();

    let (left_text, right_text) = (
        left_translation.translation.as_str(),
        right_translation.translation.as_str(),
    );

    let transform = Transform::from_xyz(sign_distance_from_gate, 1.5, -SIGN_DISTANCE_FROM_CENTER)
        .with_rotation(Quat::from_rotation_x(-PI / 2.) * Quat::from_rotation_z(PI / 16.));

    create_sign(
        commands,
        materials,
        images,
        left_text,
        transform,
        meshes,
        asset_server,
        &gate_id,
    );

    // Middle sign
    let transform = Transform::from_xyz(sign_distance_from_gate, 4.5, 0.0)
        .with_rotation(Quat::from_rotation_x(-PI / 2.) * Quat::from_rotation_y(-PI / 16.));

    create_sign(
        commands,
        materials,
        images,
        word.word.as_str(),
        transform,
        meshes,
        asset_server,
        &gate_id,
    );

    // Right sign
    let transform = Transform::from_xyz(sign_distance_from_gate, 1.5, SIGN_DISTANCE_FROM_CENTER)
        .with_rotation(Quat::from_rotation_x(-PI / 2.) * Quat::from_rotation_z(-PI / 16.));

    create_sign(
        commands,
        materials,
        images,
        right_text,
        transform,
        meshes,
        asset_server,
        &gate_id,
    );
}

fn gate_pass_checker(
    mut query: Query<(&Transform, &mut Gate)>,
    single_query: Query<&DistanceTracker>,
    player_query: Query<&Transform, With<Person>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ui_interface: ResMut<UiInterface>,
) {
    for (transform, mut gate) in &mut query {
        match gate.gate_state {
            GateState::Passed => {}
            GateState::Unpass => {
                let distance_traveled = single_query.single().distance_traveled;
                let player_trastform = player_query.iter().last().unwrap();
                if distance_traveled >= transform.translation.x {
                    let player_side = if player_trastform.translation.z > 0. {
                        CorrectSide::Right
                    } else {
                        CorrectSide::Left
                    };

                    if gate.correct_side == player_side {
                        ui_interface.text_output = format!(
                            "Correct: \"{}\" => \"{}\"",
                            gate.word.word, gate.word.translation
                        );

                        ui_interface.streak += 1;

                        if let Some(material) = materials.get_mut(&gate.material_handle) {
                            material.base_color = Color::srgb(0.2, 0.8, 0.2);
                        }
                    } else {
                        ui_interface.text_output = format!(
                            "Incorrect: \"{}\" => \"{}\"",
                            gate.word.word, gate.word.translation
                        );

                        ui_interface.streak = 0;

                        if let Some(material) = materials.get_mut(&gate.material_handle) {
                            material.base_color = Color::srgb(0.8, 0.2, 0.2);
                        }
                    }
                    gate.gate_state = GateState::Passed;
                }
            }
        }
    }
}

pub fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}

fn read_translation_file(file_name: &str) -> Vocabulary {
    // Read the TOML file
    let content = fs::read_to_string(file_name).expect("could not read translation file");

    // Parse the TOML content
    toml::from_str(&content).expect("could not parse vocab file")
}

fn read_hiragana_file(file_name: &str) -> HiraganaList {
    // Read the TOML file
    let content = fs::read_to_string(file_name).expect("could not read translation file");

    // Parse the TOML content
    toml::from_str(&content).expect("could not parse vocab file")
}

#[allow(dead_code)]
fn resource_debug_system(
    entities: Query<Entity>,
    images: Res<Assets<Image>>,
    meshes: Res<Assets<Mesh>>,
    cameras: Query<Entity, With<Camera>>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    // Initialize the timer on first run
    if timer.finished() {
        timer.set_duration(std::time::Duration::from_secs(3));
        timer.reset();
    }
    // Set up a timer to print this information every 3 seconds
    if timer.tick(time.delta()).finished() {
        let num_entities = entities.iter().count();
        // let num_textures = textures.len();
        let num_images = images.len();
        let num_meshes = meshes.len();
        let num_cameras = cameras.iter().count();

        // Print debug information
        println!("=== Debug Information ===");
        println!("Total entities: {}", num_entities);
        println!("Number of Images: {}", num_images);
        println!("Number of Meshes: {}", num_meshes);
        println!("Number of Cameras: {}", num_cameras);
        println!("=========================");
    }
}
