import assert from 'node:assert/strict';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const { chromium } = require(process.argv[2] || 'playwright');
const browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
const page = browser.contexts().flatMap((context) => context.pages())[0];
const rpc = (command) => page.evaluate(async (command) => {
  const { request } = await import('/src/bridge/commands.ts');
  return request(command);
}, command);
const original = (await rpc('get_snapshot')).settings;
try {
  for (const model of await rpc('get_models')) {
    assert.equal(model.installed_path, null, 'Use an isolated test directory without catalog downloads.');
    await page.getByRole('button', { name: model.kind === 'speech' ? 'Audio & model' : 'Activation', exact: true }).click();
    const select = page.getByRole('combobox', { name: model.kind === 'speech' ? 'Local Whisper model' : 'Local wake model', exact: true });
    const before = await select.inputValue();
    const row = page.locator('.model-download').filter({ hasText: model.label });
    await row.getByRole('button', { name: 'Download', exact: true }).click();
    const deadline = Date.now() + 240000;
    while (!(await rpc('get_models')).find((candidate) => candidate.id === model.id)?.installed_path) {
      const status = await rpc('get_snapshot');
      if (!status.download && /failed|interrupted|Insufficient|Cannot|exceeded|cancelled/.test(status.message)) throw new Error(status.message);
      assert.ok(Date.now() < deadline, 'The real model download did not finish.');
      await new Promise((resolve) => setTimeout(resolve, 250));
    }
    await row.getByText(/Installed and load checked/).waitFor();
    assert.equal(await select.inputValue(), before, 'Downloading a model must not select it silently.');
    console.log(`${model.kind}: real network download, checksum, native loading, installation and UI refresh passed.`);
  }
  assert.deepEqual((await rpc('get_snapshot')).settings, original, 'Downloads changed saved preferences.');
} finally { await browser.close(); }
