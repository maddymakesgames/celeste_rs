{
  inputs.nixpkgs.url = "github:NixOs/nixpkgs/nixos-24.11";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = {
    self, nixpkgs, flake-utils
  }: flake-utils.lib.eachDefaultSystem (system: let
      system = "x86_64-linux";
      pkgs = import nixpkgs {inherit system;};
      stdenv = pkgs.stdenv;
      lib = pkgs.lib;
      rustPlatform = pkgs.rustPlatform;
      fs = lib.fileset;
      deps = with pkgs; {
        lib = [];
        cli = [];
        gui = [libGL libxkbcommon wayland];
        macros = [];
        test_bin = [];
      };
      libPath = with pkgs; lib.makeLibraryPath [
        libGL
        libxkbcommon
        wayland
      ];
      buildDeps = with pkgs; [
        rustup
        clang
        llvmPackages.bintools
      ];
    in {
      packages = rec {
        gui = rustPlatform.buildRustPackage rec {
          pname = "celeste_rs_gui";
          version = "0.4.1+0.5.1";

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = deps.gui ++ deps.lib ++ deps.macros;

          src = fs.toSource {
            root = ./.;
            fileset = ./.;
          };

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          LD_LIBRARY_PATH = libPath;
        };

        default = gui;

        debug = rustPlatform.buildRustPackage rec {
          pname = "celeste_rs_gui";
          version = "0.4.1+0.5.1";

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = deps.gui ++ deps.lib ++ deps.macros;
          buildType = "debug";

          src = fs.toSource {
            root = ./.;
            fileset = ./.;
          };
          
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          LD_LIBRARY_PATH = libPath;
        };

        lib = rustPlatform.buildRustPackage rec {
          pname = "celeste_rs";
          version = "0.5.1";
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = deps.lib ++ deps.macros;

          src = fs.toSource {
            root = ./.;
            fileset = ./.;
          };
        };

        cli = rustPlatform.buildRustPackage rec {
          pname = "celeste_rs_cli";
          version = "0.1.0+0.5.1";
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = deps.cli ++ deps.lib ++ deps.macros;

          src = fs.toSource {
            root = ./.;
            fileset = ./.;
          };
        };

        test_bin = rustPlatform.buildRustPackage rec {
          pname = "test_bin";
          version = "0.1.0";

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = deps.test_bin ++ deps.lib ++ deps.macros;

          src = fs.toSource {
            root = ./.;
            fileset = ./.;
          };
        };
      };

      devShells.default = pkgs.mkShell {
        buildInputs = deps.lib ++ deps.macros ++ deps.test_bin ++ deps.cli ++ deps.gui;
        nativeBuildInputs = buildDeps  ++ [ pkgs.pkg-config ];

        RUSTC_VERSION = "stable";

        LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];

        shellHook = ''
            export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
            export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
          '';

        RUST_LOG = "debug";
        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        LD_LIBRARY_PATH = libPath;
      };
    }
  );
}