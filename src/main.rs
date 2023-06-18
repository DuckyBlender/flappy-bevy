#![allow(clippy::too_many_arguments)] // in some functions there are 8 arguments, I don't think it's too much and idk how to reduce it]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // don't open console window in release mode

use bevy::prelude::*;
use rand::prelude::*;

const GRAVITY: f32 = 1000.;
const FLAP_VELOCITY: f32 = 400.;

const SCROLL_SPEED: f32 = 100.;
const PIPE_SPAWN_INTERVAL: f32 = 2.;
const PIPE_GAP_HEIGHT: f32 = 125.;

const SCALE: f32 = 1.0;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

// This resource tracks the game's score
#[derive(Resource)]
struct Scoreboard {
    score: isize,
}

#[derive(Component)]
struct ScoreboardText;

#[derive(Component)]
struct Bird {
    velocity: f32,
    // animation: BirdAnimation,
}

#[derive(Component)]
struct Pipe;

#[derive(Component)]
struct Floor;

#[derive(Component)]
struct StartGameUI;

#[derive(Component)]
struct UI;

fn main() {
    let window = WindowPlugin {
        primary_window: Some(Window {
            title: "Flappy Bevy".into(),
            resolution: (288. * SCALE, 512. * SCALE).into(),
            resizable: false,
            mode: bevy::window::WindowMode::Windowed,
            ..default()
        }),
        ..default()
    };

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(window)
                .set(ImagePlugin::default_nearest()),
        )
        .add_startup_system(setup)
        // .add_systems((check_state,)) // Debugging
        .insert_resource(FlapTimer(Timer::from_seconds(0.1, TimerMode::Repeating))) //todo
        .insert_resource(SpawnTimer(Timer::from_seconds(
            PIPE_SPAWN_INTERVAL,
            TimerMode::Repeating,
        )))
        .insert_resource(Scoreboard { score: -1 })
        .add_system(bevy::window::close_on_esc)
        // Game states
        .add_state::<GameState>()
        // MENU
        .add_systems((wait_for_start, scroll_floor).in_set(OnUpdate(GameState::Menu)))
        // GAME
        .add_systems(
            (
                bird_physics,
                pipe_physics,
                scroll_floor,
                check_collisions,
                animate_bird,
            )
                .in_set(OnUpdate(GameState::Playing)),
        )
        // GAME OVER
        .add_systems((wait_for_start,).in_set(OnUpdate(GameState::GameOver)))
        // WHEN STARTING MENU
        .add_system(startup_menu.in_schedule(OnEnter(GameState::Menu)))
        // WHEN STARTING GAME
        .add_system(startup_game.in_schedule(OnEnter(GameState::Playing)))
        // WHEN STARTING GAME OVER
        .add_system(startup_game_over.in_schedule(OnEnter(GameState::GameOver)))
        // WHEN EXITING MENU OR GAME OVER
        .add_system(close_menu.in_schedule(OnExit(GameState::Menu)))
        .add_system(close_menu.in_schedule(OnExit(GameState::GameOver)))
        .run();
}

// Debugging function
// fn check_state(state: Res<State<GameState>>, scoreboard: Res<Scoreboard>) {
//     info!("{:?}: {}.", state.0, scoreboard.score,);
// }

fn startup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn start screen
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., 10.))
                .with_scale(Vec3::splat(SCALE)),
            texture: asset_server.load("sprites/message.png"),
            ..default() // Add a component that describes the UI element
        },
        StartGameUI,
        UI,
    ));
}

fn wait_for_start(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space)
        || mouse_button_input.just_pressed(MouseButton::Left)
    {
        state.set(GameState::Playing);
    }
}

