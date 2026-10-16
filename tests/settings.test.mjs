import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import ts from 'typescript';

const sourcePath = new URL('../src/common/settings.ts', import.meta.url);
const source = await readFile(sourcePath, 'utf8');
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2020 },
  fileName: 'settings.ts',
}).outputText;
const {
  BUILTIN_TTS_DEFAULTS, FEMALE_VOICES, MALE_VOICES, TTS_MODELS,
  isTtsId, loadSavedTtsSettings, mergeTtsSettings, parseSavedTtsSettings, storeSavedTtsSettings, withCurrentOption,
} = await import(`data:text/javascript;base64,${Buffer.from(compiled).toString('base64')}`);

test('settings saved in the settings tab override the defaults field by field', () => {
  const defaults = { model: 'gemini-3.8-flash-tts', femaleVoice: 'ja-jp-techagent-7', maleVoice: 'Charon' };
  assert.deepEqual(
    mergeTtsSettings(defaults, { femaleVoice: 'Kore' }),
    { model: 'gemini-3.8-flash-tts', femaleVoice: 'Kore', maleVoice: 'Charon' },
  );
});

test('invalid values never replace a default', () => {
  const merged = mergeTtsSettings(BUILTIN_TTS_DEFAULTS, { model: '', femaleVoice: 'bad voice', maleVoice: 42 });
  assert.deepEqual(merged, BUILTIN_TTS_DEFAULTS);
  assert.deepEqual(mergeTtsSettings(BUILTIN_TTS_DEFAULTS, null), BUILTIN_TTS_DEFAULTS);
});

test('parses only valid saved fields and tolerates broken storage', () => {
  assert.deepEqual(parseSavedTtsSettings('{"maleVoice":"Iapetus","extra":"x","model":"a b"}'), { maleVoice: 'Iapetus' });
  assert.deepEqual(parseSavedTtsSettings('{broken'), {});
  assert.deepEqual(parseSavedTtsSettings(null), {});
  assert.deepEqual(parseSavedTtsSettings('"Kore"'), {});
});

test('accepts the same ids as the Rust command', () => {
  for (const id of ['Erinome', 'ja-jp-advisor-8', 'gemini-3.8-flash-tts', 'voice_abc']) assert.equal(isTtsId(id), true, id);
  for (const id of ['', 'has space', 'a/b', '"quote', 'x'.repeat(65)]) assert.equal(isTtsId(id), false, id);
});

test('shows an env-specified value that is not in the built-in list', () => {
  assert.equal(withCurrentOption(MALE_VOICES, 'Charon').length, MALE_VOICES.length);
  const options = withCurrentOption(MALE_VOICES, 'ja-jp-advisor-8');
  assert.deepEqual(options[0], { value: 'ja-jp-advisor-8', label: 'ja-jp-advisor-8' });
  assert.equal(options.length, MALE_VOICES.length + 1);
});

test('keeps the env default selectable after another voice is chosen', () => {
  const options = withCurrentOption(MALE_VOICES, 'Charon', 'ja-jp-advisor-8');
  assert.ok(options.some((o) => o.value === 'ja-jp-advisor-8'));
  assert.equal(options.length, MALE_VOICES.length + 1);
  assert.equal(withCurrentOption(MALE_VOICES, 'x-1', 'x-1').length, MALE_VOICES.length + 1, 'no duplicates');
});

test('stores only overrides and removes the key when cleared', () => {
  const store = new Map();
  globalThis.localStorage = {
    getItem: (k) => (store.has(k) ? store.get(k) : null),
    setItem: (k, v) => store.set(k, String(v)),
    removeItem: (k) => store.delete(k),
  };
  try {
    storeSavedTtsSettings({ maleVoice: 'Iapetus' });
    assert.deepEqual(loadSavedTtsSettings(), { maleVoice: 'Iapetus' });
    storeSavedTtsSettings({});
    assert.equal(store.size, 0);
    assert.deepEqual(loadSavedTtsSettings(), {});
  } finally {
    delete globalThis.localStorage;
  }
  assert.deepEqual(loadSavedTtsSettings(), {}, 'missing storage falls back to no overrides');
});

test('built-in defaults are listed and voice lists do not overlap', () => {
  assert.ok(TTS_MODELS.some((m) => m.value === BUILTIN_TTS_DEFAULTS.model));
  assert.ok(FEMALE_VOICES.some((v) => v.value === BUILTIN_TTS_DEFAULTS.femaleVoice));
  assert.ok(MALE_VOICES.some((v) => v.value === BUILTIN_TTS_DEFAULTS.maleVoice));
  const female = new Set(FEMALE_VOICES.map((v) => v.value));
  assert.ok(MALE_VOICES.every((v) => !female.has(v.value)));
  assert.ok([...FEMALE_VOICES, ...MALE_VOICES, ...TTS_MODELS].every((o) => isTtsId(o.value)));
});
