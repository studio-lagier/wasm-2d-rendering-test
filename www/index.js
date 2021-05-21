import { Universe, Cell } from "wasm-game-of-life";
import { memory } from "wasm-game-of-life/wasm_game_of_life_bg";

const CELL_SIZE = 11; // px
const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";

const canvas = document.getElementById('game-of-life-canvas');
const fpsCounter = document.getElementById('fps-count');
let currentTime = performance.now();

const universe = Universe.new(canvas.width, canvas.height);

const width = universe.width();
const height = universe.height();
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;

const ctx = canvas.getContext('2d');
const lastHundredFrames = [];

// JS-driven loop. Can we make it Rust-driven?
// Answer: Unlikely, unless we want to use RAF always since it seems that Rust
// depends on the platform for a timer
const renderLoop = () => {
  universe.tick();

  ctx.clearRect(0, 0, canvas.width, canvas.height);

  drawGrid();
  drawCells();


  const now = performance.now();
  const timeSinceLastFrame = now - currentTime;
  currentTime = now;

  const FPS = 1000 / timeSinceLastFrame;
  if (lastHundredFrames.length > 100) {
    lastHundredFrames.unshift();
  }

  lastHundredFrames.push(FPS);

  const averageFPS = lastHundredFrames.reduce((a, b) => a + b) / lastHundredFrames.length;
  fpsCounter.textContent = Math.round(averageFPS);

  setImmediate(renderLoop);
};

const getIndex = (row, column) => {
  return row * width + column;
};

const drawGrid = () => {
  ctx.beginPath();
  ctx.strokeStyle = GRID_COLOR;

  // Vertical lines.
  for (let i = 0; i <= width; i++) {
    ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
    ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
  }

  // Horizontal lines.
  for (let j = 0; j <= height; j++) {
    ctx.moveTo(0,                           j * (CELL_SIZE + 1) + 1);
    ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
  }

  ctx.stroke();
};

const drawCells = () => {
  const cellsPtr = universe.cells();
  const cells = new Uint8Array(memory.buffer, cellsPtr, width * height);

  ctx.beginPath();

  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = getIndex(row, col);

      if(cells[idx] !== Cell.Alive) continue;

      ctx.fillStyle = ALIVE_COLOR;

      ctx.fillRect(
        col * (CELL_SIZE + 1) + 1,
        row * (CELL_SIZE + 1) + 1,
        CELL_SIZE,
        CELL_SIZE
      );
    }
  }

  ctx.stroke();
};

drawGrid();
drawCells();
renderLoop();