fn close_menu(mut commands: Commands, query: Query<Entity, With<UI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn startup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    start_menu: Query<Entity, With<StartGameUI>>,
    pipes_query: Query<Entity, With<Pipe>>,
    bird_query: Query<Entity, With<Bird>>,
    scoreboard_query: Query<Entity, With<ScoreboardText>>,
    mut scoreboard: ResMut<Scoreboard>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // Close the start menu
    for entity in start_menu.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Remove all pipes and birds from the previous game (if any)
    for entity in pipes_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in bird_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in scoreboard_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Reset the score (-1 because no pipe is in the beginning)
    scoreboard.score = -1;

    // Setup texture atlas for bird animation
    let texture_name = match rand::thread_rng().gen_range(0..=2) {
        0 => "sprites/red.png",
        1 => "sprites/blue.png",
        2 => "sprites/yellow.png",
        _ => unreachable!(),
    };
    let texture_handle = asset_server.load(texture_name);
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(34., 24.), 4, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let animation_indices = AnimationIndices { first: 0, last: 3 };

    // Spawn bird
    commands.spawn((
        SpriteSheetBundle {
            // scale with SCALE
            transform: Transform::from_translation(Vec3::new(-50., 0., 5.))
                .with_scale(Vec3::splat(SCALE)),
            texture_atlas: texture_atlas_handle,
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.3, TimerMode::Repeating)),
        Bird { velocity: 0. },
    ));

    // Spawn the scoreboard text
    let font = asset_server.load("fonts/flappyfont.ttf");
    let text_style = TextStyle {
        font,
        font_size: 60.0 * SCALE,
        color: Color::WHITE,
    };
    let text_alignment = TextAlignment::Center;

    commands.spawn((
        Text2dBundle {
            text: Text::from_section("0", text_style).with_alignment(text_alignment),
            transform: Transform::from_translation(Vec3::new(0., 200. * SCALE, 20.)),
            ..default()
        },
        UI,
        ScoreboardText,
    ));
}

fn startup_game_over(mut commands: Commands, asset_server: Res<AssetServer>, sound: Res<Audio>) {
    // Play the hit sound
    sound.play(asset_server.load("audio/hit.ogg"));
    // Spawn game over screen
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., 11.))
                .with_scale(Vec3::splat(SCALE)),
            texture: asset_server.load("sprites/gameover.png"),
            ..default()
        },
        UI,
    ));
}

fn check_collisions(
    mut state: ResMut<NextState<GameState>>,
    bird_query: Query<(&mut Bird, &Transform)>,
    pipe_query: Query<(Entity, &Transform), With<Pipe>>,
) {
    // Check if bird collides with the floor or ceiling
    for (_, transform) in bird_query.iter() {
        if transform.translation.y > 256. * SCALE - 12. * SCALE
            || transform.translation.y < -256. * SCALE + 56. * SCALE + 12. * SCALE
        {
            // 112 is the height of the floor (actually half)
            // 12 is half the height of the bird
            state.set(GameState::GameOver);
        }
    }

    // Pipe collision
    for (_, pipe_transform) in pipe_query.iter() {
        for (_, bird_transform) in bird_query.iter() {
            // Check if bird collides with pipe
            if collide(
                bird_transform.translation,
                Vec2::new(17. * SCALE, 12. * SCALE),
                pipe_transform.translation,
                Vec2::new(52. * SCALE, 320. * SCALE),
            ) {
                // If so, game over
                state.set(GameState::GameOver);
            }
        }
    }
}

fn collide(pos1: Vec3, size1: Vec2, pos2: Vec3, size2: Vec2) -> bool {
    let x_overlap = (pos1.x - pos2.x).abs() < (size1.x + size2.x) / 2.;
    let y_overlap = (pos1.y - pos2.y).abs() < (size1.y + size2.y) / 2.;
    x_overlap && y_overlap
}

