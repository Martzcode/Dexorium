#!/usr/bin/env node
/**
 * Fills the Microsoft Store listing template exported from Partner Center.
 *
 *   node scripts/store-listing.mjs
 *
 * The template keeps one column per listing language; this script writes the
 * English copy to both `en-us` and `fr-fr` so the import never leaves a field
 * empty. Image columns hold paths relative to the ZIP uploaded to Partner
 * Center (the docs/store-listing folder).
 */

import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const DIR = 'docs/store-listing';
const template = readdirSync(DIR).find((file) => /^listingData-.*\.csv$/.test(file));
if (!template) throw new Error('No listingData-*.csv template found in docs/store-listing');

const DESCRIPTION = `Dexorium is a fast, fully offline Pokémon encyclopedia for players, collectors and competitive trainers.

No account, no ads, no telemetry. Every screen keeps working on a plane, at a tournament table or in a room without Wi-Fi.

WHAT IS INSIDE
• Pokédex — 1,025 species with types, base stats, abilities, measurements, breeding data, evolution chains, forms and learnsets.
• Moves — 919 moves with type, category, power, accuracy, PP and generation, plus filters to narrow them down.
• Abilities — 374 abilities with their in-game descriptions.
• Type chart — the complete effectiveness matrix, from generation 1 to generation 9.
• Search — jump to any species, move or ability with Ctrl+K.
• Favourites — keep the species you care about close at hand.

BUILT FOR OFFLINE USE
The whole reference database ships inside the app: nothing is downloaded at runtime, and your favourites stay on your device.

DESIGNED FOR WINDOWS
A native app built with Tauri and Rust, with instant startup, light and dark themes, English and French interfaces, and a compact layout that stays readable at 1366 × 768.

Dexorium is an unofficial, fan-made reference tool. It is not affiliated with, endorsed by or sponsored by Nintendo, Creatures Inc. or The Pokémon Company. Pokémon and Pokémon character names are trademarks of their respective owners.`;

const SHORT_DESCRIPTION =
  'Offline Pokémon encyclopedia: 1,025 species, 919 moves, 374 abilities and the full type chart. No account, no ads, no telemetry.';

const RELEASE_NOTES = `First Microsoft Store release.

• Offline Pokédex with 1,025 species
• 919 moves and 374 abilities with filters
• Complete type chart for generations 1 to 9
• Instant search (Ctrl+K) and local favourites
• Light and dark themes
• English and French interfaces
• No account, no ads, no telemetry`;

const FEATURES = [
  '1,025 species with types, base stats, abilities, breeding data, evolution chains and learnsets',
  '919 moves with filters by type, category, generation and power',
  '374 abilities with their in-game descriptions',
  'Complete type-effectiveness chart for generations 1 to 9',
  'Instant global search with Ctrl+K',
  'Favourites stored locally on your device',
  'Light and dark themes',
  'English and French interfaces',
  'Works fully offline: no account, no ads, no telemetry',
  'Native Windows app built with Tauri and Rust: fast startup, small footprint',
];

const MINIMUM_REQUIREMENTS = [
  'OS: Windows 10 version 2004 (build 19041) or later',
  'Architecture: x64',
  'Processor: 1 GHz or faster, 2 cores',
  'Memory: 4 GB RAM',
  'Storage: 250 MB available space',
  'Display: 1366 × 768 or higher',
  'Graphics: DirectX 9 compatible',
  'Microsoft Edge WebView2 Runtime (installed by the Microsoft Store)',
];

const RECOMMENDED_REQUIREMENTS = [
  'OS: Windows 11',
  'Memory: 8 GB RAM',
  'Storage: SSD with 500 MB available',
  'Display: 1920 × 1080 or higher',
];

const SCREENSHOTS = [
  ['screenshot-1-pokedex-dark-1920x1080.png', 'Browse 1,025 species with instant search'],
  ['screenshot-2-species-dark-1920x1080.png', 'Stats, abilities, breeding data and evolutions'],
  [
    'screenshot-3-type-matrix-light-1920x1080.png',
    'The complete type chart for generations 1 to 9',
  ],
  ['screenshot-4-moves-dark-1920x1080.png', 'Filter 919 moves by type, category and generation'],
  ['screenshot-5-abilities-dark-1920x1080.png', '374 abilities with in-game descriptions'],
  ['screenshot-6-home-dark-1920x1080.png', 'Every Pokémon, fully offline'],
  ['screenshot-7-settings-dark-1920x1080.png', 'Dataset details, themes and languages'],
];

