alias d := dev

# Develop in the tauri dev window
dev extra_tauri_flags="":
    RUST_LOG="info,desktop_email_client_backend=debug" cargo tauri dev {{ extra_tauri_flags }}

# Develop inside the browser for faster hotreloading and nicer debug tools
browser-dev extra_trunk_flags="":
    cd backend && RUST_LOG="info,desktop_email_client_backend=debug" cargo r --bin non_ipc_backend --features non_ipc_backend &
    cd frontend && trunk serve --features non_ipc_backend {{ extra_trunk_flags }}

# Builds and installs the app as rpm (only for systems with dnf installed)
install-rpm:
    cargo tauri build --bundles rpm
    sudo dnf reinstall ./target/release/bundle/rpm/Desktop\ Email\ Client-0.1.0-1.x86_64.rpm -y || sudo dnf install ./target/release/bundle/rpm/Desktop\ Email\ Client-0.1.0-1.x86_64.rpm -y
