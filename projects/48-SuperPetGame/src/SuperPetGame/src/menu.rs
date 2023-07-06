use bevy::{app::AppExit, prelude::*};

use super::{despawn_screen, GameState, PetOwned, TEXT_COLOR};

use futures::StreamExt;
use sp_keyring::sr25519::sr25519::Pair;
use sp_keyring::AccountKeyring;
use subxt::utils::AccountId32;
use subxt::{tx::PairSigner, tx::TxStatus, OnlineClient, PolkadotConfig};
use thiserror::Error as ThisError;

// This plugin manages the menu, with 5 different screens:
// - a main menu with "New Game", "Settings", "Quit"
// - a settings menu with two submenus and a back button
// - two settings screen with a setting that can be set and a back button
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // At start, the menu is not enabled. This will be changed in `menu_setup` when
            // entering the `GameState::Menu` state.
            // Current screen in the menu is handled by an independent state from `GameState`
            .add_state::<MenuState>()
            .add_system(menu_setup.in_schedule(OnEnter(GameState::Menu)))
            // Systems to handle the main menu screen
            .add_systems((
                main_menu_setup.in_schedule(OnEnter(MenuState::Main)),
                despawn_screen::<OnMainMenuScreen>.in_schedule(OnExit(MenuState::Main)),
            ))
            // Systems to handle the new game menu screen
            .add_systems((
                new_game_setup.in_schedule(OnEnter(MenuState::NewGame)),
                despawn_screen::<OnNewGameScreen>.in_schedule(OnExit(MenuState::NewGame)),
            ))
            .add_systems((
                transaction_setup.in_schedule(OnEnter(MenuState::Transaction)),
                despawn_screen::<OnTransactionScreen>.in_schedule(OnExit(MenuState::Transaction)),
            ))
            // Common systems to all screens that handles buttons behaviour
            .add_systems((menu_action, button_system).in_set(OnUpdate(GameState::Menu)));
    }
}

// State used for the current menu screen
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    Main,
    NewGame,
    PlayMenu,
    FeedMenu,
    Settings,
    Update,
    Transaction,
    #[default]
    Disabled,
}

// Tag component used to tag entities added on the main menu screen
#[derive(Component)]
struct OnMainMenuScreen;

// Tag component used to tag entities added on the new game screen
#[derive(Component)]
struct OnNewGameScreen;

// Tag component used to tag entities added on the settings menu screen
#[derive(Component)]
struct OnSettingsMenuScreen;

// Tag component used to tag entities added on the update screen
#[derive(Component)]
struct OnUpdateScreen;

// Tag component used to tag entities added on the transaction screen
#[derive(Component)]
struct OnTransactionScreen;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

// Tag component used to mark which setting is currently selected
#[derive(Component)]
struct OnPlayerIdInputText;

#[derive(Component)]
struct OnPetNameInputText;

#[derive(Component)]
struct OnPetIdInputText;

#[derive(Component)]
struct OnPetSpeciesInputText;

#[derive(Component)]
struct SelectedOption;

// All actions that can be triggered from a button click
#[derive(Component)]
enum MenuButtonAction {
    NewGame,      //Create a new Game
    ContinueGame, //Continue the Game
    //Settings,//Game settings
    MintPet(String, String),
    //Update, //Pet state update
    Transaction, //Buy or Sell pet
    BackToMainMenu,
    Quit,
}

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

// This system updates the settings when a new value for a setting is selected, and marks
// the button as the one currently selected

fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

