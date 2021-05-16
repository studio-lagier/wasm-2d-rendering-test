import { Universe, Cell } from "wasm-game-of-life";
import { memory } from "wasm-game-of-life/wasm_game_of_life_bg";

const CELL_SIZE = 5; // px
const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";

const canvas = document.getElementById('game-of-life-canvas');
const fpsCounter = document.getElementById('fps-count');
let currentTime = Date.now();

canvas.height = 768;
canvas.width = 768;

const universe = Universe.new(canvas.width, canvas.height);

const ctx = canvas.getContext('2d');

// Number of pixels in grid
const imageDataSize = canvas.width * canvas.height * 4;
const imageData = new ImageData(canvas.width, canvas.height);

// JS-driven loop. Can we make it Rust-driven?
// Answer: Unlikely, unless we want to use RAF always since it seems that Rust
// depends on the platform for a timer
const renderLoop = () => {
  universe.tick();
  const pixels = new Uint8Array(memory.buffer, universe.image_data(), imageDataSize);
  imageData.data.set(pixels);
  ctx.putImageData(imageData, 0, 0);

  const now = Date.now();
  const timeSinceLastFrame = now - currentTime;
  currentTime = now;

  const FPS = 1000 / timeSinceLastFrame;
  fpsCounter.textContent = FPS;

  setImmediate(renderLoop);
};

renderLoop();