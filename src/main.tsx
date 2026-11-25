import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import '@fontsource-variable/dm-sans';
import '@fontsource-variable/manrope';
import './styles.css';
import './brand.css';

function mountApp() {
  ReactDOM.createRoot(document.getElementById('root')!).render(<React.StrictMode><App /></React.StrictMode>);
}

if (import.meta.env.MODE === 'native-e2e') {
  void import('@wdio/tauri-plugin').then(mountApp);
} else {
  mountApp();
}
