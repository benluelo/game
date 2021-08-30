#[allow(clippy::needless_pass_by_value)]
pub mod constants;
mod dungeon;
pub mod key_press_handling;
pub mod player;
pub mod utils;

use ::dungeon::DungeonTile;
use bevy::{ecs::schedule::ReportExecutionOrderAmbiguities, prelude::*};
use std::{num::NonZeroU16, ops::Index};

use crate::{dungeon::DungeonPlugin, key_press_handling::KeyPressHandlingPlugin};

pub struct CurrentFloor(pub NonZeroU16);

pub struct ExitFloor;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "game".to_string(),
            width: 500.0,
            height: 500.0,
            ..Default::default()
        })
        .add_startup_system(setup.system().label("setup"))
        .add_plugins(DefaultPlugins)
        .add_plugin(KeyPressHandlingPlugin)
        .add_plugin(DungeonPlugin)
        .insert_resource(ReportExecutionOrderAmbiguities)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    asset_server.watch_for_changes().unwrap();

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    /* ui testing */
    {
        commands
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                material: materials.add(Color::NONE.into()),
                ..Default::default()
            })
            .with_children(|parent| {
                // left vertical fill (border)
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Px(200.0), Val::Percent(100.0)),
                            border: Rect::all(Val::Px(2.0)),
                            ..Default::default()
                        },
                        material: materials.add(Color::rgb(0.65, 0.65, 0.65).into()),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        // left vertical fill (content)
                        parent
                            .spawn_bundle(NodeBundle {
                                style: Style {
                                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                    align_items: AlignItems::FlexEnd,
                                    ..Default::default()
                                },
                                material: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                // text
                                parent.spawn_bundle(TextBundle {
                                    style: Style {
                                        margin: Rect::all(Val::Px(5.0)),
                                        ..Default::default()
                                    },
                                    text: Text::with_section(
                                        "Text Example",
                                        TextStyle {
                                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                            font_size: 30.0,
                                            color: Color::WHITE,
                                        },
                                        Default::default(),
                                    ),
                                    ..Default::default()
                                });
                            });
                    });
                // right vertical fill
                parent.spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.0), Val::Percent(100.0)),
                        ..Default::default()
                    },
                    material: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
                    ..Default::default()
                });
                // absolute positioning
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Px(200.0), Val::Px(200.0)),
                            position_type: PositionType::Absolute,
                            position: Rect {
                                left: Val::Px(210.0),
                                bottom: Val::Px(10.0),
                                ..Default::default()
                            },
                            border: Rect::all(Val::Px(20.0)),
                            ..Default::default()
                        },
                        material: materials.add(Color::rgb(0.4, 0.4, 1.0).into()),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                ..Default::default()
                            },
                            material: materials.add(Color::rgb(0.8, 0.8, 1.0).into()),
                            ..Default::default()
                        });
                    });
                // render order test: reddest in the back, whitest in the front (flex center)
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            position_type: PositionType::Absolute,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        material: materials.add(Color::NONE.into()),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(NodeBundle {
                                style: Style {
                                    size: Size::new(Val::Px(100.0), Val::Px(100.0)),
                                    ..Default::default()
                                },
                                material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                parent.spawn_bundle(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Px(100.0), Val::Px(100.0)),
                                        position_type: PositionType::Absolute,
                                        position: Rect {
                                            left: Val::Px(20.0),
                                            bottom: Val::Px(20.0),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    material: materials.add(Color::rgb(1.0, 0.3, 0.3).into()),
                                    ..Default::default()
                                });
                                parent.spawn_bundle(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Px(100.0), Val::Px(100.0)),
                                        position_type: PositionType::Absolute,
                                        position: Rect {
                                            left: Val::Px(40.0),
                                            bottom: Val::Px(40.0),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    material: materials.add(Color::rgb(1.0, 0.5, 0.5).into()),
                                    ..Default::default()
                                });
                                parent.spawn_bundle(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Px(100.0), Val::Px(100.0)),
                                        position_type: PositionType::Absolute,
                                        position: Rect {
                                            left: Val::Px(60.0),
                                            bottom: Val::Px(60.0),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    material: materials.add(Color::rgb(1.0, 0.7, 0.7).into()),
                                    ..Default::default()
                                });
                                // alpha test
                                parent.spawn_bundle(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Px(100.0), Val::Px(100.0)),
                                        position_type: PositionType::Absolute,
                                        position: Rect {
                                            left: Val::Px(80.0),
                                            bottom: Val::Px(80.0),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    material: materials.add(Color::rgba(1.0, 0.9, 0.9, 0.4).into()),
                                    ..Default::default()
                                });
                            });
                    });
                // bevy logo (flex center)
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            position_type: PositionType::Absolute,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::FlexEnd,
                            ..Default::default()
                        },
                        material: materials.add(Color::NONE.into()),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        // bevy logo (image)
                        parent.spawn_bundle(ImageBundle {
                            style: Style {
                                size: Size::new(Val::Px(500.0), Val::Auto),
                                ..Default::default()
                            },
                            material: materials
                                .add(asset_server.load("branding/bevy_logo_dark_big.png").into()),
                            ..Default::default()
                        });
                    });
            });
    }

    /* ui testing */

    commands.insert_resource(Materials {
        empty_material: materials.add(
            // Color::WHITE.into()
            asset_server.load("empty.png").into(),
        ),
        wall_material: materials.add(Color::BLACK.into()),
        secret_door_material: materials.add(Color::RED.into()),
        secret_passage_material: materials.add(Color::LIME_GREEN.into()),
        treasure_chest_material: materials.add(Color::BLUE.into()),
        entrance_material: materials.add(Color::PINK.into()),
        exit_material: materials.add(Color::PURPLE.into()),
        player_material: materials.add(asset_server.load("arrow.png").into()),
    });
}

pub struct Tile {
    #[allow(dead_code)]
    tile_type: DungeonTile,
}

pub struct Materials {
    empty_material: Handle<ColorMaterial>,
    wall_material: Handle<ColorMaterial>,
    secret_door_material: Handle<ColorMaterial>,
    secret_passage_material: Handle<ColorMaterial>,
    treasure_chest_material: Handle<ColorMaterial>,
    entrance_material: Handle<ColorMaterial>,
    exit_material: Handle<ColorMaterial>,
    player_material: Handle<ColorMaterial>,
}

impl Index<DungeonTile> for Materials {
    type Output = Handle<ColorMaterial>;

    fn index(&self, index: DungeonTile) -> &Self::Output {
        match index {
            DungeonTile::Empty => &self.empty_material,
            DungeonTile::Wall => &self.wall_material,
            DungeonTile::SecretDoor { .. } => &self.secret_door_material,
            DungeonTile::SecretPassage => &self.secret_passage_material,
            DungeonTile::TreasureChest { .. } => &self.treasure_chest_material,
            DungeonTile::Entrance => &self.entrance_material,
            DungeonTile::Exit => &self.exit_material,
        }
    }
}
