alias d := dev

# Develop in the tauri dev window
dev:
    cargo tauri dev

# Develop inside the browser for faster hotreloading and nicer debug tools
browser-dev:
    cd backend && cargo r --bin non_ipc_backend --features non_ipc_backend &
    cd frontend && trunk serve --features non_ipc_backend --open true

# Builds and installs the app as rpm (only for systems with dnf installed)
install-rpm:
    cargo tauri build --bundles rpm
    sudo dnf reinstall ./target/release/bundle/rpm/Desktop\ Email\ Client-0.1.0-1.x86_64.rpm -y || sudo dnf install ./target/release/bundle/rpm/Desktop\ Email\ Client-0.1.0-1.x86_64.rpm -y
