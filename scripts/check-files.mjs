import { readdir, readFile } from 'node:fs/promises';
import { extname, join } from 'node:path';

const ignored = new Set(['.git', 'node_modules', 'target', 'dist', 'gen', 'artifacts']);
const extensions = new Set(['.rs', '.ts', '.tsx', '.js', '.mjs', '.css', '.html', '.ps1']);
const visit = async (directory) => {
  const results = await Promise.all((await readdir(directory, { withFileTypes: true })).map(async (entry) => {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) return ignored.has(entry.name) ? [] : visit(path);
    if (!extensions.has(extname(path))) return [];
    const content = await readFile(path, 'utf8');
    const count = content.length ? content.split('\n').length - Number(content.endsWith('\n')) : 0;
    return count < 300 ? [] : [`${path}: ${count} lines`];
  }));
  return results.flat();
};
const failures = await visit('.');
failures.forEach((failure) => console.error(failure));
process.exitCode = failures.length ? 1 : 0;
if (!failures.length) console.log('Every authored code file contains fewer than 300 lines.');