fn main_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    // Common style for all buttons on the screen
    let button_style = Style {
        size: Size::new(Val::Px(250.0), Val::Px(65.0)),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_icon_style = Style {
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
            OnMainMenuScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::DARK_GREEN.into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Display the game name
                    parent.spawn(
                        TextBundle::from_section(
                            "Welcome to Window Pet!",
                            TextStyle {
                                font: font.clone(),
                                font_size: 30.0,
                                color: TEXT_COLOR,
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        }),
                    );

                    // Display buttons for each action available from the main menu:
                    // - new game
                    // - settings
                    // - quit
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            MenuButtonAction::NewGame,
                        ))
                        .with_children(|parent| {
                            let icon = asset_server.load("textures/Game Icons/right.png");
                            parent.spawn(ImageBundle {
                                style: button_icon_style.clone(),
                                image: UiImage::new(icon),
                                ..default()
                            });
                            parent.spawn(TextBundle::from_section(
                                "New Game",
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
                            MenuButtonAction::ContinueGame,
                        ))
                        .with_children(|parent| {
                            let icon = asset_server.load("textures/Game Icons/right.png");
                            parent.spawn(ImageBundle {
                                style: button_icon_style.clone(),
                                image: UiImage::new(icon),
                                ..default()
                            });
                            parent.spawn(TextBundle::from_section(
                                "Continue",
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
                            //MenuButtonAction::Settings,
                        ))
                        .with_children(|parent| {
                            let icon = asset_server.load("textures/Game Icons/wrench.png");
                            parent.spawn(ImageBundle {
                                style: button_icon_style.clone(),
                                image: UiImage::new(icon),
                                ..default()
                            });
                            parent.spawn(TextBundle::from_section(
                                "Settings",
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
                            //MenuButtonAction::Update,
                        ))
                        .with_children(|parent| {
                            let icon = asset_server.load("textures/Game Icons/wrench.png");
                            parent.spawn(ImageBundle {
                                style: button_icon_style.clone(),
                                image: UiImage::new(icon),
                                ..default()
                            });
                            parent.spawn(TextBundle::from_section(
                                "Update",
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style,
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            MenuButtonAction::Transaction,
                        ))
                        .with_children(|parent| {
                            let icon = asset_server.load("textures/Game Icons/wrench.png");
                            parent.spawn(ImageBundle {
                                style: button_icon_style,
                                image: UiImage::new(icon),
                                ..default()
                            });
                            parent
                                .spawn(TextBundle::from_section("Transcation", button_text_style));
                        });
                });
        });
}
//New game menu setup, enter a webpage to mint a pet if the user don't have one.
fn new_game_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_style = Style {
        size: Size::new(Val::Px(150.0), Val::Px(50.0)),
        //margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let text_style = TextStyle {
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        font_size: 24.0,
        color: TEXT_COLOR,
    };

    let node_style = Style {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        align_content: AlignContent::Center,
        gap: Size {
            width: Val::Px(15.0),
            height: Val::Px(15.0),
        },
        ..default()
    };

    let text_node_bundle_style = Style {
        size: Size {
            width: Val::Px(200.),
            height: Val::Px(50.),
        },
        align_items: AlignItems::Center,
        align_content: AlignContent::Center,
        ..default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    gap: Size {
                        width: Val::Px(15.0),
                        height: Val::Px(15.0),
                    },
                    ..default()
                },
                background_color: Color::DARK_GREEN.into(),
                ..default()
            },
            OnNewGameScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: node_style.clone(),
                    background_color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section("Player Id   ".to_string(), text_style.clone()),
                        ..default()
                    });

                    parent
                        .spawn(
                            NodeBundle {
                                style: text_node_bundle_style.clone(),
                                background_color: Color::DARK_GRAY.into(),

                                ..default()
                            },
                            //OnIdInputText,
                        )
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle {
                                    text: Text::from_section(
                                        "Alice".to_string(),
                                        text_style.clone(),
                                    ),
                                    ..default()
                                },
                                OnPlayerIdInputText,
                            ));
                        });
                });

            parent
                .spawn(NodeBundle {
                    style: node_style.clone(),
                    background_color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section("Pet Species ".to_string(), text_style.clone()),
                        ..default()
                    });

                    parent
                        .spawn(NodeBundle {
                            style: text_node_bundle_style.clone(),
                            background_color: Color::DARK_GRAY.into(),

                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle {
                                    text: Text::from_section(
                                        "Turtle".to_string(),
                                        text_style.clone(),
                                    ),
                                    ..default()
                                },
                                OnPetSpeciesInputText,
                            ));
                        });
                });
            parent
                .spawn(NodeBundle {
                    style: node_style.clone(),
                    background_color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section("Pet Id      ".to_string(), text_style.clone()),
                        ..default()
                    });

                    parent
                        .spawn(NodeBundle {
                            style: text_node_bundle_style.clone(),
                            background_color: Color::DARK_GRAY.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle {
                                    text: Text::from_section("1".to_string(), text_style.clone()),
                                    ..default()
                                },
                                OnPetIdInputText,
                            ));
                        });
                });

            parent
                .spawn(NodeBundle {
                    style: node_style.clone(),
                    background_color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section("Pet Name    ".to_string(), text_style.clone()),
                        ..default()
                    });

                    parent
                        .spawn(NodeBundle {
                            style: text_node_bundle_style.clone(),
                            background_color: Color::DARK_GRAY.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle {
                                    text: Text::from_section(
                                        "Annatle".to_string(),
                                        text_style.clone(),
                                    ),
                                    ..default()
                                },
                                OnPetNameInputText,
                            ));
                        });
                });

            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        background_color: Color::DARK_GRAY.into(),
                        ..default()
                    },
                    MenuButtonAction::MintPet("test-name".to_string(), "test-species".to_string()),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section("Mint Pet".to_string(), text_style.clone()),
                        ..default()
                    });
                });
            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        background_color: Color::DARK_GRAY.into(),
                        ..default()
                    },
                    MenuButtonAction::BackToMainMenu,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section("Back to Main".to_string(), text_style.clone()),
                        ..default()
                    });
                });
        });
}

