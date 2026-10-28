/// <reference types="bun" />
// Check the generated site, including localized links, fragments, and asset paths.
// Bun's HTMLRewriter decodes HTML attributes without an additional parser dependency.
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

const dist = new URL('../dist/', import.meta.url);
const files = new Set();
for await (const file of new Bun.Glob('**/*').scan({ cwd: fileURLToPath(dist), onlyFiles: true })) {
  files.add(file);
}
if (!files.has('index.html')) throw new Error('Build the documentation before checking links.');

const pages = new Map();
for (const file of [...files].filter(file => file.endsWith('.html')).sort()) {
  const page = { ids: new Set(), links: [], canonical: undefined };
  const rewriter = new HTMLRewriter()
    .on('[id]', { element: element => { page.ids.add(element.getAttribute('id')); } })
    .on('a[href], area[href], link[href], script[src], img[src], iframe[src], source[src]', {
      element(element) {
        const href = element.getAttribute('href') ?? element.getAttribute('src');
        if (element.tagName === 'link' && element.getAttribute('rel') === 'canonical') page.canonical = href;
        page.links.push(href);
      },
    });
  await rewriter.transform(new Response(await readFile(new URL(file, dist)))).text();
  pages.set(file, page);
}

const home = pages.get('index.html');
if (!home.canonical) throw new Error('The built homepage must have a canonical URL.');
const base = new URL(home.canonical);
if (!base.pathname.endsWith('/')) base.pathname += '/';
const failures = [];
let checked = 0;
for (const [file, page] of pages) {
  const current = new URL(file.replace(/index\.html$/, ''), base);
  for (const href of new Set(page.links)) {
    const target = new URL(href, current);
    if (target.origin !== base.origin || !target.pathname.startsWith(base.pathname)) continue;
    checked++;
    const relative = decodeURIComponent(target.pathname.slice(base.pathname.length));
    const candidates = relative ? [relative, `${relative.replace(/\/$/, '')}/index.html`] : ['index.html'];
    const match = candidates.find(candidate => files.has(candidate));
    if (!match) {
      failures.push(`${file}: missing target ${href}`);
    } else if (target.hash && pages.has(match) && !pages.get(match).ids.has(decodeURIComponent(target.hash.slice(1)))) {
      failures.push(`${file}: missing anchor ${href}`);
    }
  }
}
if (failures.length) {
  console.error(failures.join('\n'));
  process.exitCode = 1;
} else {
  console.log(`Documentation links: ${pages.size} pages, ${checked} internal links and assets checked.`);
}
