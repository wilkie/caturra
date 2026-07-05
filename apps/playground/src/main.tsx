import { createRoot } from 'react-dom/client';
import CssBaseline from '@mui/material/CssBaseline';
import { ThemeProvider, createTheme } from '@mui/material/styles';
import { App } from './App.js';

// Light and dark schemes that follow the OS preference (CSS-variable
// theming), so the chrome matches the always-dark CodeMirror editor when
// the page is in dark mode and stays legible either way.
const theme = createTheme({
  colorSchemes: { light: true, dark: true },
  typography: { fontSize: 13 },
});

const container = document.querySelector('#root');
if (container === null) {
  throw new Error('playground markup is missing #root');
}

createRoot(container).render(
  <ThemeProvider theme={theme}>
    <CssBaseline />
    <App />
  </ThemeProvider>,
);
