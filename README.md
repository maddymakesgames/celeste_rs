# celeste_rs
[![crates.io](https://img.shields.io/crates/v/celeste_rs.svg)](https://crates.io/crates/celeste_rs)
[![docs](https://docs.rs/celeste_rs/badge.svg)](https://docs.rs/celeste_rs)
[![Rust CI](https://github.com/maddymakesgames/celeste_rs/actions/workflows/rust.yaml/badge.svg)](https://github.com/maddymakesgames/celeste_rs/actions/workflows/rust.yaml)

Collection of Rust crates to work with data formats found in the game Celeste and it's modding scene.


# celeste_rs gui editor
The main user-facing output of this repo is the gui editor.<br>
You can find a web version [here](https://maddymakesgames.github.io/celeste_rs/).<br>
Native versions will be found [here](https://github.com/maddymakesgames/celeste_rs/releases).

If you would like to compile a native version yourself you will need [the rust compiler](https://www.rust-lang.org/learn/get-started).<br>
The gui library we use, [egui](https://github.com/emilk/egui), also requires that extra libraries be installed on linux. On debian based distros you can install them with this command:<br>
`
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
`

We also include a [nix](https://nixos.org) flake making builds easy if you use the nix package manager (currently only tested on linux).

## Feedback
Please provide feedback or bug reports via github issues. If you would like to contribute you can make a pull request and I can look over it.

If you would like to contact me directly I am on discord as `@maddymakesgames`.
