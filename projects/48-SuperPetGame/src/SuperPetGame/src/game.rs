
use super::{despawn_screen, GameState, PetOwned,TEXT_COLOR};
// #[cfg(target_os = "macos")]
use bevy::prelude::*;
// This plugin will contain the game. In this case, it's just be a screen that will
// display the current settings for 5 seconds before returning to the menu

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum PlayMenuState {
    Show,
    FeedMenu,
    #[default]
    Disable,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<PlayMenuState>()
            .add_systems((
                game_setup.in_schedule(OnEnter(GameState::Game)),
                //.run_if(in_state(PetOwned::Enable)),
                despawn_screen::<OnGameScreen>.in_schedule(OnExit(GameState::Game)),
            ))
            .add_systems((
                play_menu_show.run_if(in_state(GameState::Game)),
                play_menu_setup.in_schedule(OnEnter(PlayMenuState::Show)),
                despawn_screen::<OnPlayMenuScreen>.in_schedule(OnExit(PlayMenuState::Show)),
            ))
            .add_systems((play_menu_action, button_system).in_set(OnUpdate(PlayMenuState::Show)));
    }
}

// Tag component used to tag entities added on the game screen
#[derive(Component)]
struct OnGameScreen;

#[derive(Component)]
struct OnPlayMenuScreen;

#[derive(Resource, Deref, DerefMut)]
struct GameTimer(Timer);

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
enum PlayMenuButtonAction {
    //PlayMenu,
    //FeedMenu,
    //WakeUpPet, //Wake up pet
    //SleepPet,//Make pet sleep
    //IdlePet, //Make pet into idle situation
    BackToMain,
}

fn game_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("../assets/textures/turtle-front2.png"),
            ..default()
        },
        OnGameScreen,
    ));

    // Spawn a 5 seconds timer to trigger going back to the menu
    commands.insert_resource(GameTimer(Timer::from_seconds(5.0, TimerMode::Once)));
}

// Tag component used to mark which setting is currently selected
#[derive(Component)]
struct SelectedOption;

// This system handles changing all buttons color based on mouse interaction
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, selected) in &mut interaction_query {
        *color = match (*interaction, selected) {
            (Interaction::Clicked, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

fn play_menu_show(
    mouse_button_input: Res<Input<MouseButton>>,
    mut menu_state: ResMut<NextState<PlayMenuState>>,
    //game_state: ResMut<State<GameState>>,
) {
    if mouse_button_input.pressed(MouseButton::Right) {
        menu_state.set(PlayMenuState::Show);
    }
}

fn play_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    // Common style for all buttons on the screen
    let button_style = Style {
        size: Size::new(Val::Px(86.0), Val::Px(40.0)),
        margin: UiRect {
            left: Val::Px(7.0),
            right: Val::Px(7.0),
            top: Val::Px(8.0),
            bottom: Val::Px(8.0),
        },
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let _button_icon_style = Style {
        size: Size::new(Val::Px(30.0), Val::Auto),
        // This takes the icons out of the flexbox flow, to be positioned exactly
        position_type: PositionType::Absolute,
        // The icon will be close to the left border of the button
        position: UiRect {
            left: Val::Px(10.0),
            right: Val::Auto,
            top: Val::Auto,
            bottom: Val::Auto,
        },
        ..default()
    };
    let button_text_style = TextStyle {
        font: font.clone(),
        font_size: 25.0,
        color: TEXT_COLOR,
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            OnPlayMenuScreen,
        ))
        //Show five botton
        //Feed
        //Game
        //Wake or sleep
        //Idle
        //Back to main
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        position: UiRect {
                            left: Val::Px(0.),
                            top: Val::Px(165.),
                            ..default()
                        },
                        size: Size {
                            width: Val::Px(500.),
                            height: Val::Px(56.),
                        },
                        ..default()
                    },

                    background_color: Color::ORANGE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            PlayMenuButtonAction::BackToMain,
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn(TextBundle::from_section("Feed", button_text_style.clone()));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            PlayMenuButtonAction::BackToMain,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Wake Up",
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            PlayMenuButtonAction::BackToMain,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Sleep",
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            PlayMenuButtonAction::BackToMain,
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn(TextBundle::from_section("Idle", button_text_style.clone()));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            PlayMenuButtonAction::BackToMain,
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn(TextBundle::from_section("Back", button_text_style.clone()));
                        });
                });
        });
}

fn play_menu_action(
    interaction_query: Query<
        (&Interaction, &PlayMenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut play_menu_state: ResMut<NextState<PlayMenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Clicked {
            match menu_button_action {
                // PlayMenuButtonAction::Quit => app_exit_events.send(AppExit),
                //PlayMenuButtonAction::FeedMenu => play_menu_state.set(PlayMenuState::FeedMenu),
                //PlayMenuButtonAction::WakeUpPet => play_menu_state.set(PlayMenuState::FeedMenu),
                //PlayMenuButtonAction::SleepPet => play_menu_state.set(PlayMenuState::FeedMenu),
                //PlayMenuButtonAction::IdlePet => play_menu_state.set(PlayMenuState::FeedMenu),
                PlayMenuButtonAction::BackToMain => {
                    //exit play menu
                    play_menu_state.set(PlayMenuState::Disable);
                    game_state.set(GameState::Menu);
                }
            }
        }
    }
}
