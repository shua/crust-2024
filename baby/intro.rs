use bevy::{
    app::AppExit,
    audio::PlaybackMode,
    prelude::*,
    render::camera::ScalingMode,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use std::collections::HashMap as Map;

use crate::AppState;

#[derive(Component, Default)]
pub struct DebugUi {
    text: Map<&'static str, String>,
}

impl DebugUi {
    fn watch(&mut self, key: &'static str, val: impl std::fmt::Debug) {
        self.text.insert(key, format!("{:?}", val));
    }
}

pub fn draw_debug(mut dbg: Query<(&mut Text, &DebugUi)>) {
    if cfg!(debug_assertions) {
        let (mut txt, dbg) = dbg.single_mut();
        txt.sections = (dbg.text.iter())
            .map(|(k, v)| TextSection::new(format!("{k}: {v}\n"), default()))
            .collect();
    }
}

#[derive(Component)]
pub struct Subtitle;

// ------------------------------- Intro Cutscene -------------------------------
enum Q {
    // advance time
    Tick(f32),
    // set translation
    Tran(&'static str, f32, f32, f32),
    // set rotation (in radians around z-axis)
    Rot(&'static str, f32),
    // set flip x
    Flip(&'static str, bool),
    // sound paused
    Paused(&'static str, bool),
    // sound volume
    Vol(&'static str, f32),
    // despawn
    Despawn(&'static str),
    // subtitle
    Subtitle(&'static str),
}
// Camera cues
struct CQ {
    // each field follows (start, end)
    time: (f32, f32),
    scale: (f32, f32),
    tran: (Vec3, Vec3),
}
enum AR {
    Sprite(
        &'static str,
        &'static str,
        (f32, f32, usize, usize, f32, Cycle, usize, usize),
        f32,
        bool,
    ),
    Sound(&'static str, &'static str, bool),
    Overlay(&'static str, f32),
    Image(&'static str, &'static str, (f32, f32, f32), f32),
}
const ANIM_RSC: &'static [AR] = &[
    AR::Overlay("screen", 100.),
    AR::Image("bg", "scenes/intro/bg.png", (0., -35., -10.), 1.),
    AR::Image("pile1", "scenes/intro/pile_1.png", (0., -35., 10.), 1.),
    AR::Image("pile2", "scenes/intro/pile_2.png", (0., -35., 10.), 1.),
    AR::Image("pile2", "scenes/intro/pile_2.png", (0., -35., 10.), 1.),
    AR::Image("baby_thrown", "baby-thrown.png", (0., 0., -10.), 0.4),
    AR::Sprite(
        "car",
        "car-sheet.png",
        (170., 100., 3, 4, 0.11, Cycle::Loop, 1, 6),
        1.5,
        true,
    ),
    AR::Sprite(
        "baby",
        "baby-idle-sheet.png",
        (251., 377., 3, 2, 0.1, Cycle::PingPong, 0, 4),
        0.5,
        false,
    ),
    AR::Sound("city", "sounds/city-background.wav", false),
    AR::Sound("sad_song", "sounds/biedne-dziecie.wav", true),
    AR::Sound("sad_song_jazz", "sounds/biedne-dziecie-jazz.wav", true),
    AR::Sound("car_idle", "sounds/car-idle.wav", false),
    AR::Sound("car_brake", "sounds/car-brake-squeak.wav", true),
    AR::Sound("car_win_open", "sounds/car-window-open.wav", true),
    AR::Sound("car_win_close", "sounds/car-window-close.wav", true),
    AR::Sound("woosh", "sounds/woosh.wav", true),
    AR::Sound("thump", "sounds/thump.wav", true),
    AR::Sound("car_peels_out", "sounds/car-peels-out.wav", true),
];
const ANIM_CUE_JAZZ: &'static [Q] = &[
    Q::Tran("baby", 60., -200., -10.),
    Q::Vol("city", 0.),
    Q::Paused("city", false),
    Q::Paused("sad_song_jazz", true),
    Q::Paused("car_idle", true),
    Q::Paused("car_brake", true),
    Q::Paused("car_win_open", true),
    Q::Paused("car_win_close", true),
    Q::Paused("car_peels_out", true),
    Q::Paused("woosh", true),
    Q::Paused("thump", true),
    Q::Subtitle("for my son"),
    // background soundscape fades in
    Q::Tick(3.),
    Q::Vol("city", 0.8),
    // scene reveal
    Q::Tick(1.),
    Q::Despawn("screen"),
    Q::Subtitle(""),
    // car moves into frame, engine sound gets louder
    Q::Tick(2.),
    Q::Tran("car", 700., -50., 0.),
    Q::Paused("car_idle", false),
    Q::Vol("car_idle", 0.),
    Q::Tick(4.),
    Q::Vol("car_idle", 0.2),
    // brake squeak
    Q::Tick(0.5),
    Q::Paused("car_brake", false),
    // car stops
    Q::Tick(0.25),
    Q::Tran("car", -50., -150., 0.),
    // window rolls down
    Q::Tick(1.),
    Q::Paused("car_win_open", false),
    Q::Paused("sad_song_jazz", false),
    // baby thrown
    Q::Tick(3.5),
    Q::Tran("baby_thrown", -30., -100., -10.),
    Q::Rot("baby_thrown", 1.5),
    Q::Paused("woosh", false),
    // baby hits ground
    Q::Tick(1.),
    Q::Tran("baby_thrown", 30., -220., 1.),
    Q::Paused("thump", false),
    // window rolls up
    Q::Tick(1.),
    Q::Paused("car_win_close", false),
    // car turns around
    Q::Tick(4.),
    Q::Flip("car", false),
    // car burnout
    Q::Tick(1.),
    Q::Tran("car", -50., -150., 0.),
    Q::Rot("car", 0.),
    Q::Vol("car_peels_out", 0.5),
    Q::Paused("car_peels_out", false),
    Q::Vol("car_idle", 1.0),
    // car sound fades away
    Q::Tick(0.2),
    Q::Rot("car", 0.7),
    Q::Tick(1.8),
    Q::Tran("car", 700., -50., 0.),
    Q::Vol("car_idle", 0.),
    // somber music plays
    // hold camera for few seconds
    // camera slowly zooms in on baby
    // baby wriggles on ground
    Q::Tick(0.5),
    Q::Subtitle("poor lonely baby"),
    Q::Tick(6.5),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(2.0),
    Q::Subtitle("born in the summer"),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(2.3),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(3.0),
    Q::Subtitle("abandoned in the trash"),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(2.0),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(2.0),
    Q::Subtitle("his parents did not want him"),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(2.3),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(3.0),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(2.0),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    // sudden baby reveal, upbeat wacky music plays
    Q::Tran("baby", 60., -200., -10.),
    Q::Despawn("baby_thrown"),
    Q::Tick(1.0),
    Q::Paused("sad_song_jazz", true),
    Q::Tran("baby", 60., -200., 0.),
    Q::Tick(1.0),
];
const ANIM_CUE_WAIL: &'static [Q] = &[
    Q::Tran("baby", 60., -200., -10.),
    Q::Vol("city", 0.),
    Q::Paused("city", false),
    Q::Paused("sad_song", true),
    Q::Paused("car_idle", true),
    Q::Paused("car_brake", true),
    Q::Paused("car_win_open", true),
    Q::Paused("car_win_close", true),
    Q::Paused("car_peels_out", true),
    Q::Paused("woosh", true),
    Q::Paused("thump", true),
    Q::Subtitle("for my son"),
    // background soundscape fades in
    Q::Tick(3.),
    Q::Vol("city", 0.8),
    // scene reveal
    Q::Tick(1.),
    Q::Subtitle(""),
    Q::Despawn("screen"),
    // car moves into frame, engine sound gets louder
    Q::Tick(2.),
    Q::Tran("car", 700., -50., 0.),
    Q::Paused("car_idle", false),
    Q::Vol("car_idle", 0.),
    Q::Tick(4.),
    Q::Vol("car_idle", 0.3),
    // brake squeak
    Q::Tick(0.5),
    Q::Paused("car_brake", false),
    // car stops
    Q::Tick(0.25),
    Q::Tran("car", -50., -150., 0.),
    // window rolls down
    Q::Tick(1.),
    Q::Paused("car_win_open", false),
    // baby thrown
    Q::Tick(3.5),
    Q::Tran("baby_thrown", -30., -100., -10.),
    Q::Rot("baby_thrown", 1.5),
    Q::Paused("woosh", false),
    // baby hits ground
    Q::Tick(1.),
    Q::Tran("baby_thrown", 30., -220., 1.),
    Q::Paused("thump", false),
    // window rolls up
    Q::Tick(1.),
    Q::Paused("car_win_close", false),
    // car turns around
    Q::Tick(4.),
    Q::Flip("car", false),
    // car burnout
    Q::Tick(1.),
    Q::Tran("car", -50., -150., 0.),
    Q::Rot("car", 0.),
    Q::Paused("car_peels_out", false),
    Q::Vol("car_idle", 1.0),
    // car sound fades away
    Q::Tick(0.2),
    Q::Rot("car", 0.7),
    Q::Tick(1.8),
    Q::Tran("car", 700., -50., 0.),
    Q::Vol("car_idle", 0.),
    // somber music plays
    Q::Paused("sad_song", false),
    // hold camera for few seconds
    // camera slowly zooms in on baby
    // baby wriggles on ground
    Q::Tick(1.0), // for sad_song
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(1.5),
    Q::Subtitle("poor lonely baby"),
    Q::Tick(0.5),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(2.3),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Subtitle(""),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(0.6),
    Q::Subtitle("born in the summer"),
    Q::Tick(2.4),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(2.0),
    Q::Subtitle(""),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Subtitle("abandoned in the trash"),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(2.0),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(2.3),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Subtitle(""),
    Q::Tick(0.6),
    Q::Subtitle("his parents did not want him"),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(3.0),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(2.0),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(2.0),
    Q::Rot("baby_thrown", 1.4),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.6),
    Q::Tick(0.6),
    Q::Rot("baby_thrown", 1.5),
    Q::Tick(4.0),
    // sudden baby reveal, upbeat wacky music plays
    Q::Tran("baby", 60., -200., -10.),
    Q::Despawn("baby_thrown"),
    Q::Tick(1.0),
    Q::Paused("sad_song_jazz", true),
    Q::Tran("baby", 60., -200., 0.),
    Q::Tick(1.0),
];
const CAM_CUE: &'static [CQ] = &[
    CQ {
        time: (20., 60.),
        scale: (1., 0.4),
        tran: (Vec3::new(0., 0., 0.), Vec3::new(60., -185., 0.)),
    },
    CQ {
        time: (65., 65.5),
        scale: (0.4, 0.8),
        tran: (Vec3::new(60., -185., 0.), Vec3::new(60., -120., 0.)),
    },
];

#[derive(Component)]
pub struct Bezier(CubicSegment<Vec2>);

#[derive(Component)]
pub struct MainCamera;

#[derive(Component, Clone, Copy)]
pub struct TextureAnimate {
    pub frame_len: f32,
    pub cycle: Cycle,
    pub idx_beg: usize,
    pub idx_end: usize,
}
#[derive(Clone, Copy)]
pub enum Cycle {
    PingPong,
    Loop,
}
#[derive(Resource, Default)]
pub struct CueSequencer {
    playing: bool,
    audio: Map<Name, (Vec<(f32, f32)>, Vec<(f32, bool)>)>,
    despawn: Map<Name, f32>,
    flip: Map<Name, Vec<(f32, bool)>>,
    subtitles: Vec<(f32, &'static str)>,
    time: f32,
    end: f32,
}

impl CueSequencer {
    fn get_curve<T: Copy>(curve: &Vec<(f32, T)>, time: f32) -> Option<(T, T, f32)> {
        if curve.is_empty() {
            return None;
        }
        let mut b = curve[curve.len() - 1];
        let mut a = b;
        for i in 0..curve.len() {
            if time < curve[i].0 {
                b = curve[i];
                if i == 0 {
                    a = b;
                } else {
                    a = curve[i - 1];
                }
                let s = (time - a.0) / (b.0 - a.0);
                return Some((a.1, b.1, s));
            }
        }
        return Some((a.1, b.1, 1.));
    }

    fn get_audio(&mut self, name: &Name, time: f32) -> Option<(f32, bool)> {
        let Some((vol, paused)) = self.audio.get(name) else {
            return None;
        };
        let (vol_a, vol_b, s) = Self::get_curve(vol, time).unwrap_or((1., 1., 1.));
        let vol = vol_b * s + vol_a * (1. - s);
        let (paused, paused_b, s) = Self::get_curve(paused, time).unwrap_or((true, true, 1.));
        let paused = if s >= 1. { paused_b } else { paused };
        Some((vol, paused))
    }

    fn get_despawn(&mut self, name: &Name, time: f32) -> bool {
        if let Some(t) = self.despawn.get(name) {
            return time >= *t;
        }
        return false;
    }

    fn get_flip(&mut self, name: &Name, time: f32) -> Option<bool> {
        let Some(flips) = self.flip.get(name) else {
            return None;
        };
        let mut flip = None;
        for (t, f) in flips {
            if time >= *t {
                flip = Some(*f);
            }
        }
        flip
    }

    fn get_subtitle(&mut self, time: f32) -> &'static str {
        let (sub_cur, sub_next, s) = Self::get_curve(&self.subtitles, time).unwrap_or(("", "", 1.));
        if s >= 1. {
            sub_next
        } else {
            sub_cur
        }
    }
}

pub fn sequence_cues(
    mut names: Query<(Entity, &Name)>,
    audio: Query<&AudioSink>,
    mut subtitle: Query<&mut Text, With<Subtitle>>,
    mut sprite: Query<&mut Sprite>,
    mut commands: Commands,
    mut sequence: ResMut<CueSequencer>,
    time: Res<Time>,
    mut dbg: Query<&mut DebugUi>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !sequence.playing {
        return;
    }
    if sequence.time >= sequence.end {
        next_state.set(AppState::Game);
        return;
    }

    let mut dbg = dbg.single_mut();
    dbg.watch("time", time.elapsed_seconds());

    sequence.time += time.delta_seconds();
    let t = sequence.time;
    for (e, name) in &mut names {
        if let Some((vol, paused)) = sequence.get_audio(name, t) {
            if let Ok(sink) = audio.get(e) {
                sink.set_volume(vol);
                if sink.is_paused() && !paused {
                    sink.play();
                } else if !sink.is_paused() && paused {
                    sink.pause();
                }
            }
        }
        if sequence.get_despawn(name, t) {
            if let Some(mut ecmd) = commands.get_entity(e) {
                ecmd.despawn();
            }
        }
        if let Some(flip) = sequence.get_flip(name, t) {
            if let Ok(mut s) = sprite.get_mut(e) {
                s.flip_x = flip;
            }
        }
    }
    let mut subtitle = subtitle.single_mut();
    let seq_subtitle = sequence.get_subtitle(t);
    if subtitle.sections[0].value != seq_subtitle {
        subtitle.sections[0].value = seq_subtitle.to_string();
    }
}

pub fn sequence_camera(
    mut camera: Query<(&mut OrthographicProjection, &mut Transform, &Bezier), With<MainCamera>>,
    time: Res<Time>,
) {
    let mut cur_cq: Option<&CQ> = None;
    for cq in CAM_CUE {
        let CQ {
            time: (cq_s, sq_e), ..
        } = cq;
        let time = time.elapsed_seconds();
        if time >= *cq_s && time <= *sq_e {
            cur_cq = Some(cq);
        }
    }

    let Some(CQ {
        time: (p1_t, p2_t),
        scale: (p1_s, p2_s),
        tran: (p1_tr, p2_tr),
    }) = cur_cq
    else {
        return;
    };

    let t = time.elapsed_seconds();
    let i = inverse_lerp(*p1_t..=*p2_t, t).unwrap();

    let Ok((mut proj, mut tran, bez)) = camera.get_single_mut() else {
        return;
    };

    let ease = bez.0.ease(i);
    proj.scale = lerp(*p1_s..=*p2_s, ease);
    tran.translation = p1_tr.lerp(*p2_tr, ease);
}

pub fn animate_texture(mut tex: Query<(&mut TextureAtlas, &TextureAnimate)>, time: Res<Time>) {
    for (mut atlas, anim) in &mut tex {
        let (beg, end) = (anim.idx_beg, anim.idx_end);
        let len = end + 1 - beg;
        let n = time.elapsed_seconds() / anim.frame_len;
        let n = n as usize;
        match anim.cycle {
            Cycle::PingPong => {
                let n = n % (len * 2 - 2);
                if n < len {
                    atlas.index = beg + n;
                } else {
                    atlas.index = beg + (len - (n - len) - 2);
                }
            }
            Cycle::Loop => {
                let n = n % len;
                atlas.index = beg + n;
            }
        }
    }
}

pub fn check_kbd(
    kbd: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut quit: EventWriter<AppExit>,
) {
    if kbd.pressed(KeyCode::Space) {
        next_state.set(AppState::Game);
    }
    if kbd.pressed(KeyCode::Escape) {
        quit.send(AppExit);
    }
}

pub fn setup_anim(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut sequence: ResMut<CueSequencer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let anim_cue = if (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        % 2)
        == 0
    {
        ANIM_CUE_JAZZ
    } else {
        ANIM_CUE_WAIL
    };

    let mut pos: Map<&'static str, Vec3> = Map::new();
    let mut end = 0.;
    for cue in anim_cue.iter() {
        match cue {
            Q::Tran(name, x, y, z) => {
                if !pos.contains_key(name) {
                    pos.insert(name, Vec3::new(*x, *y, *z));
                }
            }
            Q::Tick(t) => {
                end += t;
            }
            _ => {}
        }
    }
    sequence.end = end;

    let mut entities: Map<Name, Entity> = Map::new();
    for ar in ANIM_RSC.iter() {
        match ar {
            &AR::Sprite(
                name,
                tex,
                (width, height, cols, rows, frame_len, cycle, idx_beg, idx_end),
                scale,
                flip_x,
            ) => {
                let layout =
                    TextureAtlasLayout::from_grid(Vec2::new(width, height), cols, rows, None, None);
                let name = Name::new(name);
                let layout = texture_atlas_layouts.add(layout);
                let trans = pos.get(name.as_str()).cloned().unwrap_or_default();
                let cmd = commands.spawn((
                    name.clone(),
                    SpriteBundle {
                        sprite: Sprite {
                            flip_x,
                            ..default()
                        },
                        transform: Transform {
                            translation: trans,
                            scale: Vec3::new(scale, scale, 1.),
                            ..default()
                        },
                        texture: asset_server.load(tex),
                        ..default()
                    },
                    TextureAtlas { layout, index: 0 },
                    TextureAnimate {
                        frame_len,
                        cycle,
                        idx_beg,
                        idx_end,
                    },
                ));
                entities.insert(name, cmd.id());
            }
            &AR::Image(name, tex, (x, y, z), s) => {
                let cmd = commands.spawn((
                    Name::new(name),
                    SpriteBundle {
                        transform: Transform {
                            translation: Vec3::new(x, y, z),
                            scale: Vec3::new(s, s, 1.),
                            ..default()
                        },
                        texture: asset_server.load(tex),
                        ..default()
                    },
                ));
                entities.insert(Name::new(name), cmd.id());
            }
            &AR::Overlay(name, z) => {
                let cmd = commands.spawn((
                    Name::new(name),
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(
                            meshes.add(Rectangle::new(super::WINDOW_WIDTH, super::WINDOW_HEIGHT)),
                        ),
                        material: materials.add(Color::BLACK),
                        transform: Transform::from_xyz(0., 0., z),
                        ..default()
                    },
                ));
                entities.insert(Name::new(name), cmd.id());
            }
            &AR::Sound(name, snd, once) => {
                let cmd = commands.spawn((
                    Name::new(name),
                    AudioBundle {
                        source: asset_server.load(snd),
                        settings: PlaybackSettings {
                            paused: true,
                            mode: if once {
                                PlaybackMode::Once
                            } else {
                                PlaybackMode::Loop
                            },
                            ..default()
                        },
                    },
                ));
                entities.insert(Name::new(name), cmd.id());
            }
        }
    }

    for (name, eid) in &entities {
        let mut t = 0.;

        let mut pos_next = None;
        let mut pos_steps = vec![];
        let mut pos_frames = vec![];
        let mut rot_next = None;
        let mut rot_steps = vec![];
        let mut rot_frames = vec![];

        let mut paused_next = None;
        let mut vol_next = None;
        let mut vol_cues = vec![];
        let mut play_cues = vec![];
        let mut flip_next = None;
        let mut flip_cues = vec![];

        let mut sub_next = None;
        let mut sub_cues = vec![];
        let mut despawn = None;

        for cue in anim_cue.iter() {
            match cue {
                Q::Tran(kname, x, y, z) if *kname == name.as_str() => {
                    pos_next = Some(Vec3::new(*x, *y, *z));
                }
                Q::Paused(kname, paused) if *kname == name.as_str() => {
                    paused_next = Some(*paused);
                }
                Q::Vol(kname, vol) if *kname == name.as_str() => {
                    vol_next = Some(*vol);
                }
                Q::Despawn(kname) if *kname == name.as_str() => {
                    despawn = Some(t);
                }
                Q::Rot(kname, rad) if *kname == name.as_str() => {
                    rot_next = Some(Quat::from_rotation_z(*rad));
                }
                Q::Flip(kname, flip) if *kname == name.as_str() => {
                    flip_next = Some(*flip);
                }
                Q::Subtitle(sub) => {
                    sub_next = Some(*sub);
                }
                Q::Tick(dt) => {
                    if let Some(pos_next) = pos_next.take() {
                        pos_frames.push(pos_next);
                        pos_steps.push(t);
                    }
                    if let Some(rot) = rot_next.take() {
                        rot_frames.push(rot);
                        rot_steps.push(t);
                    }
                    if let Some(vol) = vol_next.take() {
                        vol_cues.push((t, vol));
                    }
                    if let Some(paused) = paused_next.take() {
                        play_cues.push((t, paused));
                    }
                    if let Some(flip) = flip_next.take() {
                        flip_cues.push((t, flip));
                    }
                    if let Some(sub) = sub_next.take() {
                        sub_cues.push((t, sub));
                    }
                    t += dt;
                }
                _ => {}
            }
        }

        if let Some(pos_next) = pos_next {
            pos_frames.push(pos_next);
            pos_steps.push(t);
        }
        if let Some(rot) = rot_next {
            rot_frames.push(rot);
            rot_steps.push(t);
        }

        if let Some(vol) = vol_next.take() {
            vol_cues.push((t, vol));
        }
        if let Some(paused) = paused_next.take() {
            play_cues.push((t, paused));
        }

        if let Some(t) = despawn {
            sequence.despawn.insert(name.clone(), t);
        }

        if let Some(flip) = flip_next.take() {
            flip_cues.push((t, flip));
        }

        if let Some(sub) = sub_next.take() {
            sub_cues.push((t, sub));
        }

        if !(pos_frames.is_empty() && rot_frames.is_empty()) {
            let mut anim = AnimationClip::default();
            if !pos_frames.is_empty() {
                anim.add_curve_to_path(
                    EntityPath {
                        parts: vec![name.clone()],
                    },
                    VariableCurve {
                        keyframe_timestamps: pos_steps,
                        keyframes: Keyframes::Translation(pos_frames),
                        interpolation: Interpolation::Linear,
                    },
                );
            }
            if !rot_frames.is_empty() {
                anim.add_curve_to_path(
                    EntityPath {
                        parts: vec![name.clone()],
                    },
                    VariableCurve {
                        keyframe_timestamps: rot_steps,
                        keyframes: Keyframes::Rotation(rot_frames),
                        interpolation: Interpolation::Linear,
                    },
                );
            }

            let mut player = AnimationPlayer::default();
            player.play(animations.add(anim));
            commands.entity(*eid).insert(player);
        }

        if !(vol_cues.is_empty() && play_cues.is_empty()) {
            sequence.audio.insert(name.clone(), (vol_cues, play_cues));
        }

        if !flip_cues.is_empty() {
            sequence.flip.insert(name.clone(), flip_cues);
        }

        if !sub_cues.is_empty() {
            sequence.subtitles = sub_cues;
        }
    }

    commands.spawn((
        Subtitle,
        TextBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font_size: 32.,
                    ..default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(32.),
                justify_self: JustifySelf::Center,
                ..default()
            },
            ..default()
        },
    ));
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(CueSequencer {
        playing: true,
        ..default()
    });
    let camera = Name::new("camera");
    commands.spawn((
        Camera2dBundle {
            projection: OrthographicProjection {
                // When creating our own OrthographicProjection we need to set the far and near
                // values ourselves.
                // See: https://bevy-cheatbook.github.io/2d/camera.html#caveat-nearfar-values
                far: 1000.,
                near: -1000.,
                scaling_mode: ScalingMode::FixedVertical(super::WINDOW_HEIGHT),
                ..default()
            },
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        },
        MainCamera {},
        camera,
        Bezier(CubicSegment::new_bezier(
            Vec2::new(0.35, 0.), // ease-in-out bezier
            Vec2::new(0.7, 1.),
        )),
    ));

    commands.spawn((
        DebugUi::default(),
        TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.),
                left: Val::Px(10.),
                ..default()
            },
            ..default()
        },
    ));

    // Pillarboxes
    let pillarbox_h_offset = (super::WINDOW_WIDTH + super::PILLARBOX_WIDTH) / 2.;
    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Rectangle::new(super::PILLARBOX_WIDTH, super::WINDOW_WIDTH))),
        material: materials.add(Color::BLACK),
        transform: Transform::from_xyz(pillarbox_h_offset, 0., 100.),
        ..default()
    });

    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Rectangle::new(super::PILLARBOX_WIDTH, super::WINDOW_WIDTH))),
        material: materials.add(Color::BLACK),
        transform: Transform::from_xyz(-pillarbox_h_offset, 0., 100.),
        ..default()
    });
}

pub fn cleanup(
    mut commands: Commands,
    camera: Query<Entity, With<MainCamera>>,
    sprites: Query<Entity, With<Sprite>>,
    meshes: Query<Entity, With<Mesh2dHandle>>,
    sounds: Query<Entity, With<Handle<AudioSource>>>,
    subtitle: Query<Entity, With<Subtitle>>,
) {
    let camera = camera.get_single().unwrap();
    commands.entity(camera).despawn();
    for s in sprites.iter() {
        commands.entity(s).despawn();
    }
    for m in meshes.iter() {
        commands.entity(m).despawn();
    }
    for s in sounds.iter() {
        commands.entity(s).despawn();
    }
    commands.entity(subtitle.single()).despawn();
    println!("cleaning up intro");
}

fn inverse_lerp(r: std::ops::RangeInclusive<f32>, v: f32) -> Option<f32> {
    if !r.contains(&v) {
        return None;
    }
    Some((v - *r.start()) / (*r.end() - *r.start()))
}
fn lerp(r: std::ops::RangeInclusive<f32>, t: f32) -> f32 {
    if t < 0. {
        *r.start()
    } else if t > 1. {
        *r.end()
    } else {
        *r.start() + t * (*r.end() - *r.start())
    }
}
