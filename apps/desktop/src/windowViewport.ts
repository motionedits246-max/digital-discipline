import type {WindowLayout} from './api';

export const viewportSize = () => ({width:window.innerWidth, height:window.innerHeight});

export function fitsLayout(layout:WindowLayout, viewport=viewportSize()) {
  // Windows and WebView2 can round logical dimensions differently at fractional DPI.
  const tolerance = 2;
  if (layout.mode === 'pill' || layout.mode === 'quick') {
    return Math.abs(viewport.width - layout.width) <= tolerance
      && Math.abs(viewport.height - layout.height) <= tolerance;
  }
  return viewport.width + tolerance >= layout.min_width
    && viewport.height + tolerance >= layout.min_height;
}

export function waitForLayout(layout:WindowLayout):Promise<boolean> {
  if (fitsLayout(layout)) return Promise.resolve(true);
  return new Promise(resolve => {
    const finish = (ready:boolean) => {
      window.removeEventListener('resize', check);
      window.clearTimeout(deadline);
      resolve(ready);
    };
    const check = () => {if (fitsLayout(layout)) finish(true);};
    const deadline = window.setTimeout(() => finish(fitsLayout(layout)), 1500);
    window.addEventListener('resize', check);
    check();
  });
}
