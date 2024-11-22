use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::render::{
    camera::RenderTarget,
    render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
use bevy::{color::palettes::css::*, pbr::CascadeShadowConfigBuilder, prelude::*};
#[allow(unused_imports)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::f32::consts::PI;

use serde::Deserialize;
use std::fs;

use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Sign {
    image_handle: Handle<Image>,
    ui_id: Entity,
}

#[derive(Component)]
struct DistanceTracker {
    distance_traveled: f32,
    _distance_from_last_sign: f32,
}

const ADVANCE_AMOUNT_PER_STEP: f32 = 0.1;
const SIGN_SPACING_DISTANCE: f32 = 25.;
const NUMBER_OF_SIGNS: u32 = 4;

#[derive(Debug, Deserialize)]
enum Category {
    Adjective,
    Noun,
    Expression,
    Verb,
    Time,
    Question,
    Response,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Translation {
    japanese_word: String,
    english_translation: String,
    category: Category,
    romaji: String,
}

#[derive(Debug, Deserialize, Resource)]
struct Vocabulary {
    translations: Vec<Translation>,
}

impl Vocabulary {
    fn ramdom_translation(&self) -> &Translation {
        // Create a random number generator
        let mut rng = thread_rng();

        // Choose a random element from the vector
        self.translations
            .choose(&mut rng)
            .expect("Vocabulary vec is empty")
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextStyle {
                    // Here we define size of our overlay
                    font_size: 50.0,
                    // We can also change color of the overlay
                    color: Color::srgb(0.0, 1.0, 0.0),
                    // If we want, we can use a custom font
                    font: default(),
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
    let vocabulary = read_translation_file("translations.toml");

    // Chessboard Planetrasnlations
    let black_material = materials.add(Color::BLACK);
    let white_material = materials.add(Color::WHITE);

    let plane_mesh = meshes.add(Plane3d::default().mesh().size(2.0, 2.0));

    for x in -3..600 {
        for z in -2..3 {
            commands.spawn((PbrBundle {
                mesh: plane_mesh.clone(),
                material: if (x + z) % 2 == 0 {
                    black_material.clone()
                } else {
                    white_material.clone()
                },
                transform: Transform::from_xyz(x as f32 * 2.0, -1.0, z as f32 * 2.0),
                ..default()
            },));
        }
    }
    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::srgb_u8(124, 144, 255)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Person,
        DistanceTracker {
            distance_traveled: 0.,
            _distance_from_last_sign: 0.,
        },
    ));

    // light

    //sun
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                // illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
                illuminance: 6_000.,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 5.0, 0.0),
                rotation: Quat::from_rotation_x(-PI / 2.5),

                ..default()
            },
            // The default cascade config is designed to handle large scenes.
            // As this example has a much smaller world, we can tighten the shadow
            // bounds for better visual quality.
            cascade_shadow_config: CascadeShadowConfigBuilder {
                first_cascade_far_bound: 4.0,
                maximum_distance: 10.0,
                ..default()
            }
            .into(),
            ..default()
        },
        Name::new("sun"),
    ));
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                // illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
                illuminance: 2_000.,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 5.0, 0.0),
                rotation: Quat::from_rotation_x(PI / 4.),
                ..default()
            },
            // The default cascade config is designed to handle large scenes.
            // As this example has a much smaller world, we can tighten the shadow
            // bounds for better visual quality.
            cascade_shadow_config: CascadeShadowConfigBuilder {
                first_cascade_far_bound: 4.0,
                maximum_distance: 10.0,
                ..default()
            }
            .into(),
            ..default()
        },
        Name::new("fill"),
    ));

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                // illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
                illuminance: 2_000.,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 5.0, 0.0),
                // rotation: Quat::from_rotation_z(PI / 4.),
                rotation: Quat::from_rotation_y(5.),
                ..default()
            },
            // The default cascade config is designed to handle large scenes.
            // As this example has a much smaller world, we can tighten the shadow
            // bounds for better visual quality.
            cascade_shadow_config: CascadeShadowConfigBuilder {
                first_cascade_far_bound: 4.0,
                maximum_distance: 10.0,
                ..default()
            }
            .into(),
            ..default()
        },
        Name::new("forward_light"),
    ));

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-9.5, 2., 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            // transform: Transform::from_xyz(-9.5, 4.5, 0.5).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        Person,
    ));

    // spawn first sign
    for i in 0..3 {
        spawn_signs(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut images,
            vocabulary.ramdom_translation(),
            vocabulary.ramdom_translation(),
            25. + i as f32 * SIGN_SPACING_DISTANCE,
            &mut asset_server,
        );
    }

    commands.insert_resource(vocabulary);
}

