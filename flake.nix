{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { fenix, nixpkgs, utils, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          system = system;
          config.allowUnfree = true; # Gurobi...
        };
        toolchain = fenix.packages.${system}.latest;
      in
      {
        packages.default = (pkgs.makeRustPlatform {
          cargo = toolchain.toolchain;
          rustc = toolchain.toolchain;
        }).buildRustPackage {
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };

        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs; [
            clang
            llvm
            llvmPackages.libclang
            lld
            pkg-config

            gurobi

            (toolchain.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rustc"
              "rustfmt"
              "rust-analyzer"
            ])
          ];
          buildInputs = with pkgs; [
            udev
            alsa-lib
            vulkan-loader
            wayland
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
            libxkbcommon

            gurobi
          ];
          LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath buildInputs;

          shellHook = ''
            export GUROBI_HOME=${pkgs.gurobi}
            export GUROBI_VERSION=$(basename $(ls -d ${pkgs.gurobi}) | sed 's/.*-\([0-9]\+\)\.\([0-9]\+\).*/\1\2/')
            export GRB_LICENSE_FILE="$HOME/.config/gurobi/gurobi.lic"
          '';
        };
      });
}
