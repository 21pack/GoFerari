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
pub const MOVEMENT_SPEEDUP: f32 = 1.0;
/// Upscaling factor for display.
pub const UPSCALE: usize = 2;
/// Logical screen width in pixels.
pub const LOGIC_WIDTH: usize = 1280 / UPSCALE;
/// Logical screen height in pixels.
pub const LOGIC_HEIGHT: usize = 720 / UPSCALE;
/// Tile size in pixels.
pub const TILE_SIZE: usize = 128;
/// Filesystem paths to all available game levels.
const LEVEL_PATHS: &[&str] = &[
    "game_levels/menu.json",
    "game_levels/level1.json",
    "game_levels/level2.json",
    "game_levels/level3.json",
    "game_levels/level4.json",
    "game_levels/level5.json",
];
/// Target frame duration for a stable gameplay loop.
const FRAME_TIME: Duration = Duration::from_micros(16667); // ~60 FPS

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

/// Initializes all core systems required to start a new game level.
///
/// This function sets up the rendering pipeline, camera, and game state based on
/// the provided level data and graphical assets.
///
/// # Arguments
///
/// * `game` – the preloaded game map containing layout, walkability, and object placement
/// * `entities_atlas` – texture atlas containing sprites for dynamic entities
/// * `tiles_atlas` – texture atlas containing static tile graphics
///
/// # Returns
///
/// A tuple containing:
/// * [`ferari::Render`] - the fully initialized renderer with pre-rendered static background.
/// * [`world::Camera`] - a camera centered on the level, configured to the logical screen dimensions.
/// * [`world::State`] - the initial game state (player position, mob positions, grid occupancy, etc.).
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
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");

    let assets_path = project_root.join("assets");

    // parse atlases
    let tiles_path = assets_path.join("tiles/atlas.json");
    let entities_path = assets_path.join("entities/atlas.json");

    let tiles_atlas = assets::Atlas::load(tiles_path.to_str().unwrap()).unwrap();
    let entities_atlas = assets::Atlas::load(entities_path.to_str().unwrap()).unwrap();

    // parse game descr
    let menu_path = project_root.join("game_levels/menu.json");
    let mut game = assets::GameMap::load(menu_path).unwrap();

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

    // Platform-specific draw thread

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
        init_level(game.clone(), entities_atlas.clone(), tiles_atlas.clone());

    // game loop
    while running.load(Ordering::Acquire) {
        if cur_level != cur_level2 {
            let level_path = project_root.join(LEVEL_PATHS[cur_level2 as usize]);

            let loaded_game = assets::GameMap::load(level_path).unwrap();
            game = loaded_game;
            cur_level = cur_level2;

            (render, camera, state) =
                init_level(game.clone(), entities_atlas.clone(), tiles_atlas.clone());
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

        // menu
        if cur_level == 0 {
            let player_idle = matches!(state.player.unit.movement, world::UnitMovement::Idle);

            for unit in &state.mobs {
                if matches!(unit.movement, world::UnitMovement::Idle) & player_idle {
                    let pos = (unit.tile_x as u32, unit.tile_y as u32);

                    if let Some(&id) = game.links.get(&pos) {
                        cur_level2 = id;
                        suc_boxes.push((unit.tile_x, unit.tile_y));
                        break;
                    }
                }
            }
        }
        // gameplay level: check box placement and win condition
        else {
            let goal_count = game.target_positions.len();

            if goal_count == 0 {
                cur_level2 = 0;
            } else {
                let mut placed_count = 0;
                let player_idle = matches!(state.player.unit.movement, world::UnitMovement::Idle);

                for unit in &state.mobs {
                    if matches!(unit.movement, world::UnitMovement::Idle) {
                        let pos = (unit.tile_x as u32, unit.tile_y as u32);

                        if game.target_positions.contains(&pos) {
                            suc_boxes.push((unit.tile_x, unit.tile_y));
                            placed_count += 1;
                        }
                    }
                }

                if placed_count == goal_count && player_idle {
                    cur_level2 = 0;
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
                } else if suc_boxes.contains(&(unit.tile_x, unit.tile_y)) {
                    "green_box".to_string()
                } else {
                    "box".to_string()
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
        thread::sleep(FRAME_TIME);
    }

    println!("Main loop exited");
}
