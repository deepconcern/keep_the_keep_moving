use bevy::prelude::*;

use crate::app_state::AppState;
use crate::colors::DARK_GRAY;

pub struct MenuPlugin;

#[derive(Component)]
struct Menu;

fn animate_buttons(
    mut button_query: Query<
        (
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
            &Interaction,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut TextColor>,
) {
    for (mut background_color, mut border_color, children, interaction) in button_query.iter_mut() {
        let Ok(mut text_color) = text_query.get_mut(children[0]) else {
            continue;
        };

        match *interaction {
            Interaction::Hovered => {
                background_color.0 = Color::BLACK;
                border_color.0 = DARK_GRAY;
                text_color.0 = DARK_GRAY;
            }
            Interaction::None => {
                background_color.0 = Color::BLACK;
                border_color.0 = Color::WHITE;
                text_color.0 = Color::WHITE;
            }
            Interaction::Pressed => {
                background_color.0 = Color::WHITE;
                border_color.0 = Color::WHITE;
                text_color.0 = Color::BLACK;
            }
        };
    }
}

fn start_game_button(
    button_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in button_query.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::Game);
        }
    }
}

fn destroy_menu(mut commands: Commands, query: Query<Entity, With<Menu>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_menu(asset_server: Res<AssetServer>, mut commands: Commands) {
    let font_handle = asset_server.load("fonts/PressStart2P-Regular.ttf");

    commands
        .spawn((
            BackgroundColor(Color::BLACK),
            Menu,
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                height: Val::Vh(100.0),
                justify_content: JustifyContent::Center,
                width: Val::Vw(100.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    align_items: AlignItems::Center,
                    display: Display::Flex,
                    flex_grow: 1.0,
                    justify_content: JustifyContent::Center,
                    width: Val::Percent(100.0),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Keep the Keep Moving!"),
                        TextColor(Color::WHITE),
                        TextFont {
                            font: font_handle.clone(),
                            ..default()
                        },
                    ));
                });
            parent
                .spawn(Node {
                    align_items: AlignItems::Center,
                    display: Display::Flex,
                    flex_grow: 1.0,
                    justify_content: JustifyContent::Center,
                    width: Val::Percent(100.0),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            BackgroundColor(Color::WHITE),
                            Button,
                            Node {
                                border: UiRect::all(Val::Px(5.0)),
                                padding: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            BorderColor(Color::WHITE),
                        ))
                        .with_child((
                            Text::new("Start Game"),
                            TextColor(Color::WHITE),
                            TextFont {
                                font: font_handle.clone(),
                                ..default()
                            },
                        ));
                });
        });
}

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), setup_menu);
        app.add_systems(OnExit(AppState::Menu), destroy_menu);
        app.add_systems(
            Update,
            (animate_buttons, start_game_button).run_if(in_state(AppState::Menu)),
        );
    }
}
