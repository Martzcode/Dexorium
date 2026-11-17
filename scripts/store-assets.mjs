#!/usr/bin/env node
/**
 * Generates the Microsoft Store listing artwork for Dexorium.
 *
 * The assets are rendered as HTML at their exact pixel size and captured with
 * headless Microsoft Edge, so the output does not depend on a Python/ImageMagick
 * toolchain being installed.
 *
 *   node scripts/store-assets.mjs
 */

import { execFile } from 'node:child_process';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { promisify } from 'node:util';

const run = promisify(execFile);

const EDGE = 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe';
const ROOT = resolve('.');
const LOGOS_DIR = join(ROOT, 'docs/store-listing/images/logos');
const PROMO_DIR = join(ROOT, 'docs/store-listing/images/promo');
const SHOT_DIR = join(ROOT, 'docs/store-listing/images/screenshots');

const ACCENT = '#e8402a';
const ACCENT_SOFT = '#ff8a65';
const BG = '#0e1016';
const SURFACE = '#141823';
const TEXT = '#f4f5f8';
const TEXT_SOFT = '#b6bdcd';
const TEXT_MUTED = '#7d8599';
const FONT = "'Segoe UI Variable Display', 'Segoe UI', system-ui, sans-serif";
const MONO = "'Cascadia Code', 'Consolas', ui-monospace, monospace";

const icon = (await readFile(join(ROOT, 'src-tauri/icons/icon.png'))).toString('base64');
const ICON_URI = `data:image/png;base64,${icon}`;
const screenshot = async (name) =>
  `data:image/png;base64,${(await readFile(join(SHOT_DIR, name))).toString('base64')}`;

const baseCss = (width, height, background) => `
  * { box-sizing: border-box; margin: 0; padding: 0; }
  html, body { width: ${width}px; height: ${height}px; overflow: hidden; }
  body {
    font-family: ${FONT};
    color: ${TEXT};
    background: ${background};
    -webkit-font-smoothing: antialiased;
  }
`;

const gradient = `
  radial-gradient(1100px 620px at 82% -10%, rgba(232, 64, 42, 0.28), transparent 62%),
  radial-gradient(900px 520px at 8% 108%, rgba(255, 138, 101, 0.16), transparent 60%),
  linear-gradient(135deg, #0b0d12 0%, ${BG} 45%, #111623 100%)
`;

function logoMark(size) {
  return `<img class="mark" src="${ICON_URI}" width="${size}" height="${size}" alt="" />`;
}

function poster({ width, height, mark, title, tagline, footnote, stack = 'center' }) {
  return `<!doctype html><html><head><meta charset="utf-8" /><style>
    ${baseCss(width, height, gradient)}
    .wrap {
      height: 100%; display: flex; flex-direction: column;
      align-items: center; justify-content: center; gap: ${Math.round(height * 0.035)}px;
      text-align: center; padding: ${Math.round(width * 0.06)}px;
    }
    .mark {
      border-radius: ${Math.round(mark * 0.22)}px;
      box-shadow: 0 24px 70px rgba(0, 0, 0, 0.55);
    }
    h1 {
      font-size: ${Math.round(width * 0.115)}px; line-height: 1;
      letter-spacing: -0.04em; font-weight: 700;
    }
    p { font-size: ${Math.round(width * 0.038)}px; color: ${TEXT_SOFT}; }
    .foot {
      position: absolute; bottom: ${Math.round(height * 0.055)}px; left: 0; right: 0;
      padding: 0 ${Math.round(width * 0.05)}px; text-align: center;
      font-family: ${MONO}; font-size: ${Math.round(width * 0.026)}px;
      letter-spacing: 0.22em; text-transform: uppercase; color: ${ACCENT_SOFT};
    }
  </style></head><body>
    <div class="wrap">
      ${logoMark(mark)}
      <h1>${title}</h1>
      <p>${tagline}</p>
    </div>
    <div class="foot">${footnote}</div>
  </body></html>`;
}

function storeLogo() {
  return `<!doctype html><html><head><meta charset="utf-8" /><style>
    ${baseCss(300, 300, 'transparent')}
    body { display: grid; place-items: center; }
    img { border-radius: 54px; }
  </style></head><body>
    <img src="${ICON_URI}" width="248" height="248" alt="" />
  </body></html>`;
}

function compactTile(width, height, { mark, titleSize, taglineSize, layout }) {
  return `<!doctype html><html><head><meta charset="utf-8" /><style>
    ${baseCss(width, height, gradient)}
    body { display: flex; align-items: center; justify-content: center; gap: ${Math.round(width * 0.06)}px; padding: ${Math.round(width * 0.07)}px;
      ${layout === 'row' ? 'flex-direction: row;' : 'flex-direction: column; text-align: center;'} }
    img { border-radius: ${Math.round(mark * 0.22)}px; box-shadow: 0 18px 50px rgba(0,0,0,.5); }
    h1 { font-size: ${titleSize}px; line-height: 1; letter-spacing: -0.04em; font-weight: 700; }
    p { font-size: ${taglineSize}px; color: ${TEXT_SOFT}; margin-top: ${Math.round(taglineSize * 0.4)}px; }
  </style></head><body>
    <img src="${ICON_URI}" width="${mark}" height="${mark}" alt="" />
    <div>
      <h1>Dexorium</h1>
      <p>Offline Pok&eacute;mon encyclopedia</p>
    </div>
  </body></html>`;
}

async function promoWide(width, height, shot) {
  const pad = Math.round(width * 0.055);
  const eyebrow = Math.round(width * 0.0118);
  const title = Math.round(width * 0.052);
  const lead = Math.round(width * 0.0165);
  const chip = Math.round(width * 0.0118);
  return `<!doctype html><html><head><meta charset="utf-8" /><style>
    ${baseCss(width, height, '#0b0d12')}
    body { position: relative; background: ${gradient}; }
    .bg {
      position: absolute; top: 0; right: 0; bottom: 0; left: 42%;
      background: url("${shot}") left center / cover no-repeat;
      -webkit-mask-image: linear-gradient(to right, transparent 0, #000 90px, #000 calc(100% - 70px), transparent 100%);
      mask-image: linear-gradient(to right, transparent 0, #000 90px, #000 calc(100% - 70px), transparent 100%);
    }
    .copy {
      position: relative; z-index: 1;
      width: ${Math.round(width * 0.46)}px; height: 100%;
      padding: ${Math.round(height * 0.1)}px 0 ${Math.round(height * 0.1)}px ${pad}px;
      display: flex; flex-direction: column; justify-content: center; gap: ${Math.round(height * 0.032)}px;
    }
    .brand { display: flex; align-items: center; gap: ${Math.round(width * 0.012)}px; }
    .brand img { border-radius: ${Math.round(width * 0.011)}px; }
    .brand span { font-size: ${Math.round(width * 0.018)}px; font-weight: 700; letter-spacing: -0.02em; }
    .eyebrow { font-family: ${MONO}; font-size: ${eyebrow}px; letter-spacing: 0.24em; text-transform: uppercase; color: ${ACCENT_SOFT}; }
    h1 { font-size: ${title}px; line-height: 1.02; letter-spacing: -0.045em; font-weight: 700; }
    h1 em { font-style: normal; color: ${ACCENT}; }
    p { font-size: ${lead}px; line-height: 1.5; color: ${TEXT_SOFT}; max-width: ${Math.round(width * 0.38)}px; }
    .chips { display: flex; flex-wrap: wrap; gap: ${Math.round(width * 0.008)}px; }
    .chip { border: 1px solid rgba(255,255,255,.14); background: rgba(14,16,22,.55); color: ${TEXT}; border-radius: 999px; padding: ${Math.round(chip * 0.55)}px ${Math.round(chip * 1.2)}px; font-size: ${chip}px; backdrop-filter: blur(6px); }
  </style></head><body>
    <div class="bg"></div>
    <div class="copy">
      <div class="brand">
        <img src="${ICON_URI}" width="${Math.round(width * 0.032)}" height="${Math.round(width * 0.032)}" alt="" />
        <span>Dexorium</span>
      </div>
      <div class="eyebrow">Offline encyclopedia</div>
      <h1>Every Pok&eacute;mon.<br /><em>Zero network.</em></h1>
      <p>Species, moves, abilities and the full type chart&nbsp;&mdash; all embedded in a fast native Windows app that never needs a connection.</p>
      <div class="chips">
        <span class="chip">1,025 species</span>
        <span class="chip">919 moves</span>
        <span class="chip">No account, no telemetry</span>
      </div>
    </div>
  </body></html>`;
}

const assets = [
  {
    file: join(LOGOS_DIR, 'StoreLogo-300x300.png'),
    width: 300,
    height: 300,
    transparent: true,
    html: storeLogo(),
  },
  {
    file: join(LOGOS_DIR, 'StoreLogo-720x1080.png'),
    width: 720,
    height: 1080,
    html: poster({
      width: 720,
      height: 1080,
      mark: 300,
      title: 'Dexorium',
      tagline: 'Offline Pokémon encyclopedia',
      footnote: 'Every Pokémon. Zero network.',
    }),
  },
  {
    file: join(LOGOS_DIR, 'StoreLogo-1080x1080.png'),
    width: 1080,
    height: 1080,
    html: poster({
      width: 1080,
      height: 1080,
      mark: 420,
      title: 'Dexorium',
      tagline: 'Offline Pokémon encyclopedia',
      footnote: 'Every Pokémon. Zero network.',
    }),
  },
  {
    file: join(PROMO_DIR, 'promo-1920x1080.png'),
    width: 1920,
    height: 1080,
    html: await promoWide(1920, 1080, await screenshot('screenshot-1-pokedex-dark-1920x1080.png')),
  },
  {
    file: join(PROMO_DIR, 'promo-2400x1200.png'),
    width: 2400,
    height: 1200,
    html: await promoWide(2400, 1200, await screenshot('screenshot-1-pokedex-dark-1920x1080.png')),
  },
  {
    file: join(PROMO_DIR, 'promo-358x358.png'),
    width: 358,
    height: 358,
    html: compactTile(358, 358, { mark: 150, titleSize: 46, taglineSize: 15, layout: 'column' }),
  },
  {
    file: join(PROMO_DIR, 'promo-1000x800.png'),
    width: 1000,
    height: 800,
    html: compactTile(1000, 800, { mark: 300, titleSize: 86, taglineSize: 26, layout: 'column' }),
  },
  {
    file: join(PROMO_DIR, 'promo-414x180.png'),
    width: 414,
    height: 180,
    html: compactTile(414, 180, { mark: 92, titleSize: 40, taglineSize: 14, layout: 'row' }),
  },
];

const staging = join(tmpdir(), 'dexorium-store-assets');
await mkdir(staging, { recursive: true });
await mkdir(LOGOS_DIR, { recursive: true });
await mkdir(PROMO_DIR, { recursive: true });

for (const [index, asset] of assets.entries()) {
  const htmlFile = join(staging, `asset-${index}.html`);
  await writeFile(htmlFile, asset.html, 'utf8');

  const args = [
    '--headless=new',
    '--disable-gpu',
    '--hide-scrollbars',
    '--force-device-scale-factor=1',
    `--window-size=${asset.width},${asset.height}`,
    `--screenshot=${asset.file}`,
    '--no-first-run',
    '--no-default-browser-check',
    `file:///${htmlFile.replaceAll('\\', '/')}`,
  ];
  if (asset.transparent) args.push('--default-background-color=00000000');

  await run(EDGE, args);
  console.log(`${asset.file} (${asset.width}x${asset.height})`);
}

console.log('Done.');
