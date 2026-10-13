#!/usr/bin/env node
// Serves docs/openapi.json as an interactive Scalar reference.
//
// Usage:
//   node scripts/openapi/preview.mjs
//
// The spec is gitignored, so it is generated on demand when absent. It is
// re-read per request rather than cached, so `just openapi` in another terminal
// shows up on a refresh.
//
// `servers` is injected in memory to point "Try it out" at the running API. It
// is deliberately never written back: the generator does not emit one, and
// persisting it would make `just openapi-check` fail against its own output.
//
// This is a standalone viewer rather than a route on the API. The spec used to
// be embedded in the binary with `include_str!`, which only worked on a machine
// that had already run `just openapi` — the gitignore makes that a chicken-and-
// egg. Reading it at runtime needs no Rust rebuild and no database, so the docs
// work even when the app is down.

import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:http';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '..', '..');
const SPEC_PATH = join(ROOT, 'docs', 'openapi.json');
const GENERATOR = join(__dirname, 'generate.mjs');

// `DOCS_PORT`, not `PORT`: scripts/email-preview.mjs already reads `PORT`, and
// sharing the name would let the two silently collide on the same socket.
const PORT = Number(process.env.DOCS_PORT) || 5859;

// The dev API, per docker-compose.yaml. Scalar defaults "Try it out" to the
// origin serving the page — this preview — so without this every request 404s.
const API_URL = process.env.OPENAPI_API_URL || 'http://localhost:8000';

const SCALAR_CDN = 'https://cdn.jsdelivr.net/npm/@scalar/api-reference';

// ---------------------------------------------------------------------------
// Spec
// ---------------------------------------------------------------------------

/**
 * Generates the spec if it is not there yet.
 *
 * Called once at boot so the cost is paid before the first request and any
 * generator failure is reported in the terminal rather than as a browser error.
 */
function ensureSpec() {
  if (existsSync(SPEC_PATH)) return;

  console.log('docs/openapi.json is missing; generating it now...');
  execFileSync(process.execPath, [GENERATOR], { stdio: 'inherit' });
}

/**
 * The spec as this viewer serves it: the document plus a `servers` entry aimed
 * at the running API.
 *
 * An existing `servers` key is overridden rather than respected. The generator
 * does not emit one, so there is nothing to preserve today, and a preview whose
 * "Try it out" points somewhere unexpected is worse than one that points at the
 * URL the operator configured.
 *
 * The entry is unlabelled because `API_URL` is not necessarily a development
 * server — it is whatever the operator pointed the viewer at.
 */
function readSpec() {
  const document = JSON.parse(readFileSync(SPEC_PATH, 'utf8'));

  return {
    ...document,
    servers: [{ url: API_URL }],
  };
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

/** Escapes text interpolated into the page. The title comes from our own generated spec, but a doc viewer is still the wrong place to be the injection point. */
function escapeHtml(text) {
  return String(text).replace(
    /[&<>"']/g,
    (character) =>
      ({
        '&': '&amp;',
        '<': '&lt;',
        '>': '&gt;',
        '"': '&quot;',
        "'": '&#39;',
      })[character],
  );
}

/**
 * The Scalar shell.
 *
 * `data-url` is relative so the page fetches the spec from whatever host it was
 * served on, rather than a hardcoded localhost that breaks behind a tunnel or a
 * container port mapping. The `dataUrl` argument therefore points at the spec
 * relative to the serving root: `/openapi.json` for the live preview, plain
 * `openapi.json` for a static build sitting beside the file on GitHub Pages.
 * The UI is loaded from a CDN, so viewing it needs network access; nothing
 * else about the preview does.
 */
function readIndexHtml(title, dataUrl = '/openapi.json') {
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>${escapeHtml(title)}</title>
  </head>
  <body>
    <script id="api-reference" data-url="${escapeHtml(dataUrl)}" data-theme="light"></script>
    <script src="${SCALAR_CDN}"></script>
  </body>
</html>
`;
}

function sendNotFound(res) {
  res.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' });
  res.end('Not found');
}

// ---------------------------------------------------------------------------
// Static build
// ---------------------------------------------------------------------------

/**
 * Emits a self-contained doc site into `outdir`: the spec plus an `index.html`
 * Scalar shell that reads it relative to the page. That pair can sit behind any
 * URL prefix — a GitHub Pages project site — without a server of its own, so a
 * workflow can publish the docs the same way it would publish any build output.
 *
 * The `servers` injection in `readSpec` applies here too, so an `OPENAPI_API_URL`
 * in the runner's environment points "Try it out" at the API.
 */
function buildStatic(outdir) {
  ensureSpec();

  const document = readSpec();
  mkdirSync(outdir, { recursive: true });

  writeFileSync(join(outdir, 'openapi.json'), JSON.stringify(document, null, 2) + '\n');
  writeFileSync(
    join(outdir, 'index.html'),
    readIndexHtml(document.info?.title ?? 'API reference', 'openapi.json'),
  );

  const operations = Object.values(document.paths ?? {}).reduce(
    (total, path) => total + Object.keys(path).length,
    0,
  );
  console.log(`Wrote OpenAPI docs to ${outdir} (${operations} operations).`);
}

// ---------------------------------------------------------------------------
// Server
// ---------------------------------------------------------------------------

ensureSpec();

const staticIdx = process.argv.indexOf('--static');
if (staticIdx >= 0) {
  buildStatic(resolve(process.argv[staticIdx + 1] || '_openapi'));
} else {
const server = createServer((req, res) => {
  const { pathname } = new URL(req.url, `http://localhost:${PORT}`);

  if (pathname === '/favicon.ico') {
    res.writeHead(204);
    res.end();
    return;
  }

  if (pathname === '/openapi.json') {
    let document;

    try {
      document = readSpec();
    } catch (err) {
      res.writeHead(500, { 'Content-Type': 'text/plain; charset=utf-8' });
      res.end(
        `Could not read docs/openapi.json.\n\n${err.message}\n\nRun: just openapi\n`,
      );
      return;
    }

    res.writeHead(200, { 'Content-Type': 'application/json; charset=utf-8' });
    res.end(JSON.stringify(document, null, 2) + '\n');
    return;
  }

  if (pathname === '/') {
    // The title is cosmetic. If the spec cannot be read the page still renders,
    // and Scalar surfaces the fetch failure in the browser with more context
    // than a 500 here would.
    let title = 'API reference';

    try {
      title = readSpec().info?.title ?? title;
    } catch {
      // Ignored deliberately: see above.
    }

    res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
    res.end(readIndexHtml(title));
    return;
  }

  sendNotFound(res);
});

server.listen(PORT, () => {
  const operations = (() => {
    try {
      return Object.values(readSpec().paths ?? {}).reduce(
        (total, path) => total + Object.keys(path).length,
        0,
      );
    } catch {
      return null;
    }
  })();

  console.log(`OpenAPI preview running at http://localhost:${PORT}`);
  console.log(`Spec: ${SPEC_PATH}${operations === null ? '' : ` (${operations} operations)`}`);
  console.log(`"Try it out" targets: ${API_URL} (override with OPENAPI_API_URL)`);
  console.log('Press Ctrl+C to stop.');
});
}
