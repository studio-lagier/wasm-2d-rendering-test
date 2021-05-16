import { main } from "wasm-game-of-life";


window.sc_internal_wrapper().then(module => {
  window.sc_internal = module;
  main();
  console.log('triggered main');
});