import init, { run_app } from './pkg/demo.js';
async function main() {
   await init('/pkg/demo_bg.wasm');
   run_app();
}
main();
