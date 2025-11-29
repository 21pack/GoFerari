use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(target_os = "macos")]
use minifb::{Key, Window, WindowOptions};

use crossbeam_channel::bounded;

use crate::behaviour::make_step;
use crate::initiator::{get_player_sprite, get_visible_objects};

use ferari::assets;
#[cfg(target_os = "linux")]
use ferari::draw;
use ferari::input;
use ferari::render;
use ferari::time;
use ferari::world;
mod behaviour;
mod initiator;

use ferari::render::RenderableEntity;

/// Animation and movement speedup
pub const MOVEMENT_SPEEDUP: f32 = 1.5;
/// Upscaling factor for display.
pub const UPSCALE: usize = 1;
/// Logical screen width in pixels.
pub const LOGIC_WIDTH: usize = 800 / UPSCALE;
/// Logical screen height in pixels.
pub const LOGIC_HEIGHT: usize = 600 / UPSCALE;
/// Tile size in pixels.
pub const TILE_SIZE: usize = 128;

#[cfg(target_os = "macos")]
macro_rules! update_window {
    ($window:ident, $running:ident, $rx_frame:ident, $input_state:ident, $width:ident, $height:ident) => {
        // handle input in this thread
        $input_state.update(&$window);

        if let Ok(frame) = $rx_frame.try_recv() {
            $window.update_with_buffer(&frame, $width, $height).unwrap();
        } else {
            $window.update();
        }

        if $window.is_key_down(Key::Escape) || !$window.is_open() {
            $running.store(false, Ordering::Release);
            break;
        }

        thread::sleep(Duration::from_millis(1));
    };
}

fn init_level(
    game: assets::GameMap,
    entities_atlas: assets::Atlas,
    tiles_atlas: assets::Atlas,
) -> (ferari::Render, world::Camera, world::State) {
    // init world_buf
    let world_width = game.size[0] as usize * TILE_SIZE * 2;
    let world_height = game.size[1] as usize * TILE_SIZE * 2;
    let world_buf: Vec<u32> = vec![195213255; world_width * world_height];

    // init render
    let shadow_map: Vec<u8> = vec![0; world_width * world_height];
    let mut render =
        render::Render::new(world_buf, world_height, world_width, entities_atlas, shadow_map);

    // init camera
    let camera = world::Camera::new(
        (world_width / 2) as f32,
        (world_height / 2) as f32,
        LOGIC_WIDTH as u16,
        LOGIC_HEIGHT as u16,
    );

    // init state of game
    let state = world::State::new(&game);

    // prerender
    render.init(&game, &tiles_atlas);

    (render, camera, state)
}

