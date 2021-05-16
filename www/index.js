import { Universe, Cell } from "wasm-game-of-life";
import { memory } from "wasm-game-of-life/wasm_game_of_life_bg";

const fpsCounter = document.getElementById('fps-count');
let currentTime = Date.now();

const universe = Universe.new(768, 768, 'game-of-life-canvas');

// JS-driven loop. Can we make it Rust-driven?
// Answer: Unlikely, unless we want to use RAF always since it seems that Rust
// depends on the platform for a timer
const renderLoop = () => {
  universe.tick();

  const now = Date.now();
  const timeSinceLastFrame = now - currentTime;
  currentTime = now;

  const FPS = 1000 / timeSinceLastFrame;
  fpsCounter.textContent = FPS;

  setImmediate(renderLoop);
};

renderLoop();