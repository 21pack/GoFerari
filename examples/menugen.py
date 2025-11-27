import json

N_ = "NOP"

A = "A"
B = "B"
C = "C"
D = "D"
E = "E"
F = "F"
G = "G"
H = "H"
I = "I"
J = "J"
K = "K"
L = "L"
M = "M"
N = "N"
O = "O"
P = "P"
Q = "Q"
R = "R"
S = "S"
T = "T"
U = "U"
V = "V"
W = "W"
X = "X"
Y = "Y"
Z = "Z"

N0 = "0"
N1 = "1"
N2 = "2"
N3 = "3"
N4 = "4"
N5 = "5"
N6 = "6"
N7 = "7"
N8 = "8"
N9 = "9"

P_ = "#"
# BOX?

map_menu = [
    [N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
    [N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
    [N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
    [N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
    [N_, N_, C, O, N, G, R, A, T, U, L, A, T, I, O, N, S, N_, N_],
    [N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
    [N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
    [N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
    [N_, N_, C, H, O, O, S, E, N_, L, E, V, E, L, N_, N_, N_, N_],
    [N_, P_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
    [N_, N_, N_, N_, N1, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
    [N_, N_, N_, N_, N2, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
    [N_, N_, N_, N_, N3, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
    [N_, N_, N_, N_, N4, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
    [N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
    [N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
    [N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
    [N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
    [N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_, N_],
]

size = len(map_menu)


def generate_data(name, map, size):
    data = {
        "meta": {"name": name, "tile_size": 256, "size": [size, size]},
        "tiles": {},
        "objects": {},
        "mobs": {},
    }

    tile_count = 1
    letter_count = 1
    for y in range(size):
        for x in range(size):
            cell = map[y][x]

            data["tiles"][f"tile_{tile_count}"] = {"x": x, "y": y, "asset": "floor"}
            tile_count += 1

            if cell == N_:
                continue
            elif cell == P_:
                data["mobs"]["player"] = {
                    "x_start": x,
                    "y_start": y,
                    "asset": cell,
                    "is_player": True,
                    "behaviour": {"type": "controlled"},
                }
            else:
                data["mobs"][f"letter_{letter_count}"] = {
                    "x_start": x,
                    "y_start": y,
                    "asset": cell,
                    "is_player": False,
                }
                letter_count += 1

    return data


def save_data_to_file(name, map, size):
    data = generate_data(name, map, size)

    with open(f"{name}.json", "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)

    return data


if __name__ == "__main__":
    save_data_to_file("menu", map_menu, 16)
