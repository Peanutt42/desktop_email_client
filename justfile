alias b := build
alias r := run

# Builds the frontend and backend
build EXTRA_CARGO_ARGS="":
	@echo "Building frontend..."
	cd ./frontend/ && trunk build {{EXTRA_CARGO_ARGS}}
	@echo "Building backend..."
	cd ./backend/ && cargo build {{EXTRA_CARGO_ARGS}}

# Serves frontend with hotreloading and runs backend
browser-dev EXTRA_CARGO_ARGS="":
	@echo "Builds frontend..."
	cd ./frontend/ && trunk serve {{EXTRA_CARGO_ARGS}} &
	@echo "Running backend..."
	cd ./backend/ && cargo run {{EXTRA_CARGO_ARGS}}

# Builds frontend and backend and runs the electron app
run:
	just build --release
	@echo "Starting electron app"
	cd ./electron-shell/ && npm start

# Builds frontend and backend and then bundles the electron app with electron-builder
# Bundle output is in `dist`
bundle:
	just build --release
	@echo "Bundeling app with electron-builder"
	cd ./electron-shell/ && npm run build
	@echo "Finished bundeling app, output is inside `dist`"
