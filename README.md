# desktop_email_client

Simple, modern looking desktop email client

### Tools to install

```
# rust, cargo and just (justfile) should already be installed

cargo install sqlx-cli
cargo install trunk
```

Or, if you use NixOS, you can just use the `flake.nix`:

```bash
nix develop
```


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
