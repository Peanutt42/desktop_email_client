# desktop_email_client

Simple, modern looking desktop email client

### How to run

Develop (tauri):

```bash
cargo tauri dev
# or just
just dev
```

Develop (inside a browser):

```bash
just browser-dev
```

Bundle (example: rpm)

```bash
cargo tauri build --bundles rpm
sudo dnf install ./target/release/bundle/rpm/Desktop\ Email\ Client-0.1.0-1.x86_64.rpm -y
# or just
just install-rpm
```
