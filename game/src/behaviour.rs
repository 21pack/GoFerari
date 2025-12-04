use crate::{initiator::lerp, input::InputSnapshot, MOVEMENT_SPEEDUP, TILE_SIZE};

use ferari::world::{Direction, State, Unit, UnitMovement};

/// Data required to start a box-pushing animation.
type PushTransition = (usize, i32, i32, i32, i32, i32, i32);

/// Pixel coordinates for recoil animation (start_x, start_y, target_x, target_y).
type PostPushTransition = (f32, f32, f32, f32);

/// Result of player animation update: busy status and optional transition data.
#[derive(Default)]
struct PlayerAnimationResult {
    /// Whether the player is currently busy with an animation
    pub player_is_busy: bool,
    /// Parameters to initiate a box-pushing sequence
    pub transition_to_push: Option<PushTransition>,
    /// Pixel coordinates to start a recoil animation
    pub transition_to_post: Option<PostPushTransition>,
}

/// Time to walk one tile (player walking).
const MOVE_DURATION: f32 = 0.35 / MOVEMENT_SPEEDUP;
/// Time to approach a box before pushing.
const PRE_PUSH_DURATION: f32 = 0.40 / MOVEMENT_SPEEDUP;
/// Recoil animation time after pushing (not speed-scaled).
const POST_PUSH_DURATION: f32 = 0.50;
/// Time for a box to slide one tile.
const BOX_MOVE_DURATION: f32 = 1.25 / MOVEMENT_SPEEDUP;
/// Duration of the player's pushing animation.
const PUSH_DURATION: f32 = BOX_MOVE_DURATION;
/// Offset between a box and the pushing player.
const PUSH_OFFSET: f32 = 0.25;

/// Computes the pixel offset (in isometric coordinates) for moving in a given direction by a specified magnitude.
///
/// The direction is specified as integer deltas (`dir_x`, `dir_y`), where only one of the components is
/// non-zero (e.g., `(1, 0)` for southeast).
/// The `magnitude` parameter scales the movement: `1.0` corresponds to moving exactly one tile.
///
/// # Arguments
///
/// * `dir_x`, `dir_y` - The X and Y component of the direction (-1, 0, or 1)
/// * `magnitude` - The scaling factor for the movement
///
/// # Returns
///
/// A tuple `(offset_x, offset_y)` in pixel coordinates.
fn get_offset(dir_x: i32, dir_y: i32, magnitude: f32) -> (f32, f32) {
    let tile_w = TILE_SIZE as f32;
    let tile_h = (TILE_SIZE as f32) * 0.5;

    let step_x = tile_w * 0.5 * magnitude;
    let step_y = tile_h * 0.5 * magnitude;

    match (dir_x, dir_y) {
        (1, 0) => (step_x, step_y),
        (-1, 0) => (-step_x, -step_y),
        (0, -1) => (step_x, -step_y),
        (0, 1) => (-step_x, step_y),
        _ => (0.0, 0.0),
    }
}

/// Maps a logical `Direction` enum value to its corresponding tile-based movement delta.
///
/// # Arguments
///
/// * `dir` - The direction to convert
///
/// # Returns
///
/// A tuple `(dx, dy)` where:
/// - `dx`: change in tile X-coordinate (+1 for SE/NW axis, 0 otherwise)
/// - `dy`: change in tile Y-coordinate (+1 for SW/NE axis, 0 otherwise)
fn get_dir_delta(dir: Direction) -> (i32, i32) {
    match dir {
        Direction::SE => (1, 0),
        Direction::NW => (-1, 0),
        Direction::NE => (0, -1),
        Direction::SW => (0, 1),
    }
}

