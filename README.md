This is a repo of simple benchmarks across different rendering solutions. Check out the different branches to test.

To build:

1. `cd www && yarn && yarn start`
2. In a separate terminal, from the project root: `cargo watch -i "pkg/*" -s "wasm-pack build"`
3. Navigate to `localhost:8080` in a browser.
4. Game of life should start.

Different branches to test:

- `tiny-skia`: Using the `tiny-skia` drawing library to render a pixel buffer & using shared memory to access across JS and Rust.
- `homemade-pixel-renderer`: Manually managing a pixel buffer in Rust, sharing with JS.
- `piet`: Using the `piet` and `piet-web` libraries, which emits JS to render but is written in Rust.
- `js-renderer`: Canvas-only renderer, just using Rust to manage board state.

[Built using wasm-pack-template]("https://travis-ci.org/rustwasm/wasm-pack-template")
