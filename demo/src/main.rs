use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crossbeam_channel::bounded;

use crate::behaviour::make_step;
use crate::initiator::{get_player_sprite, get_visible_objects};

use ferari::assets;
use ferari::draw;
use ferari::input;
use ferari::render;
use ferari::time;
use ferari::world;
mod behaviour;
mod initiator;

use ferari::render::RenderableEntity;

/// Logical screen width in pixels.
pub const LOGIC_WIDTH: usize = 800;
/// Logical screen height in pixels.
pub const LOGIC_HEIGHT: usize = 600;
/// Tile size in pixels.
pub const TILE_SIZE: usize = 256;
/// Upscaling factor for display.
pub const UPSCALE: usize = 1;

fn main() {
    // Need to find root directory
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir.join("..");

    let assets_path = project_root.join("assets");

    // parse atlases
    let tiles_path = assets_path.join("tiles/atlas.json");
    let entities_path = assets_path.join("entities/atlas.json");

    let tiles_atlas = assets::Atlas::load(tiles_path.to_str().unwrap()).unwrap();
    let entities_atlas = assets::Atlas::load(entities_path.to_str().unwrap()).unwrap();

    // parse game descr
    let game_path = project_root.join("examples/input.json");

    let game = assets::GameMap::load(game_path).unwrap();

    // init draw
    let input_state = Arc::new(input::InputState::new());
    let running = Arc::new(AtomicBool::new(true));
    let (tx_frame, rx_frame) = bounded::<Vec<u32>>(2);

    // framebuffer (`render <-> draw` connection)
    let mut back_buffer: Vec<u32> = vec![0; LOGIC_WIDTH * LOGIC_HEIGHT];

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

    // init world_buf
    let world_width = game.size[0] as usize * TILE_SIZE * 2;
    let world_height = game.size[1] as usize * TILE_SIZE * 2;

    let world_buf: Vec<u32> = vec![195213255; world_width * world_height];
    // init render
    let shadow_map: Vec<u8> = vec![0; world_width * world_height];
    let mut render =
        render::Render::new(world_buf, world_height, world_width, entities_atlas, shadow_map);

    // init camera
    let mut camera = world::Camera::new(
        (world_width / 2) as f32,
        (world_height / 2) as f32,
        LOGIC_WIDTH as u16,
        LOGIC_HEIGHT as u16,
    );

    // init time
    let mut time = time::Time::new();

    // init state of game
    let mut state = world::State::new(&game);

    // prerender
    render.init(&game, &tiles_atlas);

    // TODO: don't need? 
    // let all_units = {
    //     let mut units = vec![state.player.unit.clone()];
    //     units.extend(state.mobs.clone());
    //     units
    // };
    // let visible_entities: Vec<RenderableEntity> = all_units
    //     .into_iter()
    //     .enumerate()
    //     .map(|(i, unit)| {
    //         let name_model = if i == 0 { "running_se" } else { "pushing_se" };
    //         let frame_count = if name_model == "running_se" { 13 } else { 20 };
    //         let period = 0.1;
    //         let cycles = (time.total / period).floor() as u32;
    //         let full_name = format!("{}_{}", name_model, cycles % frame_count);
    //         RenderableEntity::new(unit.pixel_x, unit.pixel_y, full_name)
    //     })
    //     .collect();
    // render.render_frame(&visible_entities, &camera, &mut back_buffer);

    // game loop
    while running.load(Ordering::Acquire) {
        time.update();

        // test gradient (TODO: move to tests?)
        // let r = ((time.total).sin() * 127.0 + 128.0) as u32;
        // let g = ((time.total + 2.0).sin() * 127.0 + 128.0) as u32;
        // let b = ((time.total + 4.0).sin() * 127.0 + 128.0) as u32;
        // let color = (r << 16) | (g << 8) | b;

        // for px in back_buffer.iter_mut() {
        //     *px = color;
        // }

        // process input
        let input = input_state.read();
        if input.escape {
            running.store(false, Ordering::Release);
        }

        make_step(&mut state, &input, time.delta, &game);

        camera.center_x = state.player.unit.pixel_x.floor();
        camera.center_y = state.player.unit.pixel_y.floor();

        let units_for_render = get_visible_objects(&state, &camera);

        if units_for_render.is_empty() {
            continue;
        }

        // frame render
        let visible_entities: Vec<RenderableEntity> = units_for_render
            .into_iter()
            .enumerate()
            .map(|(i, unit)| {
                let sprite_name = if i == 0 {
                    get_player_sprite(&state.player, time.total as f64)
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
        thread::sleep(Duration::from_micros(16667)); // ~60 FPS
    }

    println!("Main loop exited");
}