/// Updates the player's animation state based on elapsed time and input.
///
/// This function handles all player animation phases: idle, walking, pre-pushing,
/// pushing (including chain-push logic), and post-pushing recoil.
///
/// # Arguments
///
/// * `unit` - mutable reference to the player's unit to update its pixel position and movement state
/// * `delta` - time elapsed since the last frame (in seconds)
/// * `input_state` - current input snapshot used to detect continuous push input
/// * `mob_grid` - read-only grid indicating which mob (if any) occupies each tile
/// * `game` - game map used for walkability and collision checks during chain pushes
/// * `map_width` - width of the map in tiles
///
/// # Returns
///
/// A [`PlayerAnimationResult`] containing:
/// * `player_is_busy`: `true` if the player is currently in any non-idle animation.
/// * `transition_to_push`: parameters to initiate a box-pushing sequence
///   (if a PrePushing or Pushing animation just completed and chain-push is possible).
/// * `transition_to_post`: pixel coordinates to start a recoil animation
///   (if Pushing just completed without chain-push).
fn update_player_animation(
    unit: &mut Unit,
    delta: f32,
    input_state: &InputSnapshot,
    mob_grid: &[Option<usize>],
    game: &ferari::assets::GameMap,
    map_width: usize,
) -> PlayerAnimationResult {
    let mut player_is_busy = false;

    // Walking between phases of approaching or going back to box/tile.
    let mut transition_to_push: Option<(usize, i32, i32, i32, i32, i32, i32)> = None;
    let mut transition_to_post: Option<(f32, f32, f32, f32)> = None;

    match &mut unit.movement {
        UnitMovement::Moving { start_x, start_y, target_x, target_y, elapsed_time, duration } => {
            *elapsed_time += delta;
            let progress = (*elapsed_time / *duration).min(1.0);
            unit.pixel_x = lerp(*start_x, *target_x, progress);
            unit.pixel_y = lerp(*start_y, *target_y, progress);

            if progress >= 1.0 {
                unit.pixel_x = *target_x;
                unit.pixel_y = *target_y;
                unit.movement = UnitMovement::Idle;
            } else {
                player_is_busy = true;
            }
        }

        UnitMovement::PrePushing {
            start_x,
            start_y,
            target_x,
            target_y,
            elapsed_time,
            duration,
            box_idx,
            player_next_tx,
            player_next_ty,
            box_next_tx,
            box_next_ty,
            push_dx,
            push_dy,
        } => {
            *elapsed_time += delta;
            let progress = (*elapsed_time / *duration).min(1.0);
            unit.pixel_x = lerp(*start_x, *target_x, progress);
            unit.pixel_y = lerp(*start_y, *target_y, progress);

            if progress >= 1.0 {
                transition_to_push = Some((
                    *box_idx,
                    *player_next_tx,
                    *player_next_ty,
                    *box_next_tx,
                    *box_next_ty,
                    *push_dx,
                    *push_dy,
                ));
                unit.pixel_x = *target_x;
                unit.pixel_y = *target_y;
                player_is_busy = true;
            } else {
                player_is_busy = true;
            }
        }

        UnitMovement::Pushing {
            start_x,
            start_y,
            target_x,
            target_y,
            elapsed_time,
            duration,
            recoil_target_x,
            recoil_target_y,
        } => {
            *elapsed_time += delta;
            let progress = (*elapsed_time / *duration).min(1.0);
            unit.pixel_x = lerp(*start_x, *target_x, progress);
            unit.pixel_y = lerp(*start_y, *target_y, progress);

            if progress >= 1.0 {
                unit.pixel_x = *target_x;
                unit.pixel_y = *target_y;

                let (dx, dy) = get_dir_delta(unit.direction);

                let continuing_input = match unit.direction {
                    Direction::SE => input_state.right,
                    Direction::NW => input_state.left,
                    Direction::NE => input_state.up,
                    Direction::SW => input_state.down,
                };

                let mut can_continue = false;

                if continuing_input {
                    let current_p_tx = unit.tile_x;
                    let current_p_ty = unit.tile_y;

                    let next_box_tx = current_p_tx + (dx * 2);
                    let next_box_ty = current_p_ty + (dy * 2);

                    let next_p_tx = current_p_tx + dx;
                    let next_p_ty = current_p_ty + dy;

                    let walkable = game.is_walkable(next_box_tx, next_box_ty)
                        && !game.has_collidable_object_at(next_box_tx, next_box_ty);

                    if next_box_ty >= 0 && next_box_tx >= 0 {
                        let next_box_idx =
                            (next_box_ty as usize) * map_width + (next_box_tx as usize);
                        let mob_blocking = mob_grid.get(next_box_idx).copied().flatten().is_some();

                        if walkable && !mob_blocking {
                            let current_box_pos_idx =
                                (next_p_ty as usize) * map_width + (next_p_tx as usize);

                            if let Some(box_idx) = mob_grid[current_box_pos_idx] {
                                can_continue = true;
                                transition_to_push = Some((
                                    box_idx,
                                    next_p_tx,
                                    next_p_ty,
                                    next_box_tx,
                                    next_box_ty,
                                    dx,
                                    dy,
                                ));
                            }
                        }
                    }
                }

                if !can_continue {
                    transition_to_post =
                        Some((*target_x, *target_y, *recoil_target_x, *recoil_target_y));
                }

                player_is_busy = true;
            } else {
                player_is_busy = true;
            }
        }

        UnitMovement::PostPushing {
            start_x,
            start_y,
            target_x,
            target_y,
            elapsed_time,
            duration,
        } => {
            *elapsed_time += delta;
            let progress = (*elapsed_time / *duration).min(1.0);
            unit.pixel_x = lerp(*start_x, *target_x, progress);
            unit.pixel_y = lerp(*start_y, *target_y, progress);

            if progress >= 1.0 {
                unit.pixel_x = *target_x;
                unit.pixel_y = *target_y;
                unit.movement = UnitMovement::Idle;
            } else {
                player_is_busy = true;
            }
        }

        UnitMovement::Idle => {}
    }

    PlayerAnimationResult { player_is_busy, transition_to_push, transition_to_post }
}

