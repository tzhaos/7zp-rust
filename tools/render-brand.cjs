// Asset generation only. Requires sharp; the editable source is assets/brand/logo.svg.
const fs = require('node:fs/promises');
const path = require('node:path');
const sharp = require('sharp');

async function main() {
  const root = path.resolve(__dirname, '..');
  const brand = path.join(root, 'assets/brand');
  await fs.mkdir(brand, { recursive: true });
  const source = await fs.readFile(path.join(brand, 'logo.svg'));
  await sharp(source, { density: 288 }).resize(256, 256).png().toFile(path.join(brand, 'logo.png'));
}

main().catch(error => { console.error(error); process.exitCode = 1; });