fn bird_physics(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut bird_query: Query<(&mut Bird, &mut Transform)>,
    time: Res<Time>,
    sound: Res<Audio>,
    asset_server: Res<AssetServer>,
) {
    // Check if spacebar or mouse is pressed
    if keyboard_input.just_pressed(KeyCode::Space)
        || mouse_button_input.just_pressed(MouseButton::Left)
    {
        // If so, set bird's velocity to 300
        for (mut bird, _) in bird_query.iter_mut() {
            bird.velocity = FLAP_VELOCITY * SCALE;
        }
        // Play sound
        sound.play(asset_server.load("audio/wing.ogg"));
    } else {
        // Otherwise, apply gravity to bird's velocity
        for (mut bird, _) in bird_query.iter_mut() {
            bird.velocity -= GRAVITY * SCALE * time.delta_seconds();
        }
    }

    // Apply bird's velocity and rotation to transform
    for (bird, mut transform) in bird_query.iter_mut() {
        transform.translation.y += bird.velocity * time.delta_seconds();
        transform.rotation = Quat::from_rotation_z(
            (bird.velocity / 3000. / SCALE).clamp(-0.5, 0.5) * std::f32::consts::PI,
        );
    }
}

#[derive(Resource)]
struct SpawnTimer(Timer);

fn pipe_physics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
    mut pipe_query: Query<(Entity, &mut Transform), With<Pipe>>,
    mut scoreboard: ResMut<Scoreboard>,
    mut scoreboard_text: Query<&mut Text, With<ScoreboardText>>,
    audio: Res<Audio>,
) {
    // scroll pipes to the left
    for (entity, mut transform) in pipe_query.iter_mut() {
        transform.translation.x -= SCROLL_SPEED * SCALE * time.delta_seconds();
        if transform.translation.x < -200. * SCALE {
            commands.entity(entity).despawn_recursive();
        }
    }

    if timer.0.tick(time.delta()).just_finished() {
        // Increase score
        scoreboard.score += 1;
        // Play sound if score is positive
        if scoreboard.score > 0 {
            audio.play(asset_server.load("audio/point.ogg"));
        }
        // Update the scoreboard text
        for mut text in scoreboard_text.iter_mut() {
            let score = if scoreboard.score < 0 {
                0
            } else {
                scoreboard.score
            };
            text.sections[0].value = format!("{score}");
        }

        let mut rng = rand::thread_rng();
        let random_height = rng.gen_range(0.0..256. * SCALE - PIPE_GAP_HEIGHT * SCALE);

        // Spawn top pipe
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(
                    180. * SCALE,
                    random_height + (PIPE_GAP_HEIGHT * SCALE),
                    0.,
                ))
                .with_scale(Vec3::splat(SCALE)),
                texture: asset_server.load("sprites/pipe-green.png"),
                sprite: Sprite {
                    flip_y: true,
                    ..default()
                },
                ..default()
            },
            Pipe,
        ));

        // Spawn bottom pipe
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(
                    180. * SCALE,
                    random_height - (320. * SCALE),
                    0.,
                ))
                .with_scale(Vec3::splat(SCALE)),
                texture: asset_server.load("sprites/pipe-green.png"),
                ..default()
            },
            Pipe,
        ));
    }
}

fn scroll_floor(mut floor_query: Query<&mut Transform, With<Floor>>, time: Res<Time>) {
    // Scroll floor to the left
    for mut transform in floor_query.iter_mut() {
        transform.translation.x -= SCROLL_SPEED * SCALE * time.delta_seconds();
        if transform.translation.x < -144. * SCALE {
            transform.translation.x = 144. * SCALE;
        }
    }
}

#[derive(Resource)]
struct FlapTimer(Timer);

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_bird(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            // Loop through the textures in the animation
            if sprite.index < indices.last {
                sprite.index += 1;
            } else {
                sprite.index = indices.first;
            }
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn camera
    commands.spawn(Camera2dBundle::default());

    // Spawn background sprite
    commands.spawn(SpriteBundle {
        transform: Transform::from_translation(Vec3::new(0., 0., 0.))
            .with_scale(Vec3::splat(SCALE)),
        texture: asset_server.load("sprites/background-day.png"),
        ..default()
    });

    // Spawn scale amount of ground sprites + 1 with collider component
    for i in 0..=SCALE as usize {
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(
                    (i * 144) as f32 * SCALE,
                    -256. * SCALE,
                    1.,
                ))
                .with_scale(Vec3::splat(SCALE)),
                texture: asset_server.load("sprites/base.png"),
                ..default()
            },
            Floor,
        ));
    }
}
