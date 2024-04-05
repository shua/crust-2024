//! Renders a 2D scene containing a single, moving sprite.

use bevy::{
    prelude::*, 
    render::camera::ScalingMode,
};

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
        .add_systems(Update, sprite_animation)
        .run();
}

// Indices representing a sprite sheet
#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component)]
enum SpriteAnimationType {
    // play to end, repeat
    // 123123123
    Linear,   

    // play to end, go backwards, repeat
    // 123212321
    PingPong(PingPongState), 
}

impl SpriteAnimationType {
    fn new_ping_pong() -> Self {
        Self::PingPong(PingPongState::default())
    }
}

enum PingPongState {
    Forward,
    Backward,
}

impl PingPongState {
    fn default() -> Self {
        PingPongState::Forward
    }
}

#[derive(Component, Deref, DerefMut)]
struct SpriteAnimationTimer(Timer);

fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut animations: ResMut<Assets<AnimationClip>>,
) {

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

    // Scene
    commands.spawn(SpriteBundle {
        texture: asset_server.load("scenes/intro/bg.png"),
        transform: Transform::from_xyz(0., 0., 0.,),
        ..default()
    });
    commands.spawn(SpriteBundle {
        texture: asset_server.load("scenes/intro/pile_1.png"),
        transform: Transform::from_xyz(0., 0., 10.,),
        ..default()
    });
    commands.spawn(SpriteBundle {
        texture: asset_server.load("scenes/intro/pile_2.png"),
        transform: Transform::from_xyz(0., 0., 10.,),
        ..default()
    });

    // Car
    let layout = TextureAtlasLayout::from_grid(Vec2::new(170.,100.), 3, 4, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let sprite_animation_indices = AnimationIndices{ first: 1, last: 6 };

    let car_name = Name::new("car");
    let mut car_animation = AnimationClip::default();
    car_animation.add_curve_to_path(
        EntityPath {
            parts: vec![car_name.clone()],
        },
        VariableCurve {
            keyframe_timestamps: vec![0.0, 6.0],
            keyframes: Keyframes::Translation(vec![
                Vec3::new(500., -70., 1.),
                Vec3::new(-50., -150., 1.),
            ]),
            interpolation: Interpolation::Linear,
        },
    );
    let mut player = AnimationPlayer::default();
    player.play(animations.add(car_animation));
    
    commands.spawn((
        car_name,
        SpriteBundle {
            texture: asset_server.load("car-sheet.png"),
            transform: Transform::from_xyz(-200., -150., 1.)
                .with_scale(Vec3::ONE * 1.5),
            sprite: Sprite {
                flip_x: true,
                ..default()
            },
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas_layout,
            index: sprite_animation_indices.first,
        },
        sprite_animation_indices,
        SpriteAnimationType::Linear,
        player,
        SpriteAnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));

    spawn_baby(commands, asset_server, texture_atlas_layouts);

}

fn spawn_baby(    
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = TextureAtlasLayout::from_grid(Vec2::new(251.,377.), 3, 2, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let sprite_animation_indices = AnimationIndices{ first: 0, last: 4 };

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("baby-idle-sheet.png"),
            transform: Transform::from_xyz(-0., -200., 2.)
                .with_scale(Vec3::ONE * 0.5),
            sprite: Sprite {
                flip_x: false,
                ..default()
            },
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas_layout,
            index: sprite_animation_indices.first,
        },
        sprite_animation_indices,
        SpriteAnimationType::new_ping_pong(),
        SpriteAnimationTimer(Timer::from_seconds(0.11, TimerMode::Repeating)),
    ));
}

fn sprite_animation(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut SpriteAnimationType, &mut SpriteAnimationTimer, &mut TextureAtlas)>,
) {
    for (indices, mut anim_type, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            match *anim_type {
                SpriteAnimationType::Linear => {
                    atlas.index = if atlas.index == indices.last {
                        indices.first
                    } else {
                        atlas.index + 1
                    }
                },
                SpriteAnimationType::PingPong(ref mut ppstate) => {
                    match ppstate {
                        PingPongState::Forward => {
                            atlas.index = if atlas.index == indices.last {
                                *ppstate = PingPongState::Backward;
                                atlas.index - 1
                            } else {
                                atlas.index + 1
                            }
                        },
                        PingPongState::Backward => {
                            atlas.index = if atlas.index == indices.first {
                                *ppstate = PingPongState::Forward;
                                atlas.index + 1
                            } else {
                                atlas.index - 1
                            }
                        },
                    }
                },
            }
        }
    }
}