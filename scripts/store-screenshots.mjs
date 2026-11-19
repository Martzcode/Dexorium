#!/usr/bin/env node
/**
 * Captures Microsoft Store screenshots from a running Dexorium build.
 *
 * The app must be launched with the WebView2 remote debugging port enabled:
 *
 *   $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'
 *   .\src-tauri\target\release\dexorium.exe
 *
 * Then run:
 *
 *   node scripts/store-screenshots.mjs [outDir] [shotId...]
 *
 * Each shot is rendered at 1920x1080 through the Chrome DevTools Protocol, so
 * the result does not depend on the physical screen size.
 */

import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';

const CDP_ENDPOINT = 'http://127.0.0.1:9222';
const VIEWPORT = { width: 1920, height: 1080 };

const SHOTS = [
  {
    id: 'screenshot-1-pokedex-dark',
    path: '/pokedex',
    language: 'en',
    theme: 'dark',
    waitFor: 'main',
    settle: 1800,
  },
  {
    id: 'screenshot-2-species-dark',
    path: '/pokedex/25',
    language: 'en',
    theme: 'dark',
    waitFor: 'main',
    settle: 1800,
  },
  {
    id: 'screenshot-3-type-matrix-light',
    path: '/tools/types',
    language: 'en',
    theme: 'light',
    waitFor: 'main',
    settle: 1800,
  },
  {
    id: 'screenshot-4-moves-dark',
    path: '/moves',
    language: 'en',
    theme: 'dark',
    waitFor: 'main',
    settle: 1800,
  },
  {
    id: 'screenshot-5-abilities-dark',
    path: '/abilities',
    language: 'en',
    theme: 'dark',
    waitFor: 'main',
    settle: 1800,
  },
  {
    id: 'screenshot-6-home-dark',
    path: '/',
    language: 'en',
    theme: 'dark',
    waitFor: 'main',
    settle: 1200,
  },
  {
    id: 'screenshot-7-settings-dark',
    path: '/settings',
    language: 'en',
    theme: 'dark',
    waitFor: 'main',
    settle: 1200,
  },
];

const delay = (ms) => new Promise((done) => setTimeout(done, ms));

async function findPage() {
  const response = await fetch(`${CDP_ENDPOINT}/json/list`);
  const targets = await response.json();
  const page = targets.find((target) => target.type === 'page');
  if (!page) {
    throw new Error('No page target found. Is Dexorium running with the debugging port?');
  }
  return page;
}

function createClient(url) {
  const socket = new WebSocket(url);
  const pending = new Map();
  let nextId = 1;

  socket.addEventListener('message', (event) => {
    const message = JSON.parse(event.data);
    const entry = pending.get(message.id);
    if (!entry) return;
    pending.delete(message.id);
    if (message.error) entry.reject(new Error(JSON.stringify(message.error)));
    else entry.resolve(message.result);
  });

  const ready = new Promise((done, fail) => {
    socket.addEventListener('open', done, { once: true });
    socket.addEventListener('error', fail, { once: true });
  });

  return {
    ready,
    close: () => socket.close(),
    send(method, params = {}) {
      const id = nextId++;
      return new Promise((done, fail) => {
        pending.set(id, { resolve: done, reject: fail });
        socket.send(JSON.stringify({ id, method, params }));
      });
    },
  };
}

async function evaluate(client, expression) {
  const result = await client.send('Runtime.evaluate', {
    expression,
    awaitPromise: true,
    returnByValue: true,
  });
  return result.result?.value;
}

async function waitForReady(client, selector, settle) {
  const deadline = Date.now() + 15000;
  while (Date.now() < deadline) {
    const state = await evaluate(
      client,
      `(() => ({ ready: document.readyState, hasRoot: !!document.querySelector(${JSON.stringify(selector)}), url: location.pathname }))()`,
    );
    if (state?.ready === 'complete' && state.hasRoot) break;
    await delay(200);
  }
  await delay(settle);
}

async function capture(client, shot, index, total, outDir) {
  await client.send('Page.navigate', { url: `http://tauri.localhost${shot.path}` });
  await waitForReady(client, shot.waitFor, shot.settle);

  // The device metrics override does not survive a cross-document navigation,
  // so apply it once the page is ready and give the layout a frame to settle.
  await client.send('Emulation.setDeviceMetricsOverride', {
    ...VIEWPORT,
    deviceScaleFactor: 1,
    mobile: false,
  });
  await delay(400);

  // The theme and language are read at startup from localStorage, so keep the
  // stored values in sync with the shot definition for the next navigation.
  const { data } = await client.send('Page.captureScreenshot', {
    format: 'png',
    fromSurface: true,
    captureBeyondViewport: false,
    clip: { x: 0, y: 0, ...VIEWPORT, scale: 1 },
  });

  const file = join(outDir, `${shot.id}-${VIEWPORT.width}x${VIEWPORT.height}.png`);
  await writeFile(file, Buffer.from(data, 'base64'));
  console.log(`[${index + 1}/${total}] ${file}`);
}

async function main() {
  const outDir = resolve(process.argv[2] ?? 'docs/store-listing/images/screenshots');
  const only = process.argv.slice(3);
  const shots = only.length ? SHOTS.filter((shot) => only.includes(shot.id)) : SHOTS;

  await mkdir(outDir, { recursive: true });

  const page = await findPage();
  const client = createClient(page.webSocketDebuggerUrl);
  await client.ready;
  await client.send('Page.enable');
  await client.send('Runtime.enable');
  await client.send('Emulation.setScrollbarsHidden', { hidden: true });
  await client.send('Emulation.setDeviceMetricsOverride', {
    ...VIEWPORT,
    deviceScaleFactor: 1,
    mobile: false,
  });

  // Lock the language and theme used by every shot, then let the app boot once
  // so the next navigations pick the stored preferences up. The sprite folder is
  // cleared on purpose: Store screenshots must only show the original
  // placeholders, never third-party artwork from an optional media pack.
  await evaluate(
    client,
    `(() => {
      localStorage.setItem('dexorium.language', ${JSON.stringify(shots[0].language)});
      localStorage.setItem('dexorium.theme', ${JSON.stringify(shots[0].theme)});
      localStorage.removeItem('dexorium.sprites.folder');
      return true;
    })()`,
  );

  for (const [index, shot] of shots.entries()) {
    await evaluate(
      client,
      `(() => {
        localStorage.setItem('dexorium.language', ${JSON.stringify(shot.language)});
        localStorage.setItem('dexorium.theme', ${JSON.stringify(shot.theme)});
        return true;
      })()`,
    );
    await capture(client, shot, index, shots.length, outDir);
  }

  client.close();
  console.log('Done.');
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
