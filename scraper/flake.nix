{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            python311Full
            python311Packages.pip
            python311Packages.platformdirs

            file
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.stdenv.cc.cc.lib pkgs.file ];

          shellHook = /* bash */ ''
            SOURCE_DATE_EPOCH=$(date +%s)
            VENV=.venv
            if [ -d $VENV ]; then
              source ./$VENV/bin/activate
            fi
            export PYTHONPATH=`pwd`/$VENV/${pkgs.python311Full.sitePackages}/:$PYTHONPATH
          '';
        };
      });
}
