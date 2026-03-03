{
  description = "Desktop Email Client";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "x86_64-unknown-linux-gnu" "wasm32-unknown-unknown" ];
        };

        libraries = with pkgs; [
          openssl.out
        ];

        packages = with pkgs; [
          rustup
          just

          openssl
          dbus

          # for ./frontend
          trunk

          # for ./backend
          sqlx-cli

          # for electron app in ./electron-shell
          pkg-config
          electron
          nodejs
          nodePackages.npm
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = packages ++ [ rustToolchain ];

          shellHook = ''
            # prevent npm from downloading electron and using the nixpkg version
            export ELECTRON_OVERRIDE_DIST_PATH="${pkgs.electron}/bin"
            export ELECTRON_SKIP_BINARY_DOWNLOAD=1
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
          '';
        };
      }
    );
}
