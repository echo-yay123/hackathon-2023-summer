//#[cfg(target_os = "macos")]
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin, WindowResolution};

mod game;
mod menu;
mod splash;
//mod client;

const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

// Enum that will be used as a global state for the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Splash,
    Menu,
    Game,
}

// Enum that will be used as a global state for the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum PetOwned {
    #[default]
    Disable,
    Enable,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::NONE))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: ("Super Pet Game").to_string(),
                resolution: WindowResolution::new(800., 600.).with_scale_factor_override(1.0),
                // Setting `transparent` allows the `ClearColor`'s alpha value to take effect
                transparent: true,
                // Disabling window decorations to make it feel more like a widget than a window
                decorations: true,
                //#[cfg(target_os = "macos")]
                //composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
                ..default()
            }),
            ..default()
        }))
        .add_startup_system(setup)
        // Declare the game state, whose starting value is determined by the `Default` trait
        .add_state::<GameState>()
        .add_state::<PetOwned>()
        // Adds the plugins for each state
        .add_plugin(splash::SplashPlugin)
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin)
        //.add_plugin(client::ClientPlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
/*
// Tick the timer, and change state when finished
fn game(
    time: Res<Time>,
    mut game_state: ResMut<NextState<GameState>>,
    mut timer: ResMut<GameTimer>,
) {
    if timer.tick(time.delta()).finished() {
        game_state.set(GameState::Menu);
    }
}
*/