fn move_player(mut query: Query<&mut Transform, With<Person>>) {
    for mut transform in &mut query {
        transform.translation += Vec3 {
            x: ADVANCE_AMOUNT_PER_STEP,
            y: 0.,
            z: 0.,
        };
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
    signs_query: Query<(Entity, &Transform, &Sign)>,
    vocabulary: Res<Vocabulary>,
    mut asset_server: Res<AssetServer>,
) {
    let distance_traveled = query.single().distance_traveled;

    // remove old signs
    let mut removed_sign = false;
    for (entity, transform, sign) in &signs_query {
        if distance_traveled - transform.translation.x > 10. {
            images.remove(sign.image_handle.id());
            commands.entity(entity).despawn_recursive();
            commands.entity(sign.ui_id).despawn_recursive();
            removed_sign = true;
        }
    }
    let spawn_distance = distance_traveled + SIGN_SPACING_DISTANCE * (NUMBER_OF_SIGNS - 1) as f32;
    if removed_sign {
        spawn_signs(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut images,
            vocabulary.ramdom_translation(),
            vocabulary.ramdom_translation(),
            spawn_distance,
            &mut asset_server,
        );
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
) -> Entity {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    // Fill image data with zeroes
    image.resize(size);

    let image_handle = images.add(image);

    // Create a unique camera for rendering each texture with different text
    let texture_camera = commands
        .spawn(Camera2dBundle {
            camera: Camera {
                order: -1,
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            ..default()
        })
        .id();

    // Set up the UI text for the texture
    // let font = asset_server.load("MesloLGS NF Regular.ttf");
    let font = asset_server.load("NotoSansJP-Regular.ttf");
    let ui = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: GOLD.into(),
                ..default()
            },
            TargetCamera(texture_camera),
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                text_content,
                TextStyle {
                    font,
                    font_size: 100.0,
                    color: Color::BLACK,
                    ..default()
                },
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
        .spawn((PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 6.0, 3.0)),
            // material: materials.add(Color::srgb_u8(124, 144, 255)),
            material: material_handle,
            transform,
            ..default()
        },))
        .id();

    // commands.entity(sign_mesh).add_child(ui);
    commands.entity(sign_mesh).insert(Sign {
        image_handle: image_handle.clone(),
        ui_id: ui,
    });
    commands.entity(sign_mesh).add_child(texture_camera);

    // todo I don't know why but if I add ui as a child of sign_mesh, the signs are black.... so
    // for a work around I am storing the id in the Sign component and despawning it manually.
    // commands
    //     .entity(sign_mesh)
    //     .push_children(&[texture_camera, ui]);

    sign_mesh
}

fn spawn_signs(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    translation: &Translation,
    other_translation: &Translation,
    distance: f32,
    asset_server: &mut Res<AssetServer>,
) {
    const SIGN_DISTANCE_FROM_CENTER: f32 = 6.;

    // Left sign
    let transform = Transform::from_xyz(distance, 1.5, -SIGN_DISTANCE_FROM_CENTER)
        .with_rotation(Quat::from_rotation_x(-PI / 2.) * Quat::from_rotation_z(PI / 16.));

    create_sign(
        commands,
        materials,
        images,
        translation.japanese_word.as_str(),
        transform,
        meshes,
        asset_server,
    );

    // Middle sign
    let transform = Transform::from_xyz(distance, 4.5, 0.0)
        .with_rotation(Quat::from_rotation_x(-PI / 2.) * Quat::from_rotation_y(-PI / 16.));

    create_sign(
        commands,
        materials,
        images,
        translation.english_translation.as_str(),
        transform,
        meshes,
        asset_server,
    );

    // Right sign
    let transform = Transform::from_xyz(distance, 1.5, SIGN_DISTANCE_FROM_CENTER)
        .with_rotation(Quat::from_rotation_x(-PI / 2.) * Quat::from_rotation_z(-PI / 16.));

    create_sign(
        commands,
        materials,
        images,
        other_translation.japanese_word.as_str(),
        transform,
        meshes,
        asset_server,
    );
    println!("{}", other_translation.japanese_word);
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
        // println!("Number of Textures: {}", num_textures);
        println!("Number of Images: {}", num_images);
        println!("Number of Meshes: {}", num_meshes);
        println!("Number of Cameras: {}", num_cameras);
        println!("=========================");
    }
}
