import { createRoot } from 'react-dom/client';

import { Compat } from './Compat';
// Deliberately NOT the playground's style.css: its `body { display: flex }` is for
// the editor's split panes, and it laid this page's sections out side by side.
import './compat.css';

const container = document.getElementById('root');
if (!container) {
  throw new Error('no #root element');
}

createRoot(container).render(<Compat />);
