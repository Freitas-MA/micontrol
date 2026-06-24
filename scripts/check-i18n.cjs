#!/usr/bin/env node
/**
 * Validates that all locale files have the same keys as en.json (source of truth).
 * Run: node scripts/check-i18n.cjs
 */

const fs = require('fs');
const path = require('path');

const i18nDir = path.join(__dirname, '..', 'src', 'i18n');
const en = JSON.parse(fs.readFileSync(path.join(i18nDir, 'en.json'), 'utf8'));

function getLeafKeys(obj, prefix = '') {
  const keys = [];
  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    if (typeof value === 'object' && value !== null) {
      keys.push(...getLeafKeys(value, fullKey));
    } else {
      keys.push(fullKey);
    }
  }
  return keys;
}

const enKeys = new Set(getLeafKeys(en));
const locales = ['pt', 'es', 'fr'];
let hasErrors = false;

for (const locale of locales) {
  const localeData = JSON.parse(fs.readFileSync(path.join(i18nDir, `${locale}.json`), 'utf8'));
  const localeKeys = new Set(getLeafKeys(localeData));
  
  const missing = [...enKeys].filter(k => !localeKeys.has(k));
  const extra = [...localeKeys].filter(k => !enKeys.has(k));
  
  if (missing.length > 0) {
    console.error(`❌ ${locale}.json is missing ${missing.length} keys:`);
    missing.forEach(k => console.error(`   - ${k}`));
    hasErrors = true;
  }
  
  if (extra.length > 0) {
    console.warn(`⚠️  ${locale}.json has ${extra.length} extra keys not in en.json:`);
    extra.forEach(k => console.warn(`   - ${k}`));
  }
  
  if (missing.length === 0 && extra.length === 0) {
    console.log(`✅ ${locale}.json has all keys matching en.json`);
  }
}

if (hasErrors) {
  process.exit(1);
}