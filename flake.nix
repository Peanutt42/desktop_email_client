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
          webkitgtk_6_0
	      gtk3
          cairo
          gdk-pixbuf
          glib.out
          dbus.lib
          openssl.out

          # added this for https://github.com/tauri-apps/tauri/issues/4930
          libthai
        ];

        packages = with pkgs; [
          rustup
          cargo-tauri
          trunk
          sqlx-cli

          pkg-config
          dbus
          openssl
          glib
          gtk3
          libsoup_3
          webkitgtk_6_0
          webkitgtk_4_1

          pkgs.electron
          pkgs.nodejs
          pkgs.nodePackages.npm

          just
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = packages ++ [ rustToolchain ];

          shellHook = ''
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
            export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS
            export GIO_MODULE_DIR="${pkgs.glib-networking}/lib/gio/modules/"
          '';
        };
      }
    );
}
