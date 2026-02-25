# From https://tauri.app/v1/guides/getting-started/prerequisites#setting-up-linux
# Modified to include cargo and rustc packages

let
  pkgs = import <nixpkgs> { };

  libraries = with pkgs;[
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

    # personal additions
    just
  ];
in
pkgs.mkShell {
  buildInputs = packages;

  shellHook =
    let
      joinLibs = libs: builtins.concatStringsSep ":" (builtins.map (x: "${x}/lib") libs);
      libs = joinLibs libraries;
    in
    ''
      export LD_LIBRARY_PATH=${libs}:$LD_LIBRARY_PATH
      # install tauri if not already installed
      cargo help tauri 2> /dev/null 1> /dev/null || cargo install tauri-cli
    '';
}