fn main() {
    // Need to find root directory
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir.join("..");

    let assets_path = project_root.join("assets");

    // parse atlases
    let tiles_path = assets_path.join("tiles/atlas.json");
    let entities_path = assets_path.join("entities/atlas.json");
    // let alphabet_path = assets_path.join("alphabet/atlas.json");

    let ground_atlas = assets::Atlas::load(tiles_path.to_str().unwrap()).unwrap();
    let entities_atlas = assets::Atlas::load(entities_path.to_str().unwrap()).unwrap();
    // let alphabet_atlas = assets::Atlas::load(alphabet_path.to_str().unwrap()).unwrap();
    let tiles_atlas = ground_atlas.clone();

    // parse game descr
    let menu_path = project_root.join("examples/menu.json");
    let game0 = assets::GameMap::load(menu_path).unwrap();
    let mut game = game0.clone();

    let mut cur_level = 0;
    let mut cur_level2 = 0;

    // init draw
    let input_state = Arc::new(input::InputState::new());
    let running: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let (tx_frame, rx_frame) = bounded::<Vec<u32>>(2);

    // framebuffer (`render <-> draw` connection)
    let mut back_buffer: Vec<u32> = vec![0; LOGIC_WIDTH * LOGIC_HEIGHT];

    // init time
    let mut time = time::Time::new();

    #[cfg(target_os = "linux")]
    {
        let input_state = input_state.clone();
        let running = running.clone();

        thread::spawn(move || {
            draw::run_draw_thread(
                rx_frame,
                input_state,
                running,
                LOGIC_WIDTH,
                LOGIC_HEIGHT,
                UPSCALE,
            );
        });
    }

    #[cfg(target_os = "macos")]
    let mut window = Window::new(
        "Ferari",
        LOGIC_WIDTH * UPSCALE,
        LOGIC_HEIGHT * UPSCALE,
        WindowOptions::default(),
    )
    .unwrap();

    let (mut render, mut camera, mut state) =
        init_level(game.clone(), entities_atlas.clone(), tiles_atlas);

    // cringe cast since cringe types :)
    fn cast(tile_z: i32) -> u32 {
        u32::try_from(tile_z).ok().expect("fail bounds")
    }

    // game loop
    while running.load(Ordering::Acquire) {
        if cur_level != cur_level2 {
            let level_path = match cur_level2 {
                0 => "examples/menu.json",
                1 => "examples/level1.json",
                2 => "examples/level2.json",
                3 => "examples/level3.json",
                4 => "examples/level4.json",
                _ => continue,
            };

            let level_path = project_root.join(level_path);
            let loaded_game = assets::GameMap::load(level_path).unwrap();
            let tiles_atlas =
                if cur_level2 == 0 { ground_atlas.clone() } else { ground_atlas.clone() };
            game = loaded_game.clone();
            cur_level = cur_level2;

            (render, camera, state) = init_level(game.clone(), entities_atlas.clone(), tiles_atlas);
        }

        #[cfg(target_os = "macos")]
        update_window!(window, running, rx_frame, input_state, LOGIC_WIDTH, LOGIC_HEIGHT);

        time.update();

        // process input
        let input = input_state.read();
        if input.escape {
            running.store(false, Ordering::Release);
        }

        match make_step(&mut state, &input, time.delta, &game) {
            None => (),
            Some(id) => cur_level2 = id,
        }

        camera.center_x = state.player.unit.pixel_x.floor();
        camera.center_y = state.player.unit.pixel_y.floor();

        let units_for_render = get_visible_objects(&state, &camera);

        if units_for_render.is_empty() {
            continue;
        }

        let mut suc_boxes = vec![];
        let player = state.player.unit.clone();
        // TODO refactor then and else
        // menu
        if cur_level == 0 {
            let mut pos = 0;
            loop {
                match state.mobs.get(pos) {
                    None => break,
                    // not selected yet
                    Some(unit) => {
                        let (x, y) = (cast(unit.tile_x), cast(unit.tile_y));
                        match (
                            unit.movement.clone(),
                            player.movement.clone(),
                            game.links.get(&(x, y)),
                        ) {
                            (world::UnitMovement::Idle, world::UnitMovement::Idle, Some(&id)) => {
                                cur_level2 = id;
                                suc_boxes.push((unit.tile_x, unit.tile_y));
                                break;
                            }
                            _ => (),
                        }
                        pos += 1;
                    }
                }
            }
        }
        // check if should to up the level
        else {
            let goal = game.target_positions.len();

            if goal == 0 {
                cur_level2 = 0;
            }
            let mut pos = 0;
            let mut acc = goal;
            loop {
                match state.mobs.get(pos) {
                    None => break,
                    // not finished yet
                    Some(unit) => {
                        let (x, y) = (cast(unit.tile_x), cast(unit.tile_y));
                        match unit.movement.clone() {
                            world::UnitMovement::Idle
                                if game.target_positions.contains(&(x, y)) =>
                            {
                                suc_boxes.push((unit.tile_x, unit.tile_y));
                                acc -= 1;
                            }
                            _ => (),
                        };
                        if acc == 0 {
                            match player.movement.clone() {
                                world::UnitMovement::Idle => {
                                    cur_level2 = 0;
                                    break;
                                }
                                _ => (),
                            };
                        }
                        pos += 1;
                    }
                }
            }
        }

        // frame render
        let visible_entities: Vec<RenderableEntity> = units_for_render
            .into_iter()
            .enumerate()
            .map(|(i, unit)| {
                let sprite_name = if i == 0 {
                    get_player_sprite(&state.player, time.total as f64)
                } else {
                    if suc_boxes.contains(&(unit.tile_x, unit.tile_y)) {
                        "green_box".to_string()
                    } else {
                        "box".to_string()
                    }
                };

                RenderableEntity::new(unit.pixel_x, unit.pixel_y, sprite_name)
            })
            .collect();

        render.render_frame(&visible_entities, &camera, &mut back_buffer);

        // draw frame
        if tx_frame.try_send(back_buffer.clone()).is_err() {
            // idle
        }

        // fps limit
        thread::sleep(Duration::from_micros(16667)); // ~60 FPS
    }

    println!("Main loop exited");
}
