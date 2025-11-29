import json

s = 1
sprite_frame = (126 * s, 132 * s)
sprite_size = (100 * s, 100 * s)
sprite_offset = (13, 16)
# sprite_offset = (0, 0)

frames_map = {
    "walkingforward_se": [
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 0, 0, 0],
    ],
    "walkingforward_ne": [
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 0, 0, 0],
    ],
    "walkingforward_nw": [
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 0, 0, 0],
    ],
    "walkingforward_sw": [
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 0, 0, 0],
    ],
    "walkingback_se": [
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 0, 0, 0, 0],
    ],
    "walkingback_ne": [
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 0, 0, 0, 0],
    ],
    "walkingback_nw": [
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 0, 0, 0, 0],
    ],
    "walkingback_sw": [
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 0, 0],
        [1, 1, 1, 0, 0, 0, 0],
    ],
    "running_se": [
        [1, 1, 1, 1, 0, 0, 0],
        [1, 1, 1, 1, 0, 0, 0],
        [1, 1, 1, 1, 0, 0, 0],
        [1, 1, 0, 0, 0, 0, 0],
    ],
    "running_ne": [
        [1, 1, 1, 1, 0, 0, 0],
        [1, 1, 1, 1, 0, 0, 0],
        [1, 1, 1, 1, 0, 0, 0],
        [1, 1, 0, 0, 0, 0, 0],
    ],
    "running_nw": [
        [1, 1, 1, 1, 0, 0, 0],
        [1, 1, 1, 1, 0, 0, 0],
        [1, 1, 1, 1, 0, 0, 0],
        [1, 1, 0, 0, 0, 0, 0],
    ],
    "running_sw": [
        [1, 1, 1, 1, 0, 0, 0],
        [1, 1, 1, 1, 0, 0, 0],
        [1, 1, 1, 1, 0, 0, 0],
        [1, 1, 0, 0, 0, 0, 0],
    ],
    "pushing_se": [
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 0, 0, 0, 0, 0],
    ],
    "pushing_ne": [
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 0, 0, 0, 0, 0],
    ],
    "pushing_nw": [
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 0, 0, 0, 0, 0],
    ],
    "pushing_sw": [
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1],
        [1, 1, 0, 0, 0, 0, 0],
    ],
    "idle_se": [
        [1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [1, 0, 0, 0, 0, 0, 0],
    ],
    "idle_ne": [
        [1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [1, 0, 0, 0, 0, 0, 0],
    ],
    "idle_nw": [
        [1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [1, 0, 0, 0, 0, 0, 0],
    ],
    "idle_sw": [
        [1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 0],
        [1, 0, 0, 0, 0, 0, 0],
    ],
}


def generate_atlas_json():
    output_frames = {}
    global_row_cursor = 0

    for anim_key, matrix in frames_map.items():
        animation_idx = 0

        for local_row_idx, row_data in enumerate(matrix):
            for local_col_idx, has_sprite in enumerate(row_data):
                if has_sprite == 1:
                    current_global_row = global_row_cursor + local_row_idx

                    px_x = (local_col_idx * sprite_frame[0]) + sprite_offset[0]
                    px_y = (current_global_row * sprite_frame[1]) + sprite_offset[1]

                    frame_key = f"{anim_key}_{animation_idx}"

                    output_frames[frame_key] = {
                        "x": px_x,
                        "y": px_y,
                        "w": sprite_size[0],
                        "h": sprite_size[1],
                    }

                    animation_idx += 1

        global_row_cursor += len(matrix)

    final_json = {
        "frames": output_frames,
        "meta": {"image": "atlas.png", "tile_size": sprite_frame[0], "version": 1},
    }

    return final_json


if __name__ == "__main__":
    data = generate_atlas_json()

    output_path = "atlas.json"
    with open(output_path, "w") as f:
        json.dump(data, f, indent=4)

    print(f"Generated {output_path}. May delete current frame config")

s = """
,
        "box": {
            "x": 1010,
            "y": 0,
            "w": 64,
            "h": 64
        },
        "green_box": {
            "x": 1074,
            "y": 0,
            "w": 64,
            "h": 64
        }
"""
