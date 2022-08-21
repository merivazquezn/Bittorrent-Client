Bittorrent tracker in rust

## Prerequisites

This crate uses has some dependencies of the Bittorrent client created in the root directory, so you will need our client in order to run the Tracker.

## First, you have to initialize the frontend

1. This project uses node 18.0.0. If you don't have it installed, instructions for each OS are found in [https://nodejs.org/es/download/]. You can check your node version running then `node -v` command.
2. `cd` into the `frontend` directoy, where the `package.json` file is
3. In order to install frontend dependencies, `yarn` is required. You can install it globally via npm running running `npm install --global yarn`
4. Run `yarn`command to install or update dependencies. It might take a while if it's the first time you install the dependencies.
5. Use `yarn run build` to compile the react project, the backend will automatically use the last version (which is in the /frontend/build directory)

Optional: if you will work on frontend code, after running `yarn run build` at least once:

1. run `yarn run start`, this will start frontend server on port 3000.
2. change the script tag to
   `<script defer="defer" src="http://localhost:3000/static/js/bundle.js"></script>`

That way, changes in the frontend will be automatically reflected on the server without having to build envery time

## How to run the tracker

Use the `RUST_LOG=info cargo run` command on the root tracker folder, where the Cargo.toml file is.

## Recover metrics dump

If you want to recover past metrics stored at `dump/metrics_dump`, you can run the program with the `-- --recover_metrics` flag:

`cargo run -- --recover_metrics`