/// Applies the state changes required to begin a box-pushing animation sequence.
///
/// This function:
/// 1. Updates `mob_grid` to reflect the box's new position.
/// 2. Starts the box's `Moving` animation toward its destination.
/// 3. Starts the player's `Pushing` animation (with recoil target for post-push).
///
/// # Arguments
///
/// * `state` - mutable game state to update player, mobs, and grid
/// * `box_idx` - index of the box in `state.mobs` to be pushed
/// * `(p_tx, p_ty)` - player's new tile coordinates after moving adjacent to the box
/// * `(b_tx, b_ty)` - box's new tile coordinates after being pushed
/// * `(dx, dy)` - direction of the push as tile deltas
/// * `map_width` - map width in tiles
/// * `delta` - frame time
fn apply_push_transition(
    state: &mut State,
    box_idx: usize,
    (p_tx, p_ty): (i32, i32),
    (b_tx, b_ty): (i32, i32),
    (dx, dy): (i32, i32),
    map_width: usize,
    delta: f32,
) {
    let old_box_idx = (p_ty as usize) * map_width + (p_tx as usize);
    let new_box_idx = (b_ty as usize) * map_width + (b_tx as usize);

    if state.mob_grid[old_box_idx] == Some(box_idx) {
        state.mob_grid[old_box_idx] = None;
        state.mob_grid[new_box_idx] = Some(box_idx);

        let box_unit = &mut state.mobs[box_idx];
        let (bx, by) = (box_unit.pixel_x.round(), box_unit.pixel_y.round());
        let (offset_x, offset_y) = get_offset(dx, dy, 1.0);
        let (target_bx, target_by) = (bx + offset_x, by + offset_y);

        box_unit.movement = UnitMovement::Moving {
            start_x: bx,
            start_y: by,
            target_x: target_bx,
            target_y: target_by,
            elapsed_time: 0.0 - delta,
            duration: BOX_MOVE_DURATION,
        };
        box_unit.tile_x = b_tx;
        box_unit.tile_y = b_ty;

        let (over_x, over_y) = get_offset(dx, dy, PUSH_OFFSET);
        let push_target_x = bx + over_x;
        let push_target_y = by + over_y;

        let settle_x = bx;
        let settle_y = by;

        let player = &mut state.player.unit;
        player.movement = UnitMovement::Pushing {
            start_x: player.pixel_x,
            start_y: player.pixel_y,
            target_x: push_target_x,
            target_y: push_target_y,
            elapsed_time: 0.0,
            duration: PUSH_DURATION,
            recoil_target_x: settle_x,
            recoil_target_y: settle_y,
        };
        player.tile_x = p_tx;
        player.tile_y = p_ty;
    }
}

/// Initiates the player's recoil animation after completing a push.
///
/// This sets the player's movement state to `PostPushing`, animating from
/// the pushing endpoint back to the settled position over the specified duration.
///
/// # Arguments
///
/// * `player` - mutable reference to the player's unit
/// * `(start_x, start_y)` - starting pixel position (end of push animation)
/// * `(target_x, target_y)` - target pixel position (settled recoil position)
fn apply_post_push_transition(
    player: &mut Unit,
    (start_x, start_y): (f32, f32),
    (target_x, target_y): (f32, f32),
) {
    player.movement = UnitMovement::PostPushing {
        start_x,
        start_y,
        target_x,
        target_y,
        elapsed_time: 0.0,
        duration: POST_PUSH_DURATION,
    };
}

/// Updates animations for all non-player mobile units (e.g., boxes).
///
/// For each mob in `Moving` state, this function:
/// - Advances the animation by `delta` seconds.
/// - Interpolates the pixel position using linear interpolation (`lerp`).
/// - Sets the mob to `Idle` when the animation completes.
///
/// # Arguments
///
/// * `mobs` - slice of mutable mob units to update
/// * `delta` - time elapsed since the last frame
fn update_mob_animations(mobs: &mut [Unit], delta: f32) {
    for unit in mobs.iter_mut() {
        if let UnitMovement::Moving {
            start_x,
            start_y,
            target_x,
            target_y,
            elapsed_time,
            duration,
        } = &mut unit.movement
        {
            *elapsed_time += delta;
            let progress = (*elapsed_time / *duration).min(1.0);

            unit.pixel_x = lerp(*start_x, *target_x, progress);
            unit.pixel_y = lerp(*start_y, *target_y, progress);

            if progress >= 1.0 {
                unit.pixel_x = *target_x;
                unit.pixel_y = *target_y;
                unit.movement = UnitMovement::Idle;
            }
        }
    }
}

