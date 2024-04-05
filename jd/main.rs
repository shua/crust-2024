use bevy::{
    app::AppExit,
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
    window::PrimaryWindow,
};

#[derive(Component)]
struct Control;
#[derive(Component)]
struct Collide;
#[derive(Resource)]
struct UpdateRemainder(f32, bool);
#[derive(Component, Default)]
struct Movement {
    ctl: Vec2,
    force: Vec2,
    out: Vec2,
}
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug)]
struct Tile(u8);
#[derive(Event)]
struct Quit;
#[derive(Resource, Default, Deref)]
struct TileSprites(Vec<(Color, Handle<Image>)>);

#[derive(Resource, Default)]
struct DebugInfo {
    text: Vec<TextSection>,
    collisions: Vec<(Aabb2d, (Vec2, Vec2))>,
    ctl_aabb: Option<Aabb2d>,
    cursor: Vec2,
}
#[derive(Component)]
struct DebugUi;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .insert_resource(DebugInfo::default())
        .insert_resource(UpdateRemainder(0., false))
        .insert_resource(TileSprites(vec![(Color::NONE, Handle::default())]))
        .add_event::<Quit>()
        .add_systems(Startup, setup_graphics)
        .add_systems(
            Update,
            (
                check_kbd,     //
                check_collide, //
                update_movement,
            )
                .chain(),
        )
        .add_systems(Update, (check_mouse, on_quit))
        .add_systems(PostUpdate, draw_debug)
        .run();
}

const TILE_SZ: f32 = 50.;
const MAP: (Vec2, usize, [u8; 8 * 8]) = (
    Vec2::new(-200., -200.),
    8,
    [
        1, 1, 1, 1, 1, 1, 1, 1, // 1
        1, 0, 0, 0, 0, 0, 0, 1, // 2
        1, 0, 0, 0, 0, 0, 0, 1, // 3
        1, 0, 0, 0, 0, 0, 0, 1, // 4
        1, 0, 0, 0, 0, 1, 0, 1, // 5
        1, 0, 0, 0, 0, 0, 0, 1, // 6
        1, 0, 0, 0, 0, 1, 0, 1, // 7
        1, 1, 1, 1, 1, 1, 1, 1, // 8
    ],
);

impl Tile {
    fn new_bundle(t: u8, pos: Vec3, tile_sprites: &TileSprites) -> (Collide, Tile, SpriteBundle) {
        (
            Collide,
            Tile(t),
            SpriteBundle {
                sprite: Sprite {
                    color: tile_sprites[t as usize].0,
                    custom_size: Some(Vec2::ONE),
                    ..default()
                },
                transform: Transform {
                    translation: pos,
                    scale: Vec3::new(TILE_SZ, TILE_SZ, 1.),
                    ..default()
                },
                texture: tile_sprites[t as usize].1.clone(),
                ..default()
            },
        )
    }
}

fn setup_graphics(
    mut command: Commands,
    assets: Res<AssetServer>,
    mut win: Query<&mut Window, With<PrimaryWindow>>,
    mut tile_sprites: ResMut<TileSprites>,
) {
    command.spawn(Camera2dBundle::default());
    command.spawn((
        DebugUi,
        Text2dBundle {
            text: Text::default(),
            text_anchor: bevy::sprite::Anchor::TopLeft,
            transform: Transform {
                translation: Vec3::new(-300., 300., 0.),
                scale: Vec3::ONE,
                ..default()
            },
            ..default()
        },
    ));

    command.spawn((
        Control,
        Collide,
        Movement::default(),
        SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(0.8, 1.)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., 0., 1.),
                scale: Vec3::new(45., 45., 1.),
                ..default()
            },
            texture: assets.load("baby.png"),
            ..default()
        },
    ));

    tile_sprites.0.extend([
        (Color::PURPLE, Handle::default()),
        (Color::ORANGE, Handle::default()),
        (Color::YELLOW, Handle::default()),
    ]);
    let map_origin = MAP.0.extend(0.);
    for (i, &t) in MAP.2.iter().rev().enumerate() {
        if t == 0 {
            continue;
        }
        let (x, y) = (MAP.1 - (i % MAP.1) - 1, i / MAP.1);
        let (x, y) = (x as f32 * TILE_SZ, y as f32 * TILE_SZ);
        command.spawn(Tile::new_bundle(
            t,
            map_origin + Vec3::new(x, y, 0.),
            &tile_sprites,
        ));
    }

    for mut win in &mut win {
        win.cursor.icon = CursorIcon::Pointer;
        win.cursor.visible = true;
    }
}

