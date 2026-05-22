import init, { init_runtime } from './pkg/age_runtime.js';

async function startRuntime() {
  await init();
  const response = await fetch('./scenes/main.scene.json');
  const sceneJson = await response.text();
  await init_runtime('game-canvas', sceneJson);
}

startRuntime().catch((err) => {
  console.error('Failed to start runtime:', err);
});
