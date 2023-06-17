// FLAPPY BIRD IN RUST

use bevy::prelude::*;

enum GameState {
    Menu,
    Playing,
    GameOver,
}

enum BirdAnimation {
    Downflap,
    Midflap,
    Upflap,
}

#[derive(Component)]
struct Bird {
    velocity: f32,
    animation: BirdAnimation,
}

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Pipe;

#[derive(Component)]
struct Floor;

fn main() {
    let window = WindowPlugin {
        primary_window: Some(Window {
            title: "Flappy Bevy".into(),
            resolution: (288., 512.).into(),
            ..default()
        }),
        ..default()
    };

    App::new()
        .add_plugins(DefaultPlugins.set(window))
        .add_startup_system(setup)
        .add_systems((bird_physics, scroll_colliders, spawn_pipe))
        .run();
}

fn bird_physics(
    keyboard_input: Res<Input<KeyCode>>,
    mut bird_query: Query<(&mut Bird, &mut Transform)>,
    // collider_query: Query<&Transform, With<Collider>>,
    time: Res<Time>,
) {
    // Todo - Add collision detection
    // For now, just check if bird is touching ground or ceiling
    for (mut bird, mut transform) in bird_query.iter_mut() {
        if transform.translation.y > 256. {
            bird.velocity = 0.;
            transform.translation.y = 256.;
        } else if transform.translation.y < -256. {
            bird.velocity = 0.;
            transform.translation.y = -256.;
        }
    }
    // Check if spacebar is pressed
    if keyboard_input.just_pressed(KeyCode::Space) {
        // If so, set bird's velocity to 300
        for (mut bird, _) in bird_query.iter_mut() {
            bird.velocity = 300.;
        }
    } else {
        // Otherwise, apply gravity to bird's velocity
        for (mut bird, _) in bird_query.iter_mut() {
            bird.velocity -= 300. * time.delta_seconds();
        }
    }

    // Apply bird's velocity and rotation to transform
    for (bird, mut transform) in bird_query.iter_mut() {
        transform.translation.y += bird.velocity * time.delta_seconds();
        transform.rotation =
            Quat::from_rotation_z((bird.velocity / 3000.).clamp(-0.5, 0.5) * std::f32::consts::PI);
    }
}

fn spawn_pipe(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn top pipe with collider component
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(144., 0., 2.)),
            texture: asset_server.load("sprites/pipe-green.png"),
            ..default()
        },
        Collider,
        Pipe,
    ));

    // Spawn bottom pipe with collider component
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(144., -320., 2.)),
            texture: asset_server.load("sprites/pipe-green.png"),
            ..default()
        },
        Collider,
        Pipe,
    ));
}

fn scroll_colliders(
    mut colliders_query: Query<&mut Transform, With<Collider>>,
    time: Res<Time>,
    commands: Commands,
) {
    // todo: make timer system
    // Scroll floor to the left
    for mut transform in colliders_query.iter_mut() {
        transform.translation.x -= 100. * time.delta_seconds();
        if transform.translation.x < -144. {
            transform.translation.x = 144.;
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn camera
    commands.spawn(Camera2dBundle::default());

    // Spawn background sprite
    commands.spawn(SpriteBundle {
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        texture: asset_server.load("sprites/background-day.png"),
        ..default()
    });

    // Spawn two ground sprites with collider component
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0., -256., 1.)),
            texture: asset_server.load("sprites/base.png"),
            ..default()
        },
        Collider,
        Floor,
    ));
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(144., -256., 1.)),
            texture: asset_server.load("sprites/base.png"),
            ..default()
        },
        Collider,
        Floor,
    ));

    // Spawn bird sprite with bird component
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(-50., 0., 5.)),
            texture: asset_server.load("sprites/bluebird-midflap.png"),
            ..default()
        },
        Bird {
            velocity: 0.,
            animation: BirdAnimation::Midflap,
        },
    ));
}