/// Initiates the "approach" animation before pushing a box.
///
/// The player moves halfway toward the box (0.5 tile magnitude) to prepare for the push.
/// Sets the player's movement state to `PrePushing` with parameters needed to transition
/// to full pushing once the approach completes.
///
/// # Arguments
///
/// * `player` - mutable reference to the player's unit
/// * `(dx, dy)` - push direction as tile deltas
/// * `box_idx` - index of the box to be pushed
/// * `(player_next_tx, player_next_ty)` - player's tile position after moving adjacent to the box
/// * `(box_next_tx, box_next_ty)` - box's destination tile after being pushed
fn start_pre_push_animation(
    player: &mut Unit,
    (dx, dy): (i32, i32),
    box_idx: usize,
    (player_next_tx, player_next_ty): (i32, i32),
    (box_next_tx, box_next_ty): (i32, i32),
) {
    let (px, py) = (player.pixel_x, player.pixel_y);
    let (offset_x, offset_y) = get_offset(dx, dy, PUSH_OFFSET);
    let mid_x = px + offset_x;
    let mid_y = py + offset_y;

    player.movement = UnitMovement::PrePushing {
        start_x: px,
        start_y: py,
        target_x: mid_x,
        target_y: mid_y,
        elapsed_time: 0.0,
        duration: PRE_PUSH_DURATION,
        box_idx,
        player_next_tx,
        player_next_ty,
        box_next_tx,
        box_next_ty,
        push_dx: dx,
        push_dy: dy,
    };
}

/// Initiates a standard walking animation for the player.
///
/// Moves the player by one full tile in the specified direction.
/// Updates both the animation state (pixel position over time) and the logical tile position.
///
/// # Arguments
///
/// * `player` - mutable reference to the player's unit
/// * `(dx, dy)` - movement direction as tile deltas
/// * `(next_tx, next_ty)` - destination tile coordinates
fn start_walking_animation(
    player: &mut Unit,
    (dx, dy): (i32, i32),
    (next_tx, next_ty): (i32, i32),
) {
    let (px, py) = (player.pixel_x, player.pixel_y);
    let (offset_x, offset_y) = get_offset(dx, dy, 1.0);

    player.movement = UnitMovement::Moving {
        start_x: px,
        start_y: py,
        target_x: px + offset_x,
        target_y: py + offset_y,
        elapsed_time: 0.0,
        duration: MOVE_DURATION,
    };
    player.tile_x = next_tx;
    player.tile_y = next_ty;
}

/// Advances the game simulation by one time step.
///
/// This is the core game loop function that:
/// 1. Updates all animations (player and mobs).
/// 2. Processes player input (if not busy with animations).
/// 3. Handles movement and box-pushing logic with collision detection.
///
/// # Arguments
///
/// * `curr_state` - mutable game state (player, mobs, grid)
/// * `input_state` - current player input (directional buttons)
/// * `delta` - time elapsed since last frame (for animation timing)
/// * `game` - static game map (walkability, collidable objects)
///
/// # Returns
///
/// * `Some(0)` if a special key combination is pressed.
/// * `None` otherwise (successful step with or without movement).
///
/// # Animation States Handled
///
/// - `Idle`: ready for input.
/// - `Moving`: walking between tiles.
/// - `PrePushing`: approaching a box to push.
/// - `Pushing`: actively pushing a box (supports chain-pushes).
/// - `PostPushing`: recoiling after a push.
pub fn make_step(
    curr_state: &mut State,
    input_state: &InputSnapshot,
    delta: f32,
    game: &ferari::assets::GameMap,
) -> Option<u32> {
    let map_width = game.size[0] as usize;

    // ============================================
    // ANIMATION UPDATE (player + all mobs)
    // ============================================

    // Player update
    let PlayerAnimationResult { mut player_is_busy, transition_to_push, transition_to_post } =
        update_player_animation(
            &mut curr_state.player.unit,
            delta,
            input_state,
            &curr_state.mob_grid,
            game,
            map_width,
        );

    if let Some((box_idx, p_tx, p_ty, b_tx, b_ty, dx, dy)) = transition_to_push {
        player_is_busy = true;

        apply_push_transition(
            curr_state,
            box_idx,
            (p_tx, p_ty),
            (b_tx, b_ty),
            (dx, dy),
            map_width,
            delta,
        );
    }

    if let Some((sx, sy, tx, ty)) = transition_to_post {
        player_is_busy = true;

        apply_post_push_transition(&mut curr_state.player.unit, (sx, sy), (tx, ty));
    }

    // Mob (box) update
    update_mob_animations(&mut curr_state.mobs, delta);

    // ============================================
    // INPUT PROCESSING
    // ============================================

    if input_state.left && input_state.right {
        return Some(0);
    }

    // If the player is busy (the animation has not finished), the input is not processed
    if player_is_busy {
        return None;
    }

    let (mut dx, mut dy) = (0, 0);
    let mut new_dir = curr_state.player.unit.direction;

    if input_state.right {
        dx = 1;
        dy = 0;
        new_dir = Direction::SE;
    } else if input_state.left {
        dx = -1;
        dy = 0;
        new_dir = Direction::NW;
    } else if input_state.up {
        dx = 0;
        dy = -1;
        new_dir = Direction::NE;
    } else if input_state.down {
        dx = 0;
        dy = 1;
        new_dir = Direction::SW;
    }

    curr_state.player.unit.direction = new_dir;

    if dx == 0 && dy == 0 {
        return None;
    }

    // Current player coordinates
    let p_tx = curr_state.player.unit.tile_x;
    let p_ty = curr_state.player.unit.tile_y;

    // Target player coordinates
    let next_tx = p_tx + dx;
    let next_ty = p_ty + dy;

    // ============================================
    // COLLISION AND MOVEMENT LOGIC
    // ============================================

    // Checking map boundaries and static walls
    if !game.is_walkable(next_tx, next_ty) || game.has_collidable_object_at(next_tx, next_ty) {
        return None;
    }

    // Checking dynamic objects
    let next_idx = (next_ty as usize) * map_width + (next_tx as usize);
    let mob_in_front = curr_state.mob_grid.get(next_idx).copied().flatten();

    if let Some(box_idx) = mob_in_front {
        // Trying to push the box

        let behind_tx = next_tx + dx;
        let behind_ty = next_ty + dy;

        // Checking the statics behind the box
        if !game.is_walkable(behind_tx, behind_ty)
            || game.has_collidable_object_at(behind_tx, behind_ty)
        {
            return None;
        }

        // Checking the dynamics behind the box
        let behind_idx = (behind_ty as usize) * map_width + (behind_tx as usize);
        if curr_state.mob_grid.get(behind_idx).copied().flatten().is_some() {
            return None;
        }

        // perform pre push
        start_pre_push_animation(
            &mut curr_state.player.unit,
            (dx, dy),
            box_idx,
            (next_tx, next_ty),
            (behind_tx, behind_ty),
        );
    } else {
        start_walking_animation(&mut curr_state.player.unit, (dx, dy), (next_tx, next_ty));
    }

    None
}

