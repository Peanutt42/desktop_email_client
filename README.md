# desktop_email_client

Simple, modern looking desktop email client

### Tools to install

```
# rust and cargo should already be installed

cargo install sqlx-cli
cargo install trunk
cargo install tauri-cli --version "^2.0.0" --locked
```

Or, if you use NixOS, you can just use the `flake.nix`:

```bash
nix develop
```


### How to install

Fedora (rpm):

```bash
cargo tauri build --bundles rpm
sudo dnf install ./target/release/bundle/rpm/Desktop\ Email\ Client-0.1.0-1.x86_64.rpm -y
# or just
just install-rpm
```

All other linux distros are supported as well, see tauri bundle docs <https://tauri.app/distribute/>


### How to develop

if you haven't already, create the `.env` file in `./backend` first (used by sqlx macros for comp-time validation)

```bash
cd backend && echo "DATABASE_URL=sqlite://$(pwd)/dev_db.sqlite" > .env
```

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
