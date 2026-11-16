import {ServiceHintSurface} from './ServiceHint';
import {ConfirmProvider} from './ConfirmDialog';
import React from 'react';import{createRoot}from'react-dom/client';import App from './App';import './style.css';
import './workspace-controls.css';
import './quick-panel.css';
createRoot(document.getElementById('root')!).render(<React.StrictMode><ConfirmProvider>{new URLSearchParams(location.search).get('surface')==='service-hint'?<ServiceHintSurface/>:<App/>}</ConfirmProvider></React.StrictMode>);
