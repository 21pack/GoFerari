import json

grid = (5, 32)
s = 2
sprite_frame = (256 * s, 256 * s)
sprite_size = (64 * s, 64 * s)
sprite_offset = (int((256 * s- 64 * s) / 2), 72 * s)

frames_map = {
    "running_se": [[1, 1, 1, 1, 0], [1, 1, 1, 1, 0], [1, 1, 1, 1, 0], [1, 0, 0, 0, 0]],
    "running_ne": [[1, 1, 1, 1, 0], [1, 1, 1, 1, 0], [1, 1, 1, 1, 0], [1, 0, 0, 0, 0]],
    "running_nw": [[1, 1, 1, 1, 0], [1, 1, 1, 1, 0], [1, 1, 1, 1, 0], [1, 0, 0, 0, 0]],
    "running_sw": [[1, 1, 1, 1, 0], [1, 1, 1, 1, 0], [1, 1, 1, 1, 0], [1, 0, 0, 0, 0]],
    "pushing_se": [[1, 1, 1, 1, 1], [1, 1, 1, 1, 1], [1, 1, 1, 1, 1], [1, 1, 1, 1, 1]],
    "pushing_ne": [[1, 1, 1, 1, 1], [1, 1, 1, 1, 1], [1, 1, 1, 1, 1], [1, 1, 1, 1, 1]],
    "pushing_nw": [[1, 1, 1, 1, 1], [1, 1, 1, 1, 1], [1, 1, 1, 1, 1], [1, 1, 1, 1, 1]],
    "pushing_sw": [[1, 1, 1, 1, 1], [1, 1, 1, 1, 1], [1, 1, 1, 1, 1], [1, 1, 1, 1, 1]],
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

    print(f"Generated {output_path}")
