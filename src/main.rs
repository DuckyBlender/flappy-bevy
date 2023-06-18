// FLAPPY BIRD IN RUST

use bevy::prelude::*;
use rand::prelude::*;

const GRAVITY: f32 = 1000.;
const FLAP_VELOCITY: f32 = 400.;

const SCROLL_SPEED: f32 = 100.;
const PIPE_SPAWN_INTERVAL: f32 = 2.;
const PIPE_GAP_HEIGHT: f32 = 125.;

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
    score: usize,
}

#[derive(Component)]
struct ScoreboardText;

// enum BirdAnimation {
//     Downflap,
//     Midflap,
//     Upflap,
// }

#[derive(Component)]
struct Bird {
    velocity: f32,
    // animation: BirdAnimation,
}

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Pipe;

#[derive(Component)]
struct Floor;

#[derive(Component)]
struct StartGameUI;

#[derive(Component)]
struct UI;

#[derive(Resource)]
struct PointSound(Handle<AudioSource>);

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
        .add_systems((check_state,))
        .insert_resource(FlapTimer(Timer::from_seconds(0.1, TimerMode::Repeating))) //todo
        .insert_resource(SpawnTimer(Timer::from_seconds(
            PIPE_SPAWN_INTERVAL,
            TimerMode::Repeating,
        )))
        .insert_resource(Scoreboard { score: 0 })
        .add_system(bevy::window::close_on_esc)
        // Game states
        .add_state::<GameState>()
        // MENU
        .add_systems((wait_for_start, scroll_floor).in_set(OnUpdate(GameState::Menu)))
        // GAME
        .add_systems(
            (bird_physics, pipe_physics, scroll_floor, check_collisions)
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

fn check_state(state: Res<State<GameState>>, scoreboard: Res<Scoreboard>) {
    info!("{:?}: {}.", state.0, scoreboard.score,);
}

fn startup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn start screen
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., 10.)),
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

    // Reset the score
    scoreboard.score = 0;

    // Spawn bird
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(-50., 0., 5.)),
            texture: asset_server.load("sprites/bluebird-downflap.png"),
            ..default()
        },
        Bird { velocity: 0. },
    ));

    // Spawn the scoreboard text
    let font = asset_server.load("fonts/flappyfont.ttf");
    let text_style = TextStyle {
        font,
        font_size: 60.0,
        color: Color::WHITE,
    };
    let text_alignment = TextAlignment::Center;

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(scoreboard.score.to_string(), text_style)
                .with_alignment(text_alignment),
            transform: Transform::from_translation(Vec3::new(0., 200., 20.)),
            ..default()
        },
        UI,
        ScoreboardText,
    ));
}

fn startup_game_over(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn game over screen
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., 11.)),
            texture: asset_server.load("sprites/gameover.png"),
            ..default()
        },
        UI,
    ));
}

fn check_collisions(
    mut state: ResMut<NextState<GameState>>,
    bird_query: Query<(&mut Bird, &Transform)>,
    // pipe_query: Query<(Entity, &Transform), With<Pipe>>,
) {
    // Check if bird collides with the floor or ceiling
    for (_, transform) in bird_query.iter() {
        if transform.translation.y > 256. - 12. || transform.translation.y < -256. + 56. + 12. {
            // 112 is the height of the floor (actually half)
            // 12 is half the height of the bird
            state.set(GameState::GameOver);
        }
    }

    // TODO: Pipe collisions
}

fn bird_physics(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut bird_query: Query<(&mut Bird, &mut Transform)>,
    time: Res<Time>,
) {
    // Check if spacebar or mouse is pressed
    if keyboard_input.just_pressed(KeyCode::Space)
        || mouse_button_input.just_pressed(MouseButton::Left)
    {
        // If so, set bird's velocity to 300
        for (mut bird, _) in bird_query.iter_mut() {
            bird.velocity = FLAP_VELOCITY;
        }
    } else {
        // Otherwise, apply gravity to bird's velocity
        for (mut bird, _) in bird_query.iter_mut() {
            bird.velocity -= GRAVITY * time.delta_seconds();
        }
    }

    // Apply bird's velocity and rotation to transform
    for (bird, mut transform) in bird_query.iter_mut() {
        transform.translation.y += bird.velocity * time.delta_seconds();
        transform.rotation =
            Quat::from_rotation_z((bird.velocity / 3000.).clamp(-0.5, 0.5) * std::f32::consts::PI);
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
) {
    // scroll pipes to the left
    for (entity, mut transform) in pipe_query.iter_mut() {
        transform.translation.x -= SCROLL_SPEED * time.delta_seconds();
        if transform.translation.x < -200. {
            commands.entity(entity).despawn_recursive();
        }
    }

    if timer.0.tick(time.delta()).just_finished() {
        // Increase score
        scoreboard.score += 1;
        // Update the scoreboard text
        for mut text in scoreboard_text.iter_mut() {
            text.sections[0].value = format!("{}", scoreboard.score);
        }

        let mut rng = rand::thread_rng();
        let random_height = rng.gen_range(0.0..256. - PIPE_GAP_HEIGHT);

        // Spawn top pipe
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(
                    196.,
                    random_height + PIPE_GAP_HEIGHT,
                    0.,
                )),
                texture: asset_server.load("sprites/pipe-green.png"),
                sprite: Sprite {
                    flip_y: true,
                    ..default()
                },
                ..default()
            },
            Collider,
            Pipe,
        ));

        // Spawn bottom pipe
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(196., random_height - 320., 0.)),
                texture: asset_server.load("sprites/pipe-green.png"),
                ..default()
            },
            Collider,
            Pipe,
        ));
    }
}

fn scroll_floor(mut floor_query: Query<&mut Transform, With<Floor>>, time: Res<Time>) {
    // Scroll floor to the left
    for mut transform in floor_query.iter_mut() {
        transform.translation.x -= SCROLL_SPEED * time.delta_seconds();
        if transform.translation.x < -144. {
            transform.translation.x = 144.;
        }
    }
}

#[derive(Resource)]
struct FlapTimer(Timer);

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

    // Setup sound
    let point_sound = asset_server.load("audio/point.wav");
    commands.insert_resource(PointSound(point_sound));
}
