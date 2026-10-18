import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { isAbsolute, join } from 'node:path';
import { test } from 'node:test';
import { buildEnvironment, projectDirectory } from '../build-environment.mjs';

test('preserves explicit binding and libclang configuration without mutating the environment', () => {
  const source = { LIBCLANG_PATH: 'custom-clang', BINDGEN_EXTRA_CLANG_ARGS: '-Icustom-headers' };
  const environment = buildEnvironment(source);
  assert.deepEqual(environment, source);
  assert.notEqual(environment, source);
});

test('discovers installed build headers and a checkout-relative libclang', () => {
  const environment = buildEnvironment();
  assert.equal(isAbsolute(projectDirectory), true);
  if (process.platform !== 'win32') {
    assert.deepEqual(environment, process.env);
    return;
  }
  assert.ok(environment.BINDGEN_EXTRA_CLANG_ARGS);
  const localClang = join(projectDirectory, 'artifacts', 'build-tools', 'clang', 'native');
  if (!process.env.LIBCLANG_PATH && existsSync(join(localClang, 'libclang.dll'))) {
    assert.equal(environment.LIBCLANG_PATH, localClang);
  }
});
