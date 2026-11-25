import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';

const root = path.dirname(fileURLToPath(import.meta.url));
const application = path.join(root, 'src-tauri', 'target', 'native-e2e', 'debug', process.platform === 'win32' ? 'geldapp.exe' : 'geldapp');
const outputDir = process.env.GELDAPP_E2E_LOG_DIR ?? path.join(os.tmpdir(), 'geldapp-wdio-e2e-logs');

export const config = {
  runner: 'local',
  specs: ['./tests/native/*.e2e.js'],
  maxInstances: 1,
  outputDir,
  capabilities: [{ browserName: 'tauri', 'tauri:options': { application } }],
  services: [[
    '@wdio/tauri-service',
    {
      appBinaryPath: application,
      driverProvider: 'embedded',
      embeddedPort: 4445,
      captureBackendLogs: true,
      startTimeout: 15_000,
    },
  ]],
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: { timeout: 90_000, ui: 'bdd' },
  waitforTimeout: 15_000,
  connectionRetryTimeout: 90_000,
  connectionRetryCount: 1,
  logLevel: 'info',
};