#[cfg(test)]
mod offset_dir_tests {
    use super::*;

    #[test]
    fn test_get_offset_southeast() {
        let (offset_x, offset_y) = get_offset(1, 0, 1.0);

        let tile_w = TILE_SIZE as f32;
        let tile_h = (TILE_SIZE as f32) * 0.5;
        let expected_x = tile_w * 0.5;
        let expected_y = tile_h * 0.5;

        assert_eq!(offset_x, expected_x);
        assert_eq!(offset_y, expected_y);
    }

    #[test]
    fn test_get_offset_northwest() {
        let (offset_x, offset_y) = get_offset(-1, 0, 1.0);

        let tile_w = TILE_SIZE as f32;
        let tile_h = (TILE_SIZE as f32) * 0.5;
        let expected_x = -tile_w * 0.5;
        let expected_y = -tile_h * 0.5;

        assert_eq!(offset_x, expected_x);
        assert_eq!(offset_y, expected_y);
    }

    #[test]
    fn test_get_dir_delta_ne() {
        assert_eq!(get_dir_delta(Direction::NE), (0, -1));
    }

    #[test]
    fn test_get_dir_delta_sw() {
        assert_eq!(get_dir_delta(Direction::SW), (0, 1));
    }

    #[test]
    fn test_get_dir_delta_all_directions() {
        let test_cases = vec![
            (Direction::SE, (1, 0)),
            (Direction::NW, (-1, 0)),
            (Direction::NE, (0, -1)),
            (Direction::SW, (0, 1)),
        ];

        for (direction, expected) in test_cases {
            assert_eq!(get_dir_delta(direction), expected);
        }
    }
}

#[cfg(test)]
mod update_mod_tests {
    use super::*;
    use ferari::world::Direction;

    fn create_test_mob_with_movement(movement: UnitMovement) -> Unit {
        Unit {
            pixel_x: 0.0,
            pixel_y: 0.0,
            tile_x: 0,
            tile_y: 0,
            x_speed: 10.0,
            y_speed: 10.0,
            movement,
            direction: Direction::SE,
        }
    }

    #[test]
    fn test_update_mob_animations_single_mob_completion() {
        let mut mobs = vec![create_test_mob_with_movement(UnitMovement::Moving {
            start_x: 0.0,
            start_y: 0.0,
            target_x: 100.0,
            target_y: 100.0,
            elapsed_time: 0.5,
            duration: 1.0,
        })];

        let delta = 0.6;

        update_mob_animations(&mut mobs, delta);

        assert!(matches!(mobs[0].movement, UnitMovement::Idle));
        assert_eq!(mobs[0].pixel_x, 100.0);
        assert_eq!(mobs[0].pixel_y, 100.0)
    }
}

#[cfg(test)]
mod pre_push_tests {
    use super::*;

