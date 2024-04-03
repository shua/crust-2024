//! Renders a 2D scene containing a single, moving sprite.

use bevy::{prelude::*, render::camera::ScalingMode};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Baby".into(),
                    resolution: (800., 600.).into(),
                    ..default()
                }),
            ..default()
            },
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, sprite_movement)
        .run();
}

#[derive(Component)]
enum Direction {
    Left,
    Right,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
       projection: OrthographicProjection {
        // When creating our own OrthographicProjection we need to set the far and near
        // values ourselves. See: https://bevy-cheatbook.github.io/2d/camera.html#caveat-nearfar-values
        far: 1000.,
        near: -1000.,
        scaling_mode: ScalingMode::FixedVertical(600.),
        ..default()
       }, 
        ..default()});

    commands.spawn((SpriteBundle {
        texture: asset_server.load("scenes/intro/bg.png"),
        transform: Transform::from_xyz(0., 0., 0.,),
        ..default()
    }));
    commands.spawn((SpriteBundle {
        texture: asset_server.load("scenes/intro/pile_1.png"),
        transform: Transform::from_xyz(0., 0., 10.,),
        ..default()
    }));
    commands.spawn((SpriteBundle {
        texture: asset_server.load("scenes/intro/pile_2.png"),
        transform: Transform::from_xyz(0., 0., 10.,),
        ..default()
    }));
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("baby.png"),
            transform: Transform::from_xyz(-200., -150., 1.),
            ..default()
        },
        Direction::Right,
    ));
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>) {
    for (mut logo, mut transform) in &mut sprite_position {
        match *logo {
            Direction::Left => transform.translation.x -= 150. * time.delta_seconds(),
            Direction::Right => transform.translation.x += 150. * time.delta_seconds(),
        }

        if transform.translation.x > 300. {
            *logo = Direction::Left;
        } else if transform.translation.x < -300. {
            *logo = Direction::Right;
        }
    }
}
