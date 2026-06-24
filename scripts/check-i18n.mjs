import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const rootDir = join(__dirname, '..');

/**
 * Recursively collect all leaf key paths from a nested object.
 * @param {object} obj
 * @param {string} prefix
 * @returns {string[]}
 */
function deepKeys(obj, prefix = '') {
  let keys = [];
  for (const [key, value] of Object.entries(obj)) {
    const path = prefix ? `${prefix}.${key}` : key;
    if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
      keys = keys.concat(deepKeys(value, path));
    } else {
      keys.push(path);
    }
  }
  return keys;
}

/**
 * Read and parse a locale JSON file.
 * @param {string} locale
 * @returns {object}
 */
function readLocale(locale) {
  const filePath = join(rootDir, 'src', 'i18n', `${locale}.json`);
  return JSON.parse(readFileSync(filePath, 'utf-8'));
}

const locales = ['pt', 'es', 'fr'];

let reference;
try {
  reference = readLocale('en');
} catch (err) {
  console.error(`Failed to read reference locale en.json: ${err.message}`);
  process.exit(1);
}

const referenceKeys = new Set(deepKeys(reference));
let exitCode = 0;

for (const locale of locales) {
  let data;
  try {
    data = readLocale(locale);
  } catch (err) {
    console.error(`[${locale}] Failed to read file: ${err.message}`);
    exitCode = 1;
    continue;
  }

  const localeKeys = new Set(deepKeys(data));

  for (const key of referenceKeys) {
    if (!localeKeys.has(key)) {
      console.error(`[${locale}] Missing key: ${key}`);
      exitCode = 1;
    }
  }
}

if (exitCode === 0) {
  console.log('All locale files are complete — no missing keys.');
}
process.exit(exitCode);