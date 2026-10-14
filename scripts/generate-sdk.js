#!/usr/bin/env node
// Generates Go and TypeScript SDKs from the OpenAPI spec.
//
// Pipeline:
//   1. Regenerate docs/openapi.json from the Axum routes and handlers.
//   2. Generate the Go SDK into sdk-go/ (oapi-codegen via `go run`).
//   3. Generate the TypeScript SDK into sdk-js/ (openapi-generator typescript-fetch).
//
// Usage:
//   node scripts/generate-sdk.js

import { execFileSync, execSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '..');

const SPEC = join(ROOT, 'docs', 'openapi.json');
const GO_OUT = join(ROOT, 'sdk-go');
const TS_OUT = join(ROOT, 'sdk-js');
const CLI = join(__dirname, 'node_modules', '@openapitools', 'openapi-generator-cli', 'main.js');

const OAPI_CODEGEN = 'github.com/oapi-codegen/oapi-codegen/v2/cmd/oapi-codegen@latest';

function findGo() {
  for (const candidate of ['/usr/local/go/bin/go', '/opt/homebrew/bin/go']) {
    if (existsSync(candidate)) return candidate;
  }
  try {
    return execSync('which go', { encoding: 'utf8' }).trim();
  } catch {
    console.error('Go not found. Install from https://go.dev/dl/');
    process.exit(1);
  }
}

const GO = findGo();

function run(cmd, args, opts = {}) {
  execFileSync(cmd, args, { stdio: 'inherit', cwd: ROOT, ...opts });
}

function generator(args) {
  run(process.execPath, [CLI, ...args]);
}

function cargoVersion() {
  const cargo = readFileSync(join(ROOT, 'Cargo.toml'), 'utf8');
  const m = cargo.match(/^\s*version\s*=\s*"([^"]+)"/m);
  return m ? m[1] : '0.0.0';
}

// 1. Regenerate the spec from the Rust surface.
console.log('[1/3] Regenerating docs/openapi.json');
run(process.execPath, [join(__dirname, 'openapi', 'generate.mjs')]);

// 2. Go SDK via oapi-codegen.
console.log('[2/3] Generating Go SDK -> sdk-go/');
mkdirSync(GO_OUT, { recursive: true });
run(GO, [
  'run', OAPI_CODEGEN,
  '-package', 'asterisk',
  '-generate', 'types,client,spec',
  '-o', join(GO_OUT, 'client.go'),
  '-response-type-suffix', 'Resp',
  SPEC,
], { cwd: ROOT });

if (!existsSync(join(GO_OUT, 'go.mod'))) {
  run(GO, ['mod', 'init', 'github.com/opeolluwa/asterisk/sdk-go'], { cwd: GO_OUT });
}
run(GO, ['mod', 'tidy'], { cwd: GO_OUT });

writeFileSync(join(GO_OUT, 'README.md'), `# asterisk Go SDK

Go client for the asterisk API, generated from the OpenAPI spec with
[oapi-codegen](https://github.com/oapi-codegen/oapi-codegen).

## Installation

\`\`\`bash
go get github.com/opeolluwa/asterisk/sdk-go
\`\`\`

## Usage

\`\`\`go
package main

import (
\t"context"
\t"fmt"
\t"log"
\t"net/http"

\tasterisk "github.com/opeolluwa/asterisk/sdk-go"
)

func main() {
\t// Add a bearer token to every request.
\tbearerAuth := func(ctx context.Context, req *http.Request) error {
\t\treq.Header.Set("Authorization", "Bearer YOUR_TOKEN")
\t\treturn nil
\t}

\tclient, err := asterisk.NewClientWithResponses(
\t\t"https://api.example.com",
\t\tasterisk.WithRequestEditorFn(bearerAuth),
\t)
\tif err != nil {
\t\tlog.Fatal(err)
\t}

\t// Fetch the app config.
\tresp, err := client.FetchAppConfigWithResponse(context.Background())
\tif err != nil {
\t\tlog.Fatal(err)
\t}
\tfmt.Printf("Status: %d\\n", resp.StatusCode())

\t// Log in.
\tloginResp, err := client.LoginWithResponse(context.Background(), asterisk.LoginJSONRequestBody{
\t\tEmail:    "admin@example.com",
\t\tPassword: "secret",
\t})
\tif err != nil {
\t\tlog.Fatal(err)
\t}
\tfmt.Printf("Login status: %d\\n", loginResp.StatusCode())
}
\`\`\`

## Regenerating

From the repository root:

\`\`\`bash
just sdk      # regenerate all SDKs (Go + TypeScript)
just sdk-go   # regenerate Go SDK only
\`\`\`
`);

// 3. TypeScript SDK via openapi-generator.
if (existsSync(CLI)) {
  console.log('[3/3] Generating TypeScript SDK -> sdk-js/');
  generator([
    'generate',
    '-i', SPEC,
    '-g', 'typescript-fetch',
    '-o', TS_OUT,
    `--additional-properties=npmName=@opeolluwa/asterisk,npmVersion=${cargoVersion()},gitUserId=opeolluwa,gitRepoId=asterisk,supportsES6=true,hideGenerationTimestamp=true`,
    '--global-property=apis,models,supportingFiles',
    '--skip-validate-spec',
  ]);
} else {
  console.log('[3/3] Skipping TypeScript SDK (openapi-generator-cli not installed)');
}

console.log('\nDone.');
