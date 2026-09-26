import { mount } from 'svelte';
import App from './App.svelte';
import './app.css';

const target = document.getElementById('app');
if (!target) {
  throw new Error('the application mount point is missing from the page');
}

const app = mount(App, { target });

export default app;