/** field -> value written to both the fr-fr and en-us columns. */
const VALUES = {
  Description: DESCRIPTION,
  ReleaseNotes: RELEASE_NOTES,
  Title: 'Dexorium',
  ShortDescription: SHORT_DESCRIPTION,
  DevStudio: 'Martzcode',
  CopyrightTrademarkInformation:
    'Copyright © 2026 Martzcode. Pokémon and Pokémon character names are trademarks of their respective owners. Dexorium is an unofficial fan project, not affiliated with Nintendo, Creatures Inc. or The Pokémon Company.',
  AdditionalLicenseTerms:
    'Dexorium is free software distributed under the GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later). Source code and license text: https://github.com/Martzcode/Dexorium',
  OverrideLogosForWin10: 'False',
  StoreLogo300x300: 'images/logos/StoreLogo-300x300.png',
  StoreLogo720x1080: 'images/logos/StoreLogo-720x1080.png',
  StoreLogo1080x1080: 'images/logos/StoreLogo-1080x1080.png',
  PromoImage1920x1080: 'images/promo/promo-1920x1080.png',
  PromoImage2400x1200: 'images/promo/promo-2400x1200.png',
  OptionalPromo358x358: 'images/promo/promo-358x358.png',
  OptionalPromo1000x800: 'images/promo/promo-1000x800.png',
  OptionalPromo414x180: 'images/promo/promo-414x180.png',
};

SCREENSHOTS.forEach(([file, caption], index) => {
  VALUES[`DesktopScreenshot${index + 1}`] = `images/screenshots/${file}`;
  VALUES[`DesktopScreenshotCaption${index + 1}`] = caption;
});

FEATURES.forEach((feature, index) => {
  VALUES[`Feature${index + 1}`] = feature;
});

MINIMUM_REQUIREMENTS.forEach((requirement, index) => {
  VALUES[`MinimumHardwareReq${index + 1}`] = requirement;
});

RECOMMENDED_REQUIREMENTS.forEach((requirement, index) => {
  VALUES[`RecommendedHardwareReq${index + 1}`] = requirement;
});

const SEARCH_TERMS = [
  'pokedex',
  'pokemon encyclopedia',
  'offline pokedex',
  'type chart',
  'pokemon moves',
  'pokemon abilities',
  'trainer companion',
];
SEARCH_TERMS.forEach((term, index) => {
  VALUES[`SearchTerm${index + 1}`] = term;
});

/** Splits RFC 4180 CSV text into rows, keeping quoted newlines and commas. */
function parseCsv(text) {
  const rows = [];
  let row = [];
  let cell = '';
  let quoted = false;

  for (let index = 0; index < text.length; index += 1) {
    const char = text[index];

    if (quoted) {
      if (char === '"') {
        if (text[index + 1] === '"') {
          cell += '"';
          index += 1;
        } else {
          quoted = false;
        }
      } else {
        cell += char;
      }
      continue;
    }

    if (char === '"') {
      quoted = true;
    } else if (char === ',') {
      row.push(cell);
      cell = '';
    } else if (char === '\r' || char === '\n') {
      row.push(cell);
      rows.push(row);
      row = [];
      cell = '';
      if (char === '\r' && text[index + 1] === '\n') index += 1;
    } else {
      cell += char;
    }
  }

  if (cell !== '' || row.length > 0) {
    row.push(cell);
    rows.push(row);
  }

  return rows;
}

const escape = (value) => (/[",\r\n]/.test(value) ? `"${value.replaceAll('"', '""')}"` : value);

const source = readFileSync(join(DIR, template), 'utf8').replace(/^\uFEFF/, '');
const rows = parseCsv(source);
const seen = new Set();

const output = rows.map((cells) => {
  const [field, id, type, fallback] = cells;
  const value = VALUES[field];
  if (value === undefined) return cells;

  seen.add(field);
  return [field, id, type, fallback, value, value];
});

const missing = Object.keys(VALUES).filter((field) => !seen.has(field));
if (missing.length) {
  throw new Error(`Fields missing from the template: ${missing.join(', ')}`);
}

const csv = output.map((cells) => cells.map(escape).join(',')).join('\r\n');

writeFileSync(join(DIR, template), '\uFEFF' + csv, 'utf8');

console.log(`Filled ${seen.size} fields in ${template}`);