fn check_kbd(
    kbd: Res<ButtonInput<KeyCode>>,
    mut quit: EventWriter<Quit>,
    mut ctl: Query<&mut Movement, With<Control>>,
) {
    if kbd.pressed(KeyCode::Escape) {
        quit.send(Quit);
    }

    let mut vx = 0.;
    let mut vy = 0.;
    if kbd.pressed(KeyCode::ArrowLeft) {
        vx -= 1.;
    }
    if kbd.pressed(KeyCode::ArrowRight) {
        vx += 1.;
    }
    if kbd.pressed(KeyCode::ArrowUp) {
        vy += 1.;
    }
    if kbd.pressed(KeyCode::ArrowDown) {
        vy -= 1.;
    }

    let v = Vec2::new(vx, vy);
    for mut c in &mut ctl {
        c.ctl = v * 5.;
    }
}

fn check_mouse(
    mouse: Res<ButtonInput<MouseButton>>,
    win: Query<&Window, With<PrimaryWindow>>,
    cam: Query<(&Camera, &GlobalTransform)>,
    mut tiles: Query<(
        Entity,
        &Transform,
        &mut Sprite,
        &mut Handle<Image>,
        &mut Tile,
    )>,
    tile_sprites: Res<TileSprites>,
    mut commands: Commands,
    mut dbg: ResMut<DebugInfo>,
) {
    let (cam, cam_trans) = cam.single();
    let Some(cursor) = win.single().cursor_position() else {
        return;
    };
    let Some(cursor) = cam.viewport_to_world_2d(cam_trans, cursor) else {
        return;
    };

    dbg.cursor = cursor;

    if mouse.just_pressed(MouseButton::Left) {
        let cursor_pt = Aabb2d::new(cursor, Vec2::ZERO);
        for (e, trans, mut s, mut img, mut tile) in &mut tiles {
            let tile_box = Aabb2d::new(trans.translation.xy(), trans.scale.xy() / 2.);
            if !tile_box.contains(&cursor_pt) {
                continue;
            }

            tile.0 = (tile.0 + 1) % (tile_sprites.len() as u8);
            if tile.0 == 0 {
                commands.get_entity(e).unwrap().despawn();
            } else {
                s.color = tile_sprites.0[tile.0 as usize].0;
                *img = tile_sprites.0[tile.0 as usize].1.clone();
            }
            return;
        }

        // no tile, need to insert
        let tile_pos = (cursor / TILE_SZ).round() * TILE_SZ;
        commands.spawn(Tile::new_bundle(1, tile_pos.extend(0.), &tile_sprites));
    }
}

// the intent is to cast the ctl's aabb along ctl's velocity and check for any collisions
// if there are any collisions, then reduce velocity until there aren't
//
// this is not working correctly as it sees collisions where it shouldn't
fn check_collide(
    time: Res<Time>,
    mut update_rem: ResMut<UpdateRemainder>,
    mut ctl: Query<(Entity, &Transform, &mut Movement, &mut Sprite), With<Control>>,
    col: Query<(Entity, &Transform), With<Collide>>,
    mut dbg: ResMut<DebugInfo>,
) {
    for (e, t, mut v, mut s) in &mut ctl {
        if v.ctl + v.force == Vec2::ZERO {
            v.out = Vec2::ZERO;
            continue;
        }

        let (mut dt, mut push_vert) = (update_rem.0, update_rem.1);
        dt += time.delta_seconds() * 60.;
        let mut aabb = Aabb2d::new(t.translation.xy(), t.scale.xy() / 2.);
        while dt > 1. {
            let mut collisions = vec![];
            v.force += Vec2::new(0., -9.8 / 60.);
            aabb = Aabb2d::new(aabb.center() + v.ctl.xy() + v.force.xy(), aabb.half_size());
            for (ec, col) in &col {
                if ec == e {
                    continue;
                }

                let col_aabb = Aabb2d::new(col.translation.xy(), col.scale.xy() / 2.);
                if aabb.intersects(&col_aabb) {
                    collisions.push((col_aabb, (aabb.center(), col_aabb.center() - aabb.center())));
                    let left = col_aabb.min.x - aabb.max.x;
                    let right = col_aabb.max.x - aabb.min.x;
                    let up = col_aabb.max.y - aabb.min.y;
                    let down = col_aabb.min.y - aabb.max.y;
                    let horz = if left.abs() < right.abs() {
                        left
                    } else {
                        right
                    };
                    let vert = if up.abs() < down.abs() { up } else { down };
                    let push = match horz.abs().total_cmp(&vert.abs()) {
                        std::cmp::Ordering::Greater => {
                            v.force.y = 0.;
                            Vec2::new(0., vert)
                        }
                        std::cmp::Ordering::Less => {
                            v.force.x = 0.;
                            Vec2::new(horz, 0.)
                        }
                        std::cmp::Ordering::Equal => {
                            // stuck on a corner, if we always choose one or the other than there will be no progress
                            // so we choose vertical half the time and horizontal the other half
                            push_vert = !push_vert;
                            if push_vert {
                                v.force.y = 0.;
                                Vec2::new(0., vert)
                            } else {
                                v.force.x = 0.;
                                Vec2::new(horz, 0.)
                            }
                        }
                    };
                    aabb = Aabb2d::new(aabb.center() + push, aabb.half_size());
                }
            }

            if !collisions.is_empty() {
                s.color = Color::rgb(1., 0., 0.);

                dbg.text.clear();
                dbg.text.push(TextSection::new(
                    format!("vctl: {:?}, vforce: {:?}\n", v.ctl, v.force),
                    default(),
                ));
                dbg.collisions = collisions;
                dbg.ctl_aabb = Some(Aabb2d::new(t.translation.xy() + v.ctl, t.scale.xy() / 2.));
            } else {
                s.color = Color::rgb(0., 0., 1.);
            }

            dt -= 1.;
        }
        let tnew = aabb.center();
        v.out = tnew - t.translation.xy();
        if (dt, push_vert) != (update_rem.0, update_rem.1) {
            update_rem.0 = dt;
            update_rem.1 = push_vert;
        }
    }
}

