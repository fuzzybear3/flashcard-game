use bevy::text::{Text, TextStyle};
use bevy::{color::palettes::css::*, pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy::{
    // color::palettes::css::GOLD,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
};

use std::f32::consts::PI;

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Sign;

#[derive(Component)]
struct DistanceTracker {
    distance_traveled: f32,
    distance_from_last_sign: f32,
}

const ADVANCE_AMOUNT_PER_STEP: f32 = 0.1;
const SIGN_SPACING_DISTANCE: f32 = 15.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (move_player, move_distance_marker, sign_spawn_manager),
        )
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Chessboard Plane
    let black_material = materials.add(Color::BLACK);
    let white_material = materials.add(Color::WHITE);

    let plane_mesh = meshes.add(Plane3d::default().mesh().size(2.0, 2.0));

    for x in -3..200 {
        for z in -3..4 {
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
            distance_from_last_sign: 0.,
        },
    ));

    // light
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                intensity: 2_000_000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        },
        Person,
    ));

    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                intensity: 2_000_000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(20.0, 8.0, 4.0),
            ..default()
        },
        Person,
    ));

    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                intensity: 1_100_000_000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(20.0, 20.0, 4.0),
            ..default()
        },
        Person,
    ));

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-9.5, 3.5, 0.5).looking_at(Vec3::ZERO, Vec3::Y),
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
            15. + i as f32 * SIGN_SPACING_DISTANCE,
        );
    }
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
    signs_query: Query<(Entity, &Transform), With<Sign>>,
) {
    let distance_traveled = query.single().distance_traveled;
    let mut last_sign = signs_query.iter().last().unwrap();
    let mut first_sign = signs_query.iter().last().unwrap();

    for sign in &signs_query {
        if sign.1.translation.x < last_sign.1.translation.x {
            last_sign = sign;
        }
        if sign.1.translation.x > first_sign.1.translation.x {
            first_sign = sign;
        }
    }
    println!(
        "distance traveled: {}",
        distance_traveled - last_sign.1.translation.x
    );
    if (distance_traveled - last_sign.1.translation.x) > 10. {
        commands.entity(last_sign.0).despawn();
        spawn_signs(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut images,
            first_sign.1.translation.x + SIGN_SPACING_DISTANCE,
        );
    }
}

fn spawn_signs(
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    mut images: &mut ResMut<Assets<Image>>,
    distance: f32,
) {
    const SIGN_SPACING: f32 = 6.;

    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // This is the texture that will be rendered to.
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

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);

    // Light
    commands.spawn(DirectionalLightBundle::default());

    let texture_camera = commands
        .spawn(Camera2dBundle {
            camera: Camera {
                // render before the "main pass" camera
                order: -1,
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            ..default()
        })
        .id();

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    // Cover the whole image
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
                "This is a cube",
                TextStyle {
                    font_size: 40.0,
                    color: Color::BLACK,
                    ..default()
                },
            ));
        });

    // This material has the texture that has been rendered.
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        reflectance: 0.02,
        unlit: false,

        ..default()
    });

    // Spawn a cube mesh that is scaled to be flat along one axis
    let sign_left = commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(1.0, 4.0, 8.0)),
                // material: materials.add(Color::srgb_u8(124, 144, 255)),
                material: material_handle,
                transform: Transform::from_xyz(distance, 1.5, -SIGN_SPACING),
                ..default()
            },
            Sign,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "This is a cube",
                TextStyle {
                    font_size: 40.0,
                    color: Color::BLACK,
                    ..default()
                },
            ));
        });
    // let sign_right = commands
    //     .spawn((
    //         PbrBundle {
    //             mesh: meshes.add(Cuboid::new(1.0, 4.0, 8.0)),
    //             material: materials.add(Color::srgb_u8(124, 144, 255)),
    //             transform: Transform::from_xyz(distance, 1.5, SIGN_SPACING),
    //             ..default()
    //         },
    //         Sign,
    //     ))
    //     .id();
}