fn transaction_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_style = Style {
        size: Size::new(Val::Px(150.0), Val::Px(50.0)),
        //margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let text_style = TextStyle {
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        font_size: 24.0,
        color: TEXT_COLOR,
    };

    let node_style = Style {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        align_content: AlignContent::Center,
        gap: Size {
            width: Val::Px(15.0),
            height: Val::Px(15.0),
        },
        ..default()
    };

    let text_node_bundle_style = Style {
        size: Size {
            width: Val::Px(200.),
            height: Val::Px(50.),
        },
        align_items: AlignItems::Center,
        align_content: AlignContent::Center,
        ..default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    gap: Size {
                        width: Val::Px(15.0),
                        height: Val::Px(15.0),
                    },
                    ..default()
                },
                background_color: Color::DARK_GREEN.into(),
                ..default()
            },
            OnTransactionScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: node_style.clone(),
                    background_color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section("Sender Id   ".to_string(), text_style.clone()),
                        ..default()
                    });

                    parent
                        .spawn(
                            NodeBundle {
                                style: text_node_bundle_style.clone(),
                                background_color: Color::DARK_GRAY.into(),

                                ..default()
                            },
                            //OnIdInputText,
                        )
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle {
                                    text: Text::from_section(
                                        "Alice".to_string(),
                                        text_style.clone(),
                                    ),
                                    ..default()
                                },
                                OnPlayerIdInputText,
                            ));
                        });
                });

            parent
                .spawn(NodeBundle {
                    style: node_style.clone(),
                    background_color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section("Receiver Id ".to_string(), text_style.clone()),
                        ..default()
                    });

                    parent
                        .spawn(NodeBundle {
                            style: text_node_bundle_style.clone(),
                            background_color: Color::DARK_GRAY.into(),

                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle {
                                    text: Text::from_section("Bob".to_string(), text_style.clone()),
                                    ..default()
                                },
                                OnPetSpeciesInputText,
                            ));
                        });
                });
            parent
                .spawn(NodeBundle {
                    style: node_style.clone(),
                    background_color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section("Pet Id      ".to_string(), text_style.clone()),
                        ..default()
                    });

                    parent
                        .spawn(NodeBundle {
                            style: text_node_bundle_style.clone(),
                            background_color: Color::DARK_GRAY.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle {
                                    text: Text::from_section("1".to_string(), text_style.clone()),
                                    ..default()
                                },
                                OnPetIdInputText,
                            ));
                        });
                });

            parent
                .spawn(ButtonBundle {
                    style: button_style.clone(),
                    background_color: Color::DARK_GRAY.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section("Submit".to_string(), text_style.clone()),
                        ..default()
                    });
                });
            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        background_color: Color::DARK_GRAY.into(),
                        ..default()
                    },
                    MenuButtonAction::BackToMainMenu,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section("Back to Main".to_string(), text_style.clone()),
                        ..default()
                    });
                });
        });
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut pet_owned: ResMut<NextState<PetOwned>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Clicked {
            match menu_button_action {
                MenuButtonAction::Quit => app_exit_events.send(AppExit),
                //Enter new game menu
                MenuButtonAction::NewGame => menu_state.set(MenuState::NewGame),

                MenuButtonAction::ContinueGame => {

                    game_state.set(GameState::Game);
                    menu_state.set(MenuState::Disabled);
                }

                //MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                //MenuButtonAction::Update => menu_state.set(MenuState::Update),
                MenuButtonAction::Transaction => menu_state.set(MenuState::Transaction),
                //Return to Main menu
                MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                //Submit mint_pet information
                MenuButtonAction::MintPet(name, species) => {
                    println!("mint pet, {}, {}", name, species);
                    let result = tokio::runtime::Runtime::new().unwrap().block_on(mint(1, PetSpecies::Rabbit, name.clone()));
                    match result {
                        Ok(_) => {
                            println!("minted pet");
                            pet_owned.set(PetOwned::Enable);
                            game_state.set(GameState::Game);
                            menu_state.set(MenuState::Disabled);
                            
                        },
                        Err(e) => {
                            println!("error minting pet: {:?}", e);
                            menu_state.set(MenuState::Main)
                        },
                    }
                },

                _ => menu_state.set(MenuState::Main),
            }
        }
    }
}

