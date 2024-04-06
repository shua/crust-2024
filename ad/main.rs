use bevy::{
    prelude::*, 
    audio::Volume,
    render::camera::ScalingMode,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

#[derive(Component)]
struct DebugText;

const WINDOW_WIDTH: f32 = 800.;
const WINDOW_HEIGHT: f32 = 600.;
// surely this should be wide enough
const LETTERBOX_WIDTH: f32 = 2000.;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Baby".into(),
                    resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                    ..default()
                }),
            ..default()
            },
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (sprite_animation, sound_player, volume, draw_debug))
        .run();
}

// ------------------------------- Sprite Animation -------------------------------
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

fn sprite_animation(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices, 
        &mut SpriteAnimationType, 
        &mut SpriteAnimationTimer, 
        &mut TextureAtlas)>,
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

// ------------------------------- Sound -------------------------------
#[derive(Component)]
enum SoundVolume {
    Background,
    CarIdle(f32),
}

#[derive(Component)]
struct SoundPlayTimer(Timer);

fn volume(query: Query<(&AudioSink, &SoundVolume)>, time: Res<Time>) {
    for (sink, sound) in &query {
        match sound {
            SoundVolume::Background => sink.set_volume((time.elapsed_seconds() / KEYFRAME_BG_MUSIC_VOL_MAX).min(1.0)),
            SoundVolume::CarIdle(base_volume) => sink.set_volume(
                base_volume * inv_lerp(
                    KEYFRAME_CAR_SND_IDLE_START, 
                    KEYFRAME_CAR_SND_IDLE_VOL_MAX, 
                    time.elapsed_seconds()
                )
            ),
        }
    }
}

fn sound_player(mut query: Query<(&AudioSink, &mut SoundPlayTimer)>, time: Res<Time>) {
    for (sink, mut sound_play_timer) in &mut query {
        sound_play_timer.0.tick(time.delta());
        if sound_play_timer.0.just_finished() {
            sink.play();
        }
    }
}

// ------------------------------- Intro Cutscene -------------------------------
// background soundscape starts playing but fades in quickly
const KEYFRAME_BG_MUSIC_VOL_MAX: f32 = 3.0;

// car moves into frame
const KEYFRAME_CAR_MOVE_START: f32 = 5.0;
const KEYFRAME_CAR_SND_IDLE_START: f32 = 5.0;

// car stops
const KEYFRAME_CAR_MOVE_STOP: f32 = 10.0;
const KEYFRAME_CAR_SND_IDLE_VOL_MAX: f32 = 8.0;

// brake squeak
const KEYFRAME_CAR_SND_BRAKE: f32 = 9.75;

// window roll 
const KEYFRAME_CAR_SND_WINDOW: f32 = 11.0;

// baby thrown
const KEYFRAME_BABY_THROWN: f32 = 15.0;

// thump
// car turns around
// car burnout
// car sound fades

fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {

    commands.spawn(Camera2dBundle {
       projection: OrthographicProjection {
        // When creating our own OrthographicProjection we need to set the far and near
        // values ourselves. 
        // See: https://bevy-cheatbook.github.io/2d/camera.html#caveat-nearfar-values
        far: 1000.,
        near: -1000.,
        scaling_mode: ScalingMode::FixedVertical(600.),
        ..default()
       }, 
        ..default()});

    commands.spawn((
        DebugText,
        Text2dBundle {
            text: Text::from_section("hello, baby!", TextStyle::default()),
            text_anchor: bevy::sprite::Anchor::TopLeft,
            transform: Transform {
                translation: Vec3::new(-380., 280., 10.),
                scale: Vec3::ONE,
                ..default()
            },
            ..default()
        },
    ));

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

    // Verical Letterboxes
    let letterbox_h_offset = (WINDOW_WIDTH + LETTERBOX_WIDTH) / 2.;
    commands.spawn(MaterialMesh2dBundle{
        mesh: Mesh2dHandle(meshes.add(Rectangle::new(LETTERBOX_WIDTH, WINDOW_WIDTH))),
        material: materials.add(Color::BLACK),
        transform: Transform::from_xyz(letterbox_h_offset, 0., 100.),
        ..default()
    });

    commands.spawn(MaterialMesh2dBundle{
        mesh: Mesh2dHandle(meshes.add(Rectangle::new(LETTERBOX_WIDTH, WINDOW_WIDTH))),
        material: materials.add(Color::BLACK),
        transform: Transform::from_xyz(-letterbox_h_offset, 0., 100.),
        ..default()
    });

    spawn_car(&mut commands, &asset_server, &mut texture_atlas_layouts, &mut animations);
    spawn_baby(&mut commands, &asset_server, &mut texture_atlas_layouts);

    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/city-background.wav"),
            settings: PlaybackSettings {
                paused: false,
                volume: Volume::ZERO,
                ..default()
            }
        },
        SoundVolume::Background,
    ));

    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/car-idle.wav"),
            settings: PlaybackSettings {
                paused: true,
                mode: bevy::audio::PlaybackMode::Loop,
                ..default()
            }
        },
        SoundVolume::CarIdle(0.2), // control base volume
        SoundPlayTimer(Timer::from_seconds(KEYFRAME_CAR_SND_IDLE_START, TimerMode::Once)),
    ));

    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/car-brake-squeak.wav"),
            settings: PlaybackSettings {
                paused: true,
                mode: bevy::audio::PlaybackMode::Once,
                ..default()
            }
        },
        SoundPlayTimer(Timer::from_seconds(KEYFRAME_CAR_SND_BRAKE, TimerMode::Once)),
    ));

    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/car-window-open.wav"),
            settings: PlaybackSettings {
                paused: true,
                mode: bevy::audio::PlaybackMode::Once,
                ..default()
            }
        },
        SoundPlayTimer(Timer::from_seconds(KEYFRAME_CAR_SND_WINDOW, TimerMode::Once)),
    ));

    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/baby-throw-woosh.wav"),
            settings: PlaybackSettings {
                paused: true,
                mode: bevy::audio::PlaybackMode::Once,
                ..default()
            }
        },
        SoundPlayTimer(Timer::from_seconds(KEYFRAME_BABY_THROWN, TimerMode::Once)),
    ));
}

fn spawn_car(
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    animations: &mut ResMut<Assets<AnimationClip>>,
) {
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
            keyframe_timestamps: vec![
                KEYFRAME_CAR_MOVE_START, 
                KEYFRAME_CAR_MOVE_STOP],
            keyframes: Keyframes::Translation(vec![
                Vec3::new(700., -50., 1.),
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
            transform: Transform::from_xyz(700., -50., 1.)
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
}

fn spawn_baby(    
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
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

fn draw_debug(mut text: Query<&mut Text, With<DebugText>>, time: Res<Time>) {
    for mut t in &mut text {
        *t = Text::from_section(format!("time: {:.3}", time.elapsed_seconds()), TextStyle::default());
    }
}

// Clamped inverse LERP 
// https://www.gamedev.net/articles/programming/general-and-gameplay-programming/inverse-lerp-a-super-useful-yet-often-overlooked-function-r5230/
fn inv_lerp(a: f32, b: f32, x: f32) -> f32 {
    if x < a {
        0.
    } else if x > b {
        1.
    } else {
        (x - a) / (b - a)
    }
}

