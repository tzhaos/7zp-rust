// Import Microsoft's Fluent Filled assets from the prototype's pinned package.
const fs = require('node:fs/promises');
const path = require('node:path');
const { createRequire } = require('node:module');

async function main() {
  const fromPrototype = createRequire(path.resolve(process.argv[2], 'package.json'));
  const React = fromPrototype('react');
  const { renderToStaticMarkup } = fromPrototype('react-dom/server');
  const icons = fromPrototype('@fluentui/react-icons');
  const assets = path.resolve(__dirname, '../assets/fluent');
  const sizes = {
    FolderOpen: 24,
    ArrowDownload: 24,
    ArchiveArrowBack: 24,
    ShieldCheckmark: 24,
    MoreHorizontal: 24,
    Folder: 20,
    DocumentImage: 20,
    Code: 20,
    DocumentText: 20,
    FolderZip: 20,
    Document: 20,
    CheckmarkCircle: 20,
  };
  for (const [name, size] of Object.entries(sizes)) {
    const svg = renderToStaticMarkup(React.createElement(icons[`${name}${size}Filled`]));
    await fs.writeFile(path.join(assets, `${name}Filled.svg`), `${svg}\n`);
  }
}

main().catch(error => { console.error(error); process.exitCode = 1; });
