# GoFerari

![CI Status](https://github.com/21pack/GoFerari/actions/workflows/ci.yml/badge.svg)

Game on [Ferari](https://github.com/suvorovrain/Ferari) (Fast Engine for Rendering Axonometric Rust-based Isometry).

Currently, x86_64 Linux and ARM64 macOS platforms are supported.

![demo](/demo.gif)

## Description

An isometric game in puzzle genre such as [Sokoban](https://en.wikipedia.org/wiki/Sokoban). The aim of the game is to get the boxes onto storage locations.

### Mechanics

* Player: You control the worker. Move up, down, left, or right.
* Boxes: Push the boxes, one at a time. You cannot pull them or push two boxes at once.
* Walls: Solid grey blocks form the immovable barriers. They confine your and boxes movement.
* Target Docks: Indented floor tiles mark the delivery targets where the boxes must be placed.
* Box States:
  * Light Box: A box that is not yet on a target.
  * Dark Box: A box that has been successfully pushed onto a target dock.

## Usage

### Install

* Download release archive from GitHub releases page.
* Clone and build this repo.

### Play

```shell
./play.sh
```

### Control

The player moves on a grid-based system with fixed-direction controls. This means each key consistently moves the character in one cardinal direction, regardless of the on-screen perspective.

* `WASD` or `arrow control`: movement;
* `<-`(`A`) + `->`(`D`): go to menu;
* `esc`: close game.

## Dependencies

### Linux

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo apt install cargo
rustup default stable
rustup component add rustfmt
rustup component add clippy
cargo install cargo-tarpaulin
```

### macOS

```shell
brew install rustup
rustup-init
```

## Development (Ferari)

* See [CONTRIBUTING.md](./CONTRIBUTING.md)
* Compile & run game via `cargo run -p game --release`
* View docs via `cargo doc` (use  --document-private-items if you want)
* Format your code via `cargo fmt`
* Everything else - in CI

## Authors

* **Maxim Rodionov:** [GitHub](https://github.com/RodionovMaxim05), [Telegram](https://t.me/Maxoon22)
* **Dmitri Chirkov:** [GitHub](https://github.com/kinokotakenoko9), [Telegram](https://t.me/chdmitri)
* **Nikita Shchutskii:** [GitHub](https://github.com/ns-58), [Telegram](https://t.me/szcz00)
* **Vladimir Zaikin:** [GitHub](https://github.com/Friend-zva), [Telegram](https://t.me/vo_va_w)

## License

We use assets from the following packages:
- Character:
  - [8-directional melee character](https://hormelz.itch.io/8-directional-2d-businessman-character)
- Boxes:
  - [Isometric Crates Pack](https://screamingbrainstudios.itch.io/isometric-object-pack)
- Floor:
  - [Isometric miniature dungeon](https://kenney.nl/assets/isometric-miniature-dungeon)
  - [Isometric Floor Tiles](https://screamingbrainstudios.itch.io/isotilepack)
- Letters and numbers:
  - [Isometric Letters, Numbers & Symbols Tile set 1](https://zrodfects.itch.io/32x32-isometric-letters-numbers-and-symbols-tile-set-1-starter-pack?download)

Distributed under the [MIT License](https://choosealicense.com/licenses/mit/). See [`LICENSE`](LICENSE) for more information.