fn listen_received_character_events_player_id_input(
    mut events: EventReader<ReceivedCharacter>,
    kbd: Res<Input<KeyCode>>,
    mut edit_text: Query<&mut Text, With<OnPlayerIdInputText>>,
) {
    for event in events.iter() {
        if kbd.just_pressed(KeyCode::Return) {
            let userid = &edit_text.single_mut().sections[0].value;
            println!("{userid:?}");
        } else if kbd.just_pressed(KeyCode::Back) {
            edit_text.single_mut().sections[0].value.pop();
        } else {
            edit_text.single_mut().sections[0].value.push(event.char);
        }
    }
}

#[subxt::subxt(runtime_metadata_path = "./metadata.scale")]
//#[subxt::subxt(runtime_metadata_path = "/mnt/hddisk1/github/SuperPetGame-RST/metadata.scale")]
pub mod polkadot {}
type PetId = u32;
type PetSpecies = polkadot::runtime_types::pallet_pet::pallet::Species;
type PetInfo = polkadot::runtime_types::pallet_pet::pallet::PetInfo;
//type Error = polkadot::runtime_types::pallet_pet::pallet::Error;
type PetName = polkadot::runtime_types::bounded_collections::bounded_vec::BoundedVec<u8>;

#[derive(Debug)]
pub struct PetError;


async fn mint(
    petid: PetId,
    species: PetSpecies,
    name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("start to mint!");

    let api = OnlineClient::<PolkadotConfig>::new().await?;

    //Some pet information, include petname, species, petid
    let petid: PetId = 1;
    let species = polkadot::runtime_types::pallet_pet::pallet::Species::Turtle;
    let petname = polkadot::runtime_types::bounded_collections::bounded_vec::BoundedVec(name.into_bytes());

    //Mint a pet for account Alice.
    let from = PairSigner::new(AccountKeyring::Alice.pair());

    // Build a pet mint extrinsic.
    let balance_transfer_tx = polkadot::tx().pet_module().mint(petname, species, petid);
    // Submit the balance transfer extrinsic from Alice, and wait for it to be successful
    // and in a finalized block. We get back the extrinsic events if all is well.

    let mut mint_pet = api
        .tx()
        .sign_and_submit_then_watch_default(&balance_transfer_tx, &from)
        //.await?
        //.wait_for_finalized_success()
        .await?;

    while let Some(status) = mint_pet.next().await {
        match status? {
            // It's finalized in a block!
            TxStatus::Finalized(in_block) => {
                println!(
                    "Transaction is finalized in block ",
                    //in_block.extrinsic_hash(),
                    //in_block.block_hash()
                );

                // grab the events and fail if no ExtrinsicSuccess event seen:
                let events = in_block.fetch_events().await?;

                //println!("Event:{events:?}");
                // We can look for events (this uses the static interface; we can also iterate
                //over them and dynamically decode them):
                let transfer_event =
                    events.find_first::<polkadot::pet_module::events::PetMinted>()?;

                if let Some(_event) = transfer_event {
                    println!("Yeah! You have your own pet!");
                } else {
                    println!("Error::AlreadyHavePet");
                }
            }
            TxStatus::Ready => {}
            TxStatus::InBlock(_) => {}
            // Just log any other status we encounter:
            other => {
                println!("Status: {other:?}");
            }
        }
    }

    Ok(())
}