fn update_movement(mut movers: Query<(&mut Transform, &Movement)>) {
    for (mut t, v) in &mut movers {
        t.translation.x += v.out.x;
        t.translation.y += v.out.y;

        // kill box
        if t.translation.y < -1000. {
            t.translation = Vec3::ZERO;
        }
    }
}

fn draw_debug(dbg: Res<DebugInfo>, mut gizmos: Gizmos, mut ui: Query<&mut Text, With<DebugUi>>) {
    if dbg.is_changed() && !dbg.text.is_empty() {
        for mut ui in &mut ui {
            ui.sections = dbg.text.clone();
        }
    }
    if !dbg.collisions.is_empty() {
        for (aabb, (_origin, _ray)) in dbg.collisions.iter() {
            gizmos.rect_2d(aabb.center(), 0., aabb.half_size() * 2., Color::RED);
            // gizmos.ray_2d(*origin, *ray, Color::GREEN);
        }
        if let Some(aabb) = &dbg.ctl_aabb {
            gizmos.rect_2d(aabb.center(), 0., aabb.half_size() * 2., Color::GREEN);
        }
    }
    let cursor = (dbg.cursor / TILE_SZ).round() * TILE_SZ;
    gizmos.rect_2d(cursor, 0., Vec2::new(TILE_SZ, TILE_SZ), Color::GREEN);
}

fn on_quit(
    quit: EventReader<Quit>,
    tiles: Query<(&Transform, &Tile), With<Tile>>,
    mut exit: EventWriter<AppExit>,
) {
    if !quit.is_empty() {
        let mut data: Vec<_> = tiles
            .iter()
            .map(|(t, s)| (t.translation.xy(), *s))
            .collect();
        data.sort_by(|(t1, _), (t2, _)| match t1.y.total_cmp(&t2.y) {
            std::cmp::Ordering::Equal => t1.x.total_cmp(&t2.x),
            c => c,
        });
        let mut min = data[0].0;
        let mut max = data[data.len() - 1].0;
        for (d, _) in &data {
            if d.x < min.x {
                min.x = d.x;
            }
            if d.x > max.x {
                max.x = d.x;
            }
        }

        let width = ((max.x - min.x) / TILE_SZ) as usize + 1;
        let height = ((max.y - min.y) / TILE_SZ) as usize + 1;
        println!("const MAP: (Vec2, usize, [u8; {width} * {height}]) = (");
        println!("  Vec2::new({:?}, {:?}),", min.x.floor(), min.y.floor());
        println!("  {width},");
        println!("  [");
        let mut map = vec![vec![0u8; width]; height];
        for (trans, tile) in data {
            let trans = (trans - min) / TILE_SZ;
            map[trans.y as usize][trans.x as usize] = tile.0;
        }
        for (y, row) in map.into_iter().rev().enumerate() {
            print!("    ");
            for t in row {
                print!("{t}, ");
            }
            println!(" // {y}");
        }
        println!("  ],");
        println!(");");

        exit.send(AppExit);
    }
}