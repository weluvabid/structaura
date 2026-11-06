import assert from 'node:assert/strict';
import test, { before } from 'node:test';
import {
  compareFingerprints,
  fingerprintJavaScript,
  fingerprintSource,
  fingerprintTypeScript,
  init,
} from '../src/index.js';

before(async () => {
  await init();
});

test('initialization is idempotent', async () => {
  await init();
  await init();
});

test('JavaScript fingerprints ignore configured formatting and comment differences', () => {
  const compact = fingerprintJavaScript('const total = left + right;');
  const formatted = fingerprintJavaScript('// note\nconst total=(left+right)');
  assert.equal(compact.value, formatted.value);
  assert.equal(compact.language, 'javascript');
  assert.equal(compact.formatVersion, 1);
  assert.equal(compact.normalizationVersion, 1);
  assert.match(compact.digest, /^[a-f0-9]{64}$/);
});

test('TypeScript type structure affects fingerprints', () => {
  const number = fingerprintTypeScript('const value: number = 1;');
  const string = fingerprintTypeScript('const value: string = 1;');
  assert.notEqual(number.value, string.value);
  assert.equal(number.language, 'typescript');
});

test('generic dispatch agrees with the language-specific entry point', () => {
  assert.equal(
    fingerprintSource('const value = 1;', 'javascript').value,
    fingerprintJavaScript('const value = 1;').value,
  );
});

test('syntax errors and unsupported languages fail explicitly', () => {
  assert.throws(() => fingerprintJavaScript('const = ;'), SyntaxError);
  assert.throws(() => fingerprintSource('const value = 1;', 'rust'), TypeError);
  assert.throws(() => fingerprintJavaScript(42), TypeError);
});

test('fingerprint comparison rejects incompatible languages', () => {
  const javascript = fingerprintJavaScript('const value = 1;');
  const typescript = fingerprintTypeScript('const value: number = 1;');
  assert.throws(() => compareFingerprints(javascript, typescript));
  assert.equal(compareFingerprints(javascript, javascript), true);
});
