import { spawn } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const tauriCli = path.join(root, 'node_modules', '@tauri-apps', 'cli', 'tauri.js');
const cargoTarget = path.join(root, 'src-tauri', 'target', 'native-e2e');
const child = spawn(process.execPath, [
  tauriCli,
  'build',
  '--debug',
  '--no-bundle',
  '--ci',
  '--features',
  'e2e',
  '--config',
  'src-tauri/tauri.e2e.conf.json',
], {
  cwd: root,
  env: { ...process.env, CARGO_TARGET_DIR: cargoTarget },
  stdio: 'inherit',
});

const [code] = await new Promise((resolve, reject) => {
  child.once('error', reject);
  child.once('exit', (exitCode, signal) => {
    if (signal) reject(new Error(`Tauri e2e build exited on ${signal}`));
    else resolve([exitCode]);
  });
});
process.exitCode = code ?? 1;