    #[test]
    fn test_start_pre_push_animation_basic() {
        let mut player = Unit {
            pixel_x: 100.0,
            pixel_y: 100.0,
            tile_x: 5,
            tile_y: 5,
            x_speed: 10.0,
            y_speed: 10.0,
            movement: UnitMovement::Idle,
            direction: Direction::SE,
        };
        let original_x = player.pixel_x;
        let original_y = player.pixel_y;

        let direction = (1, 0);
        let box_idx = 42;
        let player_next_tile = (6, 5);
        let box_next_tile = (7, 5);

        start_pre_push_animation(&mut player, direction, box_idx, player_next_tile, box_next_tile);

        match &player.movement {
            UnitMovement::PrePushing {
                start_x,
                start_y,
                target_x,
                target_y,
                elapsed_time,
                duration,
                box_idx: actual_box_idx,
                player_next_tx,
                player_next_ty,
                box_next_tx,
                box_next_ty,
                push_dx,
                push_dy,
            } => {
                assert_eq!(*start_x, original_x);
                assert_eq!(*start_y, original_y);

                let (expected_offset_x, expected_offset_y) = get_offset(1, 0, PUSH_OFFSET);
                assert_eq!(*target_x, original_x + expected_offset_x);
                assert_eq!(*target_y, original_y + expected_offset_y);

                assert_eq!(*elapsed_time, 0.0);
                assert_eq!(*duration, PRE_PUSH_DURATION);

                assert_eq!(*actual_box_idx, box_idx);
                assert_eq!(*player_next_tx, player_next_tile.0);
                assert_eq!(*player_next_ty, player_next_tile.1);
                assert_eq!(*box_next_tx, box_next_tile.0);
                assert_eq!(*box_next_ty, box_next_tile.1);
                assert_eq!(*push_dx, direction.0);
                assert_eq!(*push_dy, direction.1);
            }
            other => panic!("Expected PrePushing movement, got {:?}", other),
        }

        assert_eq!(player.pixel_x, original_x);
        assert_eq!(player.pixel_y, original_y);
    }
}

#[cfg(test)]
mod post_push_tests {
    use super::*;

    #[test]
    fn test_apply_post_push_transition() {
        let mut player = Unit {
            pixel_x: 0.0,
            pixel_y: 0.0,
            tile_x: 0,
            tile_y: 0,
            x_speed: 10.0,
            y_speed: 10.0,
            movement: UnitMovement::Idle,
            direction: Direction::SE,
        };

        let start = (10.0, 20.0);
        let target = (30.0, 40.0);

        apply_post_push_transition(&mut player, start, target);

        match &player.movement {
            UnitMovement::PostPushing {
                start_x,
                start_y,
                target_x,
                target_y,
                elapsed_time,
                duration,
            } => {
                assert_eq!(*start_x, start.0);
                assert_eq!(*start_y, start.1);
                assert_eq!(*target_x, target.0);
                assert_eq!(*target_y, target.1);

                assert_eq!(*elapsed_time, 0.0);
                assert_eq!(*duration, POST_PUSH_DURATION);
            }
            other => {
                panic!("Expected PostPushing movement, got {:?}", other);
            }
        }
    }
}

#[cfg(test)]
mod push_tests {
    use super::*;
    use ferari::world::Player;

