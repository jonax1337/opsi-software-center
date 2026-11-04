import { createRequire } from 'node:module';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const moduleRoot = process.argv[2];
const require = createRequire(moduleRoot ? resolve(moduleRoot, 'package.json') : import.meta.url);
const sharp = require('sharp');
const exports = JSON.parse(await readFile(resolve(root, 'source/exports.json'), 'utf8'));

for (const item of exports) {
  const input = await readFile(resolve(root, item.input));
  const output = resolve(root, item.output);
  await mkdir(dirname(output), { recursive: true });
  await sharp(input, { density: 384, limitInputPixels: false })
    .resize({ width: item.width })
    .png({ compressionLevel: 9 })
    .toFile(output);
}

const sizes = [16, 24, 32, 48, 64, 128, 256];
const frames = await Promise.all(sizes.map(size => readFile(resolve(root, `png/mark/${size}.png`))));
const header = Buffer.alloc(6 + sizes.length * 16);
header.writeUInt16LE(1, 2);
header.writeUInt16LE(sizes.length, 4);
let offset = header.length;
frames.forEach((frame, index) => {
  const entry = 6 + index * 16;
  header[entry] = sizes[index] === 256 ? 0 : sizes[index];
  header[entry + 1] = header[entry];
  header.writeUInt16LE(1, entry + 4);
  header.writeUInt16LE(32, entry + 6);
  header.writeUInt32LE(frame.length, entry + 8);
  header.writeUInt32LE(offset, entry + 12);
  offset += frame.length;
});
await mkdir(resolve(root, 'icons'), { recursive: true });
await writeFile(resolve(root, 'icons/favicon.ico'), Buffer.concat([header, ...frames]));
console.log(`Exported ${exports.length} PNG files and a seven-size favicon.`);
