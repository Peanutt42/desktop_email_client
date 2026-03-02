# desktop_email_client

Simple, modern looking desktop email client

### Tools to install

if you use NixOS, you can just use the `flake.nix`:

```bash
nix develop
```

Or, for all other distros/platforms:

```
# rust, cargo, just (justfile) and npm should already be installed on your system

cargo install sqlx-cli
cargo install trunk
```

And then install the npm dependencies of the electron app:

```bash
cd ./electron-shell
npm install
```

### How to install the app

```bash
just bundle
```

The output will be in `./electron-shell/dist`.
Then you can just install the file generated, for linux its an AppImage for now so you just run `./electron-shell/dist/Desktop Email Client-0.1.0.AppImage`

### How to develop

if you haven't already, create the `.env` file in `./backend` first (used by sqlx macros for comp-time validation)

```bash
cd backend && echo "DATABASE_URL=sqlite://$(pwd)/dev_db.sqlite" > .env
```

Run the electron app:

```bash
just run
```

Develop (inside a browser, with autorebuilding frontend):

```bash
just browser-dev
```
