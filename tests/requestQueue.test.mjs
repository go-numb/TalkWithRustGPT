import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import ts from 'typescript';

const sourcePath = new URL('../src/common/requestQueue.ts', import.meta.url);
const source = await readFile(sourcePath, 'utf8');
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2020 },
  fileName: 'requestQueue.ts',
}).outputText;
const { RequestQueue } = await import(`data:text/javascript;base64,${Buffer.from(compiled).toString('base64')}`);

function deferred() {
  let resolve;
  let reject;
  const promise = new Promise((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

function started() {
  const signal = deferred();
  return signal;
}

test('runs the first request and one promoted reservation in sequence', async () => {
  const queue = new RequestQueue();
  const firstGate = deferred();
  const secondGate = deferred();
  const secondStarted = started();
  const calls = [];
  const run = queue.run('first', async (item) => {
    calls.push(item);
    if (item === 'first') await firstGate.promise;
    else {
      secondStarted.resolve();
      await secondGate.promise;
    }
  });

  assert.equal(queue.isRunning, true);
  assert.equal(queue.reserve('second'), true);
  assert.equal(queue.reserve('third'), false);
  firstGate.resolve();
  await secondStarted.promise;
  assert.deepEqual(calls, ['first', 'second']);
  assert.equal(queue.reserve('third'), true);
  secondGate.resolve();
  await run;

  assert.deepEqual(calls, ['first', 'second', 'third']);
  assert.equal(queue.isRunning, false);
  assert.equal(queue.pending, null);
});

test('cancels a reservation before it is sent', async () => {
  const queue = new RequestQueue();
  const gate = deferred();
  const calls = [];
  const run = queue.run('first', async (item) => {
    calls.push(item);
    await gate.promise;
  });

  assert.equal(queue.reserve('second'), true);
  assert.equal(queue.cancel(), 'second');
  assert.equal(queue.pending, null);
  gate.resolve();
  await run;
  assert.deepEqual(calls, ['first']);
});

test('clears a reservation and stops when sending fails', async () => {
  const queue = new RequestQueue();
  const failure = new Error('send failed');
  const run = queue.run('first', async () => {
    assert.equal(queue.reserve('second'), true);
    throw failure;
  });

  await assert.rejects(run, failure);
  assert.equal(queue.pending, null);
  assert.equal(queue.isRunning, false);
});

test('rejects concurrent runs and rejects reservations while idle', async () => {
  const queue = new RequestQueue();
  const gate = deferred();
  const run = queue.run('first', async () => gate.promise);

  assert.equal(queue.reserve('second'), true);
  await assert.rejects(queue.run('other', async () => {}), /already running/);
  assert.equal(queue.pending, 'second');
  gate.resolve();
  await run;
  assert.equal(queue.reserve('idle'), false);
});

test('can be run again after an error', async () => {
  const queue = new RequestQueue();
  await assert.rejects(queue.run('bad', async () => { throw new Error('bad'); }), /bad/);
  const calls = [];
  await queue.run('good', async (item) => { calls.push(item); });
  assert.deepEqual(calls, ['good']);
  assert.equal(queue.isRunning, false);
});