    #[test]
    fn test_apply_push_transition() {
        let map_width = 10;
        let player_tile = (5, 5);
        let box_tile = (6, 5);

        let mut state = State {
            player: Player {
                unit: Unit {
                    pixel_x: 100.0,
                    pixel_y: 100.0,
                    tile_x: player_tile.0,
                    tile_y: player_tile.1,
                    movement: UnitMovement::Idle,
                    direction: Direction::SE,
                    x_speed: 10.0,
                    y_speed: 10.0,
                },
            },
            mobs: vec![
                Unit {
                    pixel_x: 164.0,
                    pixel_y: 132.0,
                    tile_x: box_tile.0,
                    tile_y: box_tile.1,
                    movement: UnitMovement::Idle,
                    direction: Direction::SE,
                    x_speed: 10.0,
                    y_speed: 10.0,
                },
                Unit {
                    pixel_x: 200.0,
                    pixel_y: 200.0,
                    tile_x: 7,
                    tile_y: 5,
                    movement: UnitMovement::Idle,
                    direction: Direction::SE,
                    x_speed: 10.0,
                    y_speed: 10.0,
                },
            ],
            mob_grid: vec![None; map_width * map_width],
            ..State::default()
        };

        let box_idx = 0;
        let direction = (1, 0);
        let delta = 0.016;

        let old_box_idx = box_tile.1 as usize * map_width + box_tile.1 as usize;
        state.mob_grid[old_box_idx] = Some(0);

        let original_player_pixel_x = state.player.unit.pixel_x;
        let original_player_pixel_y = state.player.unit.pixel_y;
        let original_box_pixel_x = state.mobs[0].pixel_x;
        let original_box_pixel_y = state.mobs[0].pixel_y;

        apply_push_transition(
            &mut state,
            box_idx,
            player_tile,
            box_tile,
            direction,
            map_width,
            delta,
        );

        let new_box_idx = box_tile.1 as usize * map_width + box_tile.1 as usize + 1;
        assert_eq!(state.mob_grid[old_box_idx], None);
        assert_eq!(state.mob_grid[new_box_idx], Some(0));

        match &state.mobs[0].movement {
            UnitMovement::Moving {
                start_x,
                start_y,
                target_x,
                target_y,
                elapsed_time,
                duration,
            } => {
                assert_eq!(*start_x, original_box_pixel_x.round());
                assert_eq!(*start_y, original_box_pixel_y.round());

                let (offset_x, offset_y) = get_offset(1, 0, 1.0);
                assert_eq!(*target_x, original_box_pixel_x.round() + offset_x);
                assert_eq!(*target_y, original_box_pixel_y.round() + offset_y);

                assert_eq!(*elapsed_time, 0.0 - delta);
                assert_eq!(*duration, BOX_MOVE_DURATION);
            }
            other => panic!("Expected Moving movement, got {:?}", other),
        }

        assert_eq!(state.mobs[0].tile_x, box_tile.0);
        assert_eq!(state.mobs[0].tile_y, box_tile.1);

        match &state.player.unit.movement {
            UnitMovement::Pushing {
                start_x,
                start_y,
                target_x,
                target_y,
                elapsed_time,
                duration,
                recoil_target_x,
                recoil_target_y,
            } => {
                assert_eq!(*start_x, original_player_pixel_x);
                assert_eq!(*start_y, original_player_pixel_y);

                let (push_offset_x, push_offset_y) = get_offset(1, 0, PUSH_OFFSET);
                assert_eq!(*target_x, original_box_pixel_x.round() + push_offset_x);
                assert_eq!(*target_y, original_box_pixel_y.round() + push_offset_y);

                assert_eq!(*recoil_target_x, original_box_pixel_x.round());
                assert_eq!(*recoil_target_y, original_box_pixel_y.round());

                assert_eq!(*elapsed_time, 0.0);
                assert_eq!(*duration, PUSH_DURATION);
            }
            other => panic!("Expected Pushing movement, got {:?}", other),
        }

        assert_eq!(state.player.unit.tile_x, player_tile.0);
        assert_eq!(state.player.unit.tile_y, player_tile.1);

        assert!(matches!(state.mobs[1].movement, UnitMovement::Idle));
        assert_eq!(state.mobs[1].pixel_x, 200.0);
        assert_eq!(state.mobs[1].pixel_y, 200.0);
    }
}

#[cfg(test)]
mod walking_tests {
    use super::*;

    #[test]
    fn test_start_walking_animation() {
        let next_tile = (6, 5);
        let direction = (1, 0);

        let mut player = Unit {
            pixel_x: 100.0,
            pixel_y: 100.0,
            tile_x: next_tile.0 - 1,
            tile_y: next_tile.1,
            x_speed: 10.0,
            y_speed: 10.0,
            movement: UnitMovement::Idle,
            direction: Direction::SE,
        };

        start_walking_animation(&mut player, direction, next_tile);

        match &player.movement {
            UnitMovement::Moving {
                start_x,
                start_y,
                target_x,
                target_y,
                elapsed_time,
                duration,
            } => {
                assert_eq!(*start_x, 100.0);
                assert_eq!(*start_y, 100.0);

                let (offset_x, offset_y) = get_offset(1, 0, 1.0);
                assert_eq!(*target_x, 100.0 + offset_x);
                assert_eq!(*target_y, 100.0 + offset_y);

                assert_eq!(*elapsed_time, 0.0);
                assert_eq!(*duration, MOVE_DURATION);
            }
            other => panic!("Expected Moving movement, got {:?}", other),
        }

        assert_eq!(player.tile_x, 6);
        assert_eq!(player.tile_y, 5);

        assert_eq!(player.pixel_x, 100.0);
        assert_eq!(player.pixel_y, 100.0);
    }
}

#[cfg(test)]
mod update_player_tests {
    use super::*;
    use ferari::assets::GameMap;

    fn create_test_map() -> GameMap {
        GameMap::load("../game_levels/level1.json").unwrap()
    }

    fn create_unit_with_movement(movement: UnitMovement) -> Unit {
        Unit {
            pixel_x: 496.0,
            pixel_y: 664.0,
            tile_x: 1,
            tile_y: 1,
            x_speed: 10.0,
            y_speed: 10.0,
            movement,
            direction: Direction::SW,
        }
    }

