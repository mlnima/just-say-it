import { createRequire } from 'node:module';
import { readFile, writeFile, rename, unlink, access } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_MODULE_PATH || 'playwright');
const directory = process.argv[2];
const browser = await chromium.launch({ channel: 'msedge', headless: false, args: ['--force-renderer-accessibility'] });
const page = await browser.newPage();
await page.goto(pathToFileURL(resolve('tests/desktop/browser-fixture.html')).href);
await page.locator('#first').click();
const connection = await browser.newBrowserCDPSession();
const { processInfo } = await connection.send('SystemInfo.getProcessInfo');
await writeFile(join(directory, 'ready'), JSON.stringify(processInfo.map((process) => process.id)));
while (true) {
  const commandFile = join(directory, 'command.json');
  const exists = await access(commandFile).then(() => true, () => false);
  if (exists) {
    const command = JSON.parse(await readFile(commandFile, 'utf8'));
    if (command.action === 'focus') await page.locator(`#${command.id}`).click();
    if (command.action === 'read') {
      const result = { first: await page.locator('#first').inputValue(), second: await page.locator('#second').inputValue(), editable: await page.locator('#editable').innerText() };
      await writeFile(join(directory, 'result.part'), JSON.stringify(result));
      await rename(join(directory, 'result.part'), join(directory, 'result.json'));
    }
    await unlink(commandFile);
    if (command.action === 'close') break;
  }
  await new Promise((resolve) => setTimeout(resolve, 50));
}
await browser.close();
