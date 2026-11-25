import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig(({ mode }) => ({
  plugins: [react()],
  server: { port: 1420, strictPort: true },
  build: {
    outDir: mode === 'native-e2e' ? 'dist-native-e2e' : 'dist',
    target: ['chrome111', 'edge111', 'safari16.4'],
    cssTarget: ['chrome111', 'edge111', 'safari16.4'],
  },
  clearScreen: false,
}));