    #[test]
    fn test_update_player_animation_pre_pushing() {
        let mut unit = create_unit_with_movement(UnitMovement::PrePushing {
            start_x: 496.0,
            start_y: 664.0,
            target_x: 432.0,
            target_y: 696.0,
            elapsed_time: 0.8,
            duration: 1.0,
            box_idx: 0,
            player_next_tx: 1,
            player_next_ty: 2,
            box_next_tx: 1,
            box_next_ty: 3,
            push_dx: 0,
            push_dy: 1,
        });

        let delta = 0.3;
        let input_state =
            InputSnapshot { up: false, left: false, down: true, right: false, escape: false };

        let game_map = create_test_map();
        let map_width = game_map.size[0] as usize;
        let mob_grid = vec![None; map_width * game_map.size[1] as usize];

        let result = update_player_animation(
            &mut unit,
            delta,
            &input_state,
            &mob_grid,
            &game_map,
            map_width,
        );

        assert!(result.player_is_busy);

        match result.transition_to_push {
            Some((box_idx, p_tx, p_ty, b_tx, b_ty, dx, dy)) => {
                assert_eq!(box_idx, 0);
                assert_eq!(p_tx, 1);
                assert_eq!(p_ty, 2);
                assert_eq!(b_tx, 1);
                assert_eq!(b_ty, 3);
                assert_eq!(dx, 0);
                assert_eq!(dy, 1);
            }
            None => panic!("Expected transition_to_push after PrePushing"),
        }

        assert_eq!(unit.pixel_x, 432.0);
        assert_eq!(unit.pixel_y, 696.0);

        assert!(result.transition_to_post.is_none());
    }

    #[test]
    fn test_update_player_animation_pushing() {
        let mut unit = create_unit_with_movement(UnitMovement::Pushing {
            start_x: 496.0,
            start_y: 664.0,
            target_x: 432.0,
            target_y: 696.0,
            elapsed_time: 0.9,
            duration: 1.0,
            recoil_target_x: 496.0,
            recoil_target_y: 664.0,
        });

        let delta = 0.2;
        let input_state =
            InputSnapshot { up: false, left: false, down: true, right: false, escape: false };

        let game_map = create_test_map();
        let map_width = game_map.size[0] as usize;
        let mut mob_grid = vec![None; map_width * game_map.size[1] as usize];

        let cur_box_idx = 2 * map_width + 1;
        mob_grid[cur_box_idx] = Some(0);

        let result = update_player_animation(
            &mut unit,
            delta,
            &input_state,
            &mob_grid,
            &game_map,
            map_width,
        );

        assert!(result.player_is_busy);

        match result.transition_to_push {
            Some((box_idx, p_tx, p_ty, b_tx, b_ty, dx, dy)) => {
                assert_eq!(box_idx, 0);
                assert_eq!(p_tx, 1);
                assert_eq!(p_ty, 2);
                assert_eq!(b_tx, 1);
                assert_eq!(b_ty, 3);
                assert_eq!(dx, 0);
                assert_eq!(dy, 1);
            }
            None => panic!("Expected transition_to_push if enabled pushing"),
        }

        assert_eq!(unit.pixel_x, 432.0);
        assert_eq!(unit.pixel_y, 696.0);

        assert!(result.transition_to_post.is_none());
    }
}

#[cfg(test)]
mod make_step_tests {
    use super::*;
    use ferari::assets::GameMap;

    fn create_test_map() -> GameMap {
        GameMap::load("../game_levels/level2.json").unwrap()
    }

    #[test]
    fn test_make_step_simple_walk() {
        let game_map = create_test_map();
        let mut state = State::new(&game_map);

        let input_state =
            InputSnapshot { up: false, left: false, down: false, right: true, escape: false };
        let delta = 0.016;
        let result = make_step(&mut state, &input_state, delta, &game_map);
        assert!(result.is_none());

        match &state.player.unit.movement {
            UnitMovement::Moving { .. } => {}
            other => panic!("Expected Moving movement, got {:?}", other),
        }

        assert_eq!(state.player.unit.direction, Direction::SE);
        assert_eq!(state.player.unit.tile_x, 3);
        assert_eq!(state.player.unit.tile_y, 3);
    }

    #[test]
    fn test_make_step_push_box() {
        let game_map = create_test_map();
        let mut state = State::new(&game_map);

        let map_width = game_map.size[0] as usize;
        let box_front_idx = 1 * map_width + 2;
        state.mob_grid[box_front_idx] = Some(0);

        let input_state =
            InputSnapshot { up: false, left: true, down: false, right: false, escape: false };
        let delta = 0.016;
        let result = make_step(&mut state, &input_state, delta, &game_map);
        assert!(result.is_none());

        match &state.player.unit.movement {
            UnitMovement::PrePushing { .. } => {}
            other => panic!("Expected PrePushing movement, got {:?}", other),
        }

        assert_eq!(state.player.unit.direction, Direction::NW);
        assert_eq!(state.player.unit.tile_x, 2);
        assert_eq!(state.player.unit.tile_y, 3);
    }
}
