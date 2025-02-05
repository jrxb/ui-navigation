use bevy::prelude::*;

use bevy_ui_navigation::{
    systems::{default_gamepad_input, default_keyboard_input, InputMapping},
    Focusable, Focused, NavEvent, NavMenu, NavigationPlugin,
};

/// Shows how navigation is supported even between siblings separated by a
/// hierahierarchical level of nodes, shows how to "wall of" a part of the UI
/// (so that it requires different interactions to reach)
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(NavigationPlugin)
        .init_resource::<ButtonMaterials>()
        .init_resource::<InputMapping>()
        .add_startup_system(setup)
        .add_system(button_system)
        .add_system(print_nav_events)
        .add_system(default_keyboard_input)
        .add_system(default_gamepad_input)
        .run();
}

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    focused: Handle<ColorMaterial>,
    pink: Handle<ColorMaterial>,
    backgrounds: [Handle<ColorMaterial>; 3],
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            focused: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
            pink: materials.add(Color::rgba(1.00, 0.35, 1.0, 0.5).into()),
            backgrounds: [
                materials.add(Color::rgba(1.0, 0.35, 0.35, 0.5).into()),
                materials.add(Color::rgba(0.35, 1.0, 0.35, 0.5).into()),
                materials.add(Color::rgba(0.35, 0.35, 1.0, 0.5).into()),
            ],
        }
    }
}

fn print_nav_events(mut events: EventReader<NavEvent>) {
    for event in events.iter() {
        println!("{:?}", event);
    }
}

fn button_system(
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<(Option<&Focused>, &mut Handle<ColorMaterial>), With<Button>>,
) {
    for (interaction, mut material) in interaction_query.iter_mut() {
        match interaction {
            Some(_) => {
                *material = button_materials.focused.clone();
            }
            None => {
                *material = button_materials.normal.clone();
            }
        }
    }
}

fn setup(mut commands: Commands, button_materials: Res<ButtonMaterials>) {
    let size = |width, height| Size::new(Val::Percent(width), Val::Percent(height));
    let flex_wrap = FlexWrap::Wrap;
    let style = Style {
        size: size(100.0, 100.0),
        flex_wrap,
        ..Style::default()
    };
    let bundle = NodeBundle {
        style,
        ..Default::default()
    };
    let size = size(45.0, 45.0);
    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(bundle)
        // The `Focusable`s buttons are not direct siblings, we can navigate through
        // them beyond direct hierarchical relationships.
        //
        // To prevent this, we can add a `NavMenu` as a sort of boundary
        // between different sets of `Focusable`s. This requires having an
        // englobing `NavMenu` that contains all other `NavMenu`s or
        // `Focusable`s
        //
        // YOU MUSTE ADD A NavMenu enclosing ALL Focusable and ALL NavMenu (but
        // themselves) Subtile broken behavior will ensure otherwise
        .insert(NavMenu::root())
        .with_children(|commands| {
            for i in 0..3 {
                let style = Style {
                    size,
                    ..Style::default()
                };
                let bundle = NodeBundle {
                    style,
                    material: button_materials.backgrounds[i].clone(),
                    ..Default::default()
                };
                commands.spawn_bundle(bundle).with_children(|commands| {
                    spawn_button(commands, &button_materials);
                    spawn_button(commands, &button_materials);
                    spawn_button(commands, &button_materials);
                });
            }
            let style = Style {
                size,
                ..Style::default()
            };
            let bundle = NodeBundle {
                style,
                material: button_materials.pink.clone(),
                ..Default::default()
            };
            commands
                .spawn_bundle(bundle)
                // We don't want to be able to access the pink square, so we
                // add a `NavMenu` as boundary
                .insert(NavMenu::root())
                .with_children(|commands| {
                    spawn_button(commands, &button_materials);
                    spawn_button(commands, &button_materials);
                    spawn_button(commands, &button_materials);
                    spawn_button(commands, &button_materials);
                });
        });
}
fn spawn_button(commands: &mut ChildBuilder, button_materials: &ButtonMaterials) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(120.0), Val::Px(60.0)),
                margin: Rect::all(Val::Percent(4.0)),
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        // The `Focusable`s are not direct siblings, we can navigate through
        // them beyond direct hierarchical relationships.
        .insert(Focusable::default());
}
