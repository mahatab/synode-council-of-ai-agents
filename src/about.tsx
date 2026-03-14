import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import AboutWindow from './components/about/AboutWindow';
import './styles/globals.css';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <AboutWindow />
  </StrictMode>,
);
