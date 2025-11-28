from PIL import Image
import json


SIZE_TILE = 256


def create_scaled_atlas_alphabet(atlas_path, json_data, output_path, scale=4):
    atlas = Image.open(atlas_path)
    with open(json_data, "r") as f:
        data_tiles = json.load(f)

    tile_size = data_tiles["meta"]["tile_size"]
    tile_size_new = tile_size * scale

    width_new, height_new = SIZE_TILE * 4, SIZE_TILE * 9
    atlas_new = Image.new("RGBA", (width_new, height_new))

    tiles_data_new = {}
    tiles_data_new["frames"] = {}
    tiles_data_new["meta"] = {
        "image": "atlas_x2.png",
        "tile_size": SIZE_TILE,
        "version": 1,
    }

    for i, (tile_name, coords) in enumerate(data_tiles["frames"].items()):
        x, y, w, h = coords["x"], coords["y"], coords["w"], coords["h"]
        tile = atlas.crop((x, y, x + w, y + h))

        resized_tile = tile.resize(
            (tile_size_new, tile_size_new), Image.Resampling.NEAREST
        )

        col, row = i % 4, i // 4
        x_new = col * SIZE_TILE + 60
        y_new = row * SIZE_TILE + 20

        atlas_new.paste(resized_tile, (x_new, y_new))

        tiles_data_new["frames"][tile_name] = {
            "x": col * SIZE_TILE,
            "y": row * SIZE_TILE,
            "w": SIZE_TILE,
            "h": SIZE_TILE,
        }

    atlas_new.save(output_path)

    new_json_path = output_path.replace(".png", ".json")
    with open(new_json_path, "w") as f:
        json.dump(tiles_data_new, f, indent=2)


def create_atlas_ground(atlas_path, json_data, output_path):
    atlas = Image.open(atlas_path)
    with open(json_data, "r") as f:
        data_tiles = json.load(f)

    tile_size = data_tiles["meta"]["tile_size"]

    width_new, height_new = tile_size * 4, tile_size * 2
    atlas_new = Image.new("RGBA", (width_new, height_new))

    tiles_data_new = {}
    tiles_data_new["frames"] = {}
    tiles_data_new["meta"] = {
        "image": "atlas_x2.png",
        "tile_size": SIZE_TILE,
        "version": 1,
    }

    for i, (tile_name, coords) in enumerate(data_tiles["frames"].items()):
        x, y, w, h = coords["x"], coords["y"], coords["w"], coords["h"]
        tile = atlas.crop((x, y, x + w, y + h))

        resized_tile = tile

        col, row = i % 4, i // 4
        x_new = col * SIZE_TILE
        y_new = row * SIZE_TILE

        tile_size_new = SIZE_TILE

        if tile_name == "wall_tile":
            x_new += 22
            y_new -= 14
            tile_size_new = tile_size - 48
            resized_tile = tile.resize(
                (tile_size_new, tile_size_new), Image.Resampling.NEAREST
            )

        atlas_new.paste(resized_tile, (x_new, y_new))

        tiles_data_new["frames"][tile_name] = {
            "x": col * SIZE_TILE,
            "y": row * SIZE_TILE,
            "w": SIZE_TILE,
            "h": SIZE_TILE,
        }

    atlas_new.save(output_path)

    new_json_path = output_path.replace(".png", ".json")
    with open(new_json_path, "w") as f:
        json.dump(tiles_data_new, f, indent=2)


create_atlas_ground("assets/atlas.png", "assets/atlas.json", "assets/atlas_x2.png")
