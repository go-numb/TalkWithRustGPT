import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import ts from 'typescript';

const sourcePath = new URL('../src/common/speech.ts', import.meta.url);
const source = await readFile(sourcePath, 'utf8');
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2020 },
  fileName: 'speech.ts',
}).outputText;
const { SpeechSession, splitForSpeech, toSpeechText, isTtsVoice, toArrayBuffer } =
  await import(`data:text/javascript;base64,${Buffer.from(compiled).toString('base64')}`);

function deferred() {
  let resolve;
  let reject;
  const promise = new Promise((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

const tick = () => new Promise((resolve) => setImmediate(resolve));
const audioOf = (text) => new TextEncoder().encode(text).buffer;
const textOf = (audio) => new TextDecoder().decode(audio);

test('removes the response footer and blank lines before reading', () => {
  const text = '回答です。\n\n\n  次の段落。 \n\nModel: gemini-3.8-flash, Total token: 30, Elaps: 2s';
  assert.equal(toSpeechText(text), '回答です。\n次の段落。');
});

test('normalizes IPC audio to an ArrayBuffer', () => {
  const buffer = new Uint8Array([82, 73, 70, 70]).buffer;
  assert.equal(toArrayBuffer(buffer), buffer);
  assert.equal(textOf(toArrayBuffer([82, 73, 70, 70])), 'RIFF', 'postMessage fallback sends a number array');
  assert.equal(textOf(toArrayBuffer(new Uint8Array([82, 73, 70, 70]))), 'RIFF');
});

test('accepts only the supported voices', () => {
  assert.equal(isTtsVoice('female'), true);
  assert.equal(isTtsVoice('male'), true);
  assert.equal(isTtsVoice('Kore'), false);
  assert.equal(isTtsVoice(null), false);
});

test('splits at sentence ends with a short first chunk', () => {
  const sentence = 'これは読み上げのテストの文です。'; // 16 chars
  const text = sentence.repeat(40);
  const chunks = splitForSpeech(text, 100, 40);

  assert.ok(chunks[0].length <= 40, chunks[0]);
  assert.ok(chunks.slice(1).every((chunk) => chunk.length <= 100));
  assert.ok(chunks.every((chunk) => chunk.endsWith('。')), 'chunks end at sentence boundaries');
  assert.equal(chunks.join(''), text);
});

test('does not treat decimal points as sentence ends', () => {
  const chunks = splitForSpeech('円周率は 3.14 です。Next sentence. Last one', 12, 12);
  assert.ok(chunks.some((chunk) => chunk.includes('3.14')), JSON.stringify(chunks));
});

test('hard-splits long text without punctuation and keeps surrogate pairs', () => {
  const text = '🎉'.repeat(30);
  const chunks = splitForSpeech(text, 10, 10);
  assert.equal(chunks.length, 3);
  assert.equal(chunks.join(''), text);
  assert.ok(chunks.every((chunk) => Array.from(chunk).length === 10));
});

test('keeps line breaks that follow a sentence end', () => {
  assert.deepEqual(splitForSpeech('見出し\n一つ目。\n二つ目。'), ['見出し\n一つ目。\n二つ目。']);
});

test('joins a very short last chunk to the previous one', () => {
  const sentence = 'あ'.repeat(15) + '。';
  assert.deepEqual(splitForSpeech(sentence + sentence + 'はい。', 17, 17), [sentence, `${sentence}\nはい。`]);
  assert.deepEqual(splitForSpeech('OK.'), ['OK.'], 'a short answer is still read');
});

test('returns no chunks for blank text', () => {
  assert.deepEqual(splitForSpeech('  \n '), []);
});

test('plays chunks in order and prefetches the next one during playback', async () => {
  const session = new SpeechSession();
  const calls = [];
  const played = [];
  const playing = [];
  const synthesize = async (text) => {
    calls.push(text);
    return audioOf(text);
  };
  const playAudio = (audio) => {
    played.push(textOf(audio));
    const done = deferred();
    playing.push(done);
    return done.promise;
  };

  const result = session.speak(['a', 'b', 'c'], synthesize, playAudio);
  await tick();
  assert.deepEqual(played, ['a']);
  assert.deepEqual(calls, ['a', 'b'], 'next chunk is requested while the first plays');
  assert.equal(session.active, true);

  playing[0].resolve();
  await tick();
  assert.deepEqual(played, ['a', 'b']);
  playing[1].resolve();
  await tick();
  playing[2].resolve();

  assert.equal(await result, true);
  assert.deepEqual(played, ['a', 'b', 'c']);
  assert.equal(session.active, false);
});

test('stop() during synthesis resolves false without playing', async () => {
  const session = new SpeechSession();
  const pending = deferred();
  let playedCount = 0;

  const result = session.speak(['a'], () => pending.promise, async () => { playedCount++; });
  session.stop();
  assert.equal(await result, false);
  assert.equal(session.active, false);

  pending.resolve(audioOf('a'));
  await tick();
  assert.equal(playedCount, 0);
});

test('stop() aborts current playback and ignores a failing prefetch', async () => {
  const session = new SpeechSession();
  const prefetch = deferred();
  let signal;
  const synthesize = (text) => (text === 'a' ? Promise.resolve(audioOf('a')) : prefetch.promise);
  const playAudio = (_audio, abortSignal) => {
    signal = abortSignal;
    return new Promise((resolve) => abortSignal.addEventListener('abort', resolve));
  };

  const result = session.speak(['a', 'b'], synthesize, playAudio);
  await tick();
  session.stop();
  assert.equal(signal.aborted, true);
  assert.equal(await result, false);

  // A rejected prefetch after stop must not surface as an unhandled rejection.
  prefetch.reject(new Error('late failure'));
  await tick();
});

test('a new speak() cancels the previous one', async () => {
  const session = new SpeechSession();
  const first = deferred();
  const played = [];
  const playAudio = async (audio) => { played.push(textOf(audio)); };

  const previous = session.speak(['old'], () => first.promise, playAudio);
  const next = session.speak(['new'], async (text) => audioOf(text), playAudio);

  assert.equal(await previous, false);
  assert.equal(await next, true);
  first.resolve(audioOf('old'));
  await tick();
  assert.deepEqual(played, ['new']);
});

test('synthesis errors reject and release the session', async () => {
  const session = new SpeechSession();
  await assert.rejects(
    session.speak(['a'], async () => { throw new Error('TTS API error (400)'); }, async () => {}),
    /TTS API error/,
  );
  assert.equal(session.active, false);
});
