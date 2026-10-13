#!/usr/bin/env node
// Generates docs/openapi.json by scanning the Rust REST surface directly.
//
// There is no OpenAPI framework in the dependency tree: this is a pure text
// parser. It reads the `Router` chains in `src/routes/*.rs` to discover
// endpoints, the handler signatures in `src/handlers/*.rs` for request bodies and
// response shapes, and the `derive` attributes on the structs in
// `src/extractors`, `src/dto` and `src/models` to build component schemas. It
// emits an OpenAPI 3.1 document and, with `--check`, diffs it against the
// committed one.
//
// Usage:
//   node scripts/openapi/generate.mjs          # write docs/openapi.json
//   node scripts/openapi/generate.mjs --check  # exit non-zero if stale

import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '..', '..');
const OUT = join(ROOT, 'docs', 'openapi.json');

const HTTP_METHODS = ['get', 'post', 'put', 'patch', 'delete', 'head', 'options'];

// Meta routes that serve documentation rather than API data. The human-facing
// Scalar UI returns HTML, so it is not part of the API contract.
const IGNORED_PATHS = new Set(['/api-docs']);

const TAGS = {
  auth: 'Authentication',
  customer_auth: 'Customer Authentication',
  users: 'Users',
  products: 'Products',
  teams: 'Team',
  email: 'Emails',
  invitation: 'Invitations',
  uploads: 'Uploads',
  orders: 'Orders',
  customer: 'Customers',
  complaint: 'Complaints',
  config: 'App Config',
  roles: 'Roles',
  inventory: 'Inventory',
  marketplace: 'Marketplace',
  customer_address: 'Customer Addresses',
  cart: 'Cart',
  notifications: 'Notifications',
  customer_order: 'Customer Orders',
  customer_complaint: 'Customer Complaints',
  country: 'Countries',
  payment: 'Payments',
  public: 'Public',
};

// ---------------------------------------------------------------------------
// Text helpers
// ---------------------------------------------------------------------------

function splitTopLevel(input, sep = ',') {
  const out = [];
  let depth = 0;
  let cur = '';
  let i = 0;

  while (i < input.length) {
    const c = input[i];

    if (c === '"' || c === "'") {
      const quote = c;
      cur += c;
      i += 1;
      while (i < input.length) {
        const d = input[i];
        cur += d;
        if (d === '\\') {
          i += 1;
          if (i < input.length) {
            cur += input[i];
            i += 1;
          }
          continue;
        }
        if (d === quote) {
          i += 1;
          break;
        }
        i += 1;
      }
      continue;
    }

    if ('([{<'.includes(c)) {
      depth += 1;
      cur += c;
      i += 1;
      continue;
    }

    if (')]}>'.includes(c)) {
      if (c === '>' && input[i - 1] === '-') {
        cur += c;
        i += 1;
        continue;
      }
      depth = Math.max(0, depth - 1);
      cur += c;
      i += 1;
      continue;
    }

    if (c === sep && depth === 0) {
      out.push(cur.trim());
      cur = '';
      i += 1;
      continue;
    }

    cur += c;
    i += 1;
  }

  if (cur.trim() !== '') {
    out.push(cur.trim());
  }

  return out;
}

// Finds the `:` that separates a parameter's pattern from its type. A plain
// `indexOf(':')` would stop on the `:` in a destructuring pattern such as
// `TypedMultipart(CreateUploadRequest { file_type: _, .. })`.
function findTypeColon(param) {
  let depth = 0;
  let i = 0;

  while (i < param.length) {
    const c = param[i];
    if (c === '"' || c === "'") {
      const quote = c;
      i += 1;
      while (i < param.length) {
        if (param[i] === '\\') {
          i += 2;
          continue;
        }
        if (param[i] === quote) {
          i += 1;
          break;
        }
        i += 1;
      }
      continue;
    }
    if ('([{<'.includes(c)) depth += 1;
    else if (')]}>'.includes(c)) depth -= 1;
    else if (c === ':' && depth === 0 && param[i + 1] !== ':') return i;
    i += 1;
  }

  return -1;
}

function findMatching(text, openIdx) {
  const open = text[openIdx];
  const close = open === '(' ? ')' : open === '[' ? ']' : '}';
  let depth = 0;
  let i = openIdx;

  while (i < text.length) {
    const c = text[i];

    if (c === '"' || c === "'") {
      const quote = c;
      i += 1;
      while (i < text.length) {
        if (text[i] === '\\') {
          i += 2;
          continue;
        }
        if (text[i] === quote) {
          i += 1;
          break;
        }
        i += 1;
      }
      continue;
    }

    if (c === open) {
      depth += 1;
    } else if (c === close) {
      depth -= 1;
      if (depth === 0) return i;
    }
    i += 1;
  }

  return -1;
}

function stripComments(input) {
  let out = '';
  let i = 0;

  while (i < input.length) {
    const c = input[i];

    if (c === '/' && input[i + 1] === '/') {
      i += 2;
      while (i < input.length && input[i] !== '\n') i += 1;
      continue;
    }

    if (c === '/' && input[i + 1] === '*') {
      i += 2;
      while (i < input.length && !(input[i] === '*' && input[i + 1] === '/')) i += 1;
      i += 2;
      out += ' ';
      continue;
    }

    if (c === 'r' && input[i + 1] === '#' && input[i + 2] === '"') {
      let hashes = 0;
      let j = i + 2;
      while (input[j] === '#') {
        hashes += 1;
        j += 1;
      }
      const closer = '"' + '#'.repeat(hashes);
      out += input.slice(i, j + 1);
      i = j + 1;
      const end = input.indexOf(closer, i);
      if (end < 0) {
        i = input.length;
      } else {
        out += input.slice(i, end + closer.length);
        i = end + closer.length;
      }
      continue;
    }

    if (c === '"' || c === "'") {
      const quote = c;
      out += c;
      i += 1;
      while (i < input.length) {
        const d = input[i];
        out += d;
        if (d === '\\') {
          i += 1;
          if (i < input.length) {
            out += input[i];
            i += 1;
          }
          continue;
        }
        if (d === quote) {
          i += 1;
          break;
        }
        i += 1;
      }
      continue;
    }

    out += c;
    i += 1;
  }

  return out;
}

function parseGeneric(type) {
  const t = type.trim();
  const m = t.match(/^([A-Za-z_][\w:]*)\s*</);
  if (!m) return null;

  const open = t.indexOf('<', m[0].length - 1);
  let depth = 0;
  let end = -1;

  for (let i = open; i < t.length; i += 1) {
    if (t[i] === '<') depth += 1;
    else if (t[i] === '>') {
      depth -= 1;
      if (depth === 0) {
        end = i;
        break;
      }
    }
  }

  if (end < 0) return null;

  return { name: m[1], args: splitTopLevel(t.slice(open + 1, end)) };
}

function unquote(value) {
  const t = (value || '').trim();
  const m = t.match(/^"((?:[^"\\]|\\.)*)"$/);
  return m ? m[1] : t;
}

function nakedType(type) {
  return type.trim().replace(/\s+/g, ' ');
}

function baseName(type) {
  return type.trim().split('::').pop().replace(/[?&].*/, '');
}

function toCamel(name) {
  return name.replace(/_([a-z0-9])/g, (_, c) => c.toUpperCase());
}

function toSnake(name) {
  return name.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toLowerCase();
}

function toKebab(name) {
  return toSnake(name).replace(/_/g, '-');
}

function transformCase(name, style) {
  switch (style) {
    case 'camelCase':
      return toCamel(name);
    case 'snake_case':
      return toSnake(name);
    case 'kebab-case':
      return toKebab(name);
    case 'PascalCase':
      return name.charAt(0).toUpperCase() + name.slice(1);
    default:
      return name;
  }
}

function humanize(name) {
  const words = name.replace(/_/g, ' ');
  return words.charAt(0).toUpperCase() + words.slice(1);
}

function capitalize(name) {
  return name.charAt(0).toUpperCase() + name.slice(1);
}

function normalizePath(path) {
  let p = path.replace(/\/{2,}/g, '/');
  if (!p.startsWith('/')) p = '/' + p;
  if (p.length > 1 && p.endsWith('/')) p = p.slice(0, -1);
  return p;
}

function joinPath(prefix, path) {
  if (!path || path === '/') return normalizePath(prefix || '/');
  if (!prefix) return normalizePath(path);
  return normalizePath(prefix + path);
}

// ---------------------------------------------------------------------------
// Type registry
// ---------------------------------------------------------------------------

const registry = new Map(); // key -> entry
const entryByFile = [];

function register(entry) {
  const existing = registry.get(entry.key);
  if (existing) return existing;
  registry.set(entry.key, entry);
  return entry;
}

function componentNameFor(key) {
  const m = key.match(/^models\.([a-z_]+)\.Model$/);
  if (m) {
    const parts = m[1].split('_');
    return parts.map(capitalize).join('') + 'Model';
  }
  return key;
}

function isPrimitive(type) {
  return /^(String|bool|i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize|f32|f64)$/.test(
    type.trim()
  );
}

function normalizeTypeKey(type, fileInfo) {
  const t = type.trim();
  if (isPrimitive(t)) return null;

  if (fileInfo.isModel && t === 'Model') return `models.${fileInfo.mod}.Model`;
  if (fileInfo.aliases && fileInfo.aliases[t]) return fileInfo.aliases[t];

  let m = t.match(/(?:crate::)?models::([a-z_]+)::Model$/);
  if (m) return `models.${m[1]}.Model`;

  m = t.match(/^([a-z_]+)::Model$/);
  if (m) return `models.${m[1]}.Model`;

  return t.split('::').pop();
}

function stripRef(type) {
  return type.trim().replace(/^&\s*(?:'[A-Za-z_]\w*\s*)?/, '');
}

function typeRefs(type, fileInfo) {
  const t = stripRef(type);
  if (!t) return [];

  if (t.startsWith('(') && t.endsWith(')')) {
    return splitTopLevel(t.slice(1, -1)).flatMap((x) => typeRefs(x, fileInfo));
  }

  const generic = parseGeneric(t);
  if (generic) {
    return generic.args.flatMap((x) => typeRefs(x, fileInfo));
  }

  if (isPrimitive(t)) return [];
  if (/^(serde_json::)?Value$/.test(t)) return [];
  if (/^(DateTime|DateTimeWithTimeZone|NaiveDateTime|FieldData|NamedTempFile)$/.test(t)) return [];

  const key = normalizeTypeKey(t, fileInfo);
  return key ? [key] : [];
}

// ---------------------------------------------------------------------------
// Rust struct / enum parsing
// ---------------------------------------------------------------------------

function precedingBlock(text, index) {
  const before = text.slice(0, index);
  const cut = Math.max(before.lastIndexOf('\n\n'), before.lastIndexOf(';\n'), before.lastIndexOf('}\n'));
  return before.slice(cut + 1);
}

function parseDoc(block) {
  const lines = block.split('\n');
  const doc = [];
  for (const line of lines) {
    const m = line.match(/^\s*\/\/\/\s?(.*)$/);
    if (m) doc.push(m[1].trim());
  }
  return doc.join(' ').trim();
}

function parseTypes(text, fileInfo) {
  const headerRe = /\bpub\s+(struct|enum)\s+([A-Za-z0-9_]+)/g;
  let match;

  while ((match = headerRe.exec(text)) !== null) {
    const kind = match[1];
    const name = match[2];
    const headerEnd = match.index + match[0].length;
    const braceIdx = text.indexOf('{', headerEnd);
    if (braceIdx < 0) continue;
    const closeIdx = findMatching(text, braceIdx);
    if (closeIdx < 0) continue;

    const body = text.slice(braceIdx + 1, closeIdx);
    const block = precedingBlock(text, match.index);
    const renameAll = /rename_all\s*=\s*"([^"]+)"/.exec(block);
    const untagged = /serde\([^)]*untagged/.test(block);
    const description = parseDoc(block);

    // A `Model` struct in `src/models` namespaces its component by module so the
    // dozens of entity projections do not all collide on one name.
    let key = name;
    let isModel = false;
    if (kind === 'struct' && name === 'Model' && fileInfo.isModel) {
      key = `models.${fileInfo.mod}.Model`;
      isModel = true;
    }

    if (kind === 'struct') {
      const entry = register({
        key,
        kind: 'struct',
        name: componentNameFor(key),
        fileInfo,
        description,
        renameAll: renameAll ? renameAll[1] : null,
        fields: parseStructFields(body, renameAll ? renameAll[1] : null),
        isModel,
      });
      entryByFile.push(entry);
    } else {
      const entry = register({
        key,
        kind: 'enum',
        name: componentNameFor(key),
        fileInfo,
        description,
        renameAll: renameAll ? renameAll[1] : null,
        untagged,
        variants: parseEnumVariants(body),
      });
      entryByFile.push(entry);
    }
  }
}

function parseStructFields(body, renameAll) {
  const fields = [];
  const re =
    /(?:^|\n)((?:[ \t]*\/\/\/[^\n]*\n|[ \t]*#\[[^\]]*\]\s*\n?)*)[ \t]*pub\s+([a-z_][a-z0-9_]*)\s*:\s*([^,]+?)\s*,/g;
  let match;

  while ((match = re.exec(body)) !== null) {
    const attrs = match[1] || '';
    const rawName = match[2];
    const rawType = nakedType(match[3]);

    if (/serde\([^)]*skip\b/.test(attrs)) continue;

    const rename = /serde\([^)]*rename\s*=\s*"([^"]+)"/.exec(attrs);
    const tsType = /ts\([^)]*type\s*=\s*"((?:[^"\\]|\\.)*)"/.exec(attrs);
    const doc = parseDoc(attrs);
    const optional = /^\s*Option\s*</.test(rawType) || /serde\([^)]*default/.test(attrs);

    fields.push({
      name: rename ? rename[1] : renameAll ? transformCase(rawName, renameAll) : rawName,
      rawName,
      type: rawType,
      optional,
      tsType: tsType ? tsType[1].replace(/\\"/g, '"') : null,
      description: doc,
    });
  }

  return fields;
}

function parseEnumVariants(body) {
  const chunks = splitTopLevel(body, ',');
  const variants = [];

  for (const chunk of chunks) {
    if (!chunk.trim()) continue;
    const nameMatch = chunk.match(/([A-Za-z_][A-Za-z0-9_]*)\s*(?:\(([^)]*)\))?\s*$/);
    if (!nameMatch) continue;

    const name = nameMatch[1];
    const payload = nameMatch[2] ? nameMatch[2].trim() : null;
    const stringValue = /sea_orm\([^)]*string_value\s*=\s*"([^"]+)"/.exec(chunk);
    const serdeRename = /serde\([^)]*rename\s*=\s*"([^"]+)"/.exec(chunk);
    const skip = /ts\([^)]*skip/.test(chunk);

    variants.push({
      name,
      payload,
      stringValue: stringValue ? stringValue[1] : null,
      rename: serdeRename ? serdeRename[1] : null,
      skip,
    });
  }

  return variants;
}

// ---------------------------------------------------------------------------
// Route parsing
// ---------------------------------------------------------------------------

function parseImports(text, fileInfo) {
  const handlers = {};
  const aliasRe = /use\s+crate::models::([a-z_]+)::Model\s+as\s+([A-Za-z_]\w*)\s*;/g;
  let m;
  while ((m = aliasRe.exec(text)) !== null) {
    fileInfo.aliases[m[2]] = `models.${m[1]}.Model`;
  }

  const bracedRe = /use\s+crate::handlers::([a-z_]+)::\{([\s\S]*?)\};/g;
  while ((m = bracedRe.exec(text)) !== null) {
    const mod = m[1];
    for (const raw of m[2].split(',')) {
      const parts = raw.trim().split(/\s+as\s+/);
      const name = parts[0].trim();
      if (!name) continue;
      handlers[parts[1] ? parts[1].trim() : name] = mod;
    }
  }

  const singleRe = /use\s+crate::handlers::([a-z_]+)::([a-z_][A-Za-z0-9_]*);/g;
  while ((m = singleRe.exec(text)) !== null) {
    handlers[m[2]] = m[1];
  }

  return handlers;
}

function parseChain(expr) {
  const s = expr.trim();
  const boundaries = [];
  let depth = 0;
  let i = 0;

  while (i < s.length) {
    const c = s[i];

    if (c === '"' || c === "'") {
      const quote = c;
      i += 1;
      while (i < s.length) {
        if (s[i] === '\\') {
          i += 2;
          continue;
        }
        if (s[i] === quote) {
          i += 1;
          break;
        }
        i += 1;
      }
      continue;
    }

    if ('([{<'.includes(c)) depth += 1;
    else if (')]}>'.includes(c)) {
      if (!(c === '>' && s[i - 1] === '-')) depth -= 1;
    } else if (c === '.' && depth === 0) {
      boundaries.push(i);
    }
    i += 1;
  }

  if (boundaries.length === 0) {
    return { base: s, calls: [] };
  }

  const base = s.slice(0, boundaries[0]).trim();
  const calls = [];

  for (let k = 0; k < boundaries.length; k += 1) {
    const start = boundaries[k];
    const end = k + 1 < boundaries.length ? boundaries[k + 1] : s.length;
    calls.push(parseMethodCall(s.slice(start, end)));
  }

  return { base, calls };
}

function parseMethodCall(segment) {
  let seg = segment.trim();
  if (seg.startsWith('.')) seg = seg.slice(1);
  const p = seg.indexOf('(');
  if (p < 0) return { name: seg.trim(), args: [] };
  const name = seg.slice(0, p).trim();
  const close = findMatching(seg, p);
  const argsStr = close < 0 ? seg.slice(p + 1) : seg.slice(p + 1, close);
  return { name, args: splitTopLevel(argsStr) };
}

function parseMethods(methodsExpr) {
  const out = [];
  const re = /(get|post|put|patch|delete|head|options)\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)/g;
  let m;
  while ((m = re.exec(methodsExpr)) !== null) {
    out.push({ method: m[1], handler: m[2] });
  }
  return out;
}

function evaluate(expr, env, visited) {
  const stack = visited || new Set();
  const { base, calls } = parseChain(expr);
  let endpoints = [];
  const baseId = base.trim();

  if (/^[A-Za-z_]\w*$/.test(baseId) && env.lets[baseId] !== undefined && !stack.has(baseId)) {
    stack.add(baseId);
    endpoints = evaluate(env.lets[baseId], env, stack);
  }

  for (const call of calls) {
    switch (call.name) {
      case 'route': {
        const path = unquote(call.args[0] || '');
        const methods = parseMethods(call.args[1] || '');
        for (const method of methods) {
          endpoints.push({
            method: method.method,
            path,
            handler: method.handler,
            module: env.handlers[method.handler] || env.defaultModule,
            auth: null,
          });
        }
        break;
      }
      case 'merge': {
        const sub = evaluate(call.args[0] || '', env, new Set(stack));
        endpoints = endpoints.concat(sub);
        break;
      }
      case 'nest': {
        const prefix = unquote(call.args[0] || '');
        const sub = evaluate(call.args[1] || '', env, new Set(stack));
        for (const ep of sub) {
          endpoints.push({ ...ep, path: joinPath(prefix, ep.path) });
        }
        break;
      }
      case 'layer':
      case 'route_layer': {
        const arg = call.args[0] || '';
        const optional = /optional_authenticate/.test(arg);
        const required = !optional && /\bauthenticate\b/.test(arg);
        if (optional) {
          for (const ep of endpoints) if (!ep.auth) ep.auth = 'optional';
        } else if (required) {
          for (const ep of endpoints) ep.auth = 'required';
        }
        break;
      }
      default:
        break;
    }
  }

  return endpoints;
}

function parseRouteFile(file) {
  const raw = readFileSync(file, 'utf8');
  const text = stripComments(raw);
  const base = file.split('/').pop().replace(/\.rs$/, '');
  const fileInfo = { mod: base, isModel: false, aliases: {} };
  const handlers = parseImports(text, fileInfo);
  const aliasMap = parseImports(text, { mod: base, isModel: false, aliases: {} });

  const fnRe = /fn\s+[A-Za-z_]\w*\s*\([^)]*\)\s*->\s*Router\s*\{/;
  const fnMatch = fnRe.exec(text);
  if (!fnMatch) return [];

  const bodyStart = text.indexOf('{', fnMatch.index);
  const bodyEnd = findMatching(text, bodyStart);
  const body = text.slice(bodyStart + 1, bodyEnd);

  const statements = splitTopLevel(body, ';');
  const lets = {};
  let returnExpr = '';

  for (let i = 0; i < statements.length; i += 1) {
    const stmt = statements[i].trim();
    if (!stmt) continue;
    if (stmt.startsWith('let ')) {
      const m = stmt.match(/^let\s+(?:mut\s+)?([A-Za-z_]\w*)\s*=\s*([\s\S]+)$/);
      if (m) lets[m[1]] = m[2];
    } else if (i === statements.length - 1 || /Router::new|\.nest\(|\.merge\(/.test(stmt)) {
      returnExpr = stmt;
    }
  }

  const env = { lets, handlers: aliasMap, defaultModule: base };
  return evaluate(returnExpr, env, new Set());
}

// ---------------------------------------------------------------------------
// Handler parsing
// ---------------------------------------------------------------------------

const handlerInfo = {};
const handlerAliases = {};

function signatureFor(text, name) {
  const re = new RegExp(`pub\\s+async\\s+fn\\s+${name}\\s*\\(`);
  const m = re.exec(text);
  if (!m) return null;

  const openParen = text.indexOf('(', m.index);
  const closeParen = findMatching(text, openParen);
  const params = text.slice(openParen + 1, closeParen);

  const afterParams = text.slice(closeParen + 1);
  const arrow = afterParams.indexOf('->');
  let returnType = '';
  let braceIdx = afterParams.indexOf('{');

  if (arrow >= 0 && (braceIdx < 0 || arrow < braceIdx)) {
    returnType = afterParams.slice(arrow + 2, braceIdx >= 0 ? braceIdx : undefined).trim();
  }

  const bodyOpen = text.indexOf('{', closeParen);
  const bodyClose = bodyOpen >= 0 ? findMatching(text, bodyOpen) : -1;
  const body = bodyOpen >= 0 && bodyClose >= 0 ? text.slice(bodyOpen + 1, bodyClose) : '';

  let bodyType = null;
  let bodyKind = null;
  let queryType = null;
  let hasClaims = false;
  let hasOptionalClaims = false;
  let hasMultipart = false;

  for (const param of splitTopLevel(params)) {
    if (!param.trim()) continue;
    const colon = findTypeColon(param);
    const type = colon >= 0 ? nakedType(param.slice(colon + 1)) : '';

    const simple = /^([A-Za-z_]\w*)$/.exec(type);
    if (simple && simple[1] === 'Multipart') hasMultipart = true;

    const generic = parseGeneric(type);
    if (generic) {
      const inner = generic.args[0];
      if (generic.name === 'ValidatedRequest' || generic.name === 'Json' || generic.name === 'AuthenticatedRequest') {
        bodyType = inner;
        bodyKind = 'json';
      } else if (generic.name === 'TypedMultipart') {
        bodyType = inner;
        bodyKind = 'multipart';
      } else if (generic.name === 'Query') {
        queryType = inner;
      }
    }

    if (/\bOptionalClaims\b/.test(type)) hasOptionalClaims = true;
    else if (/\bClaims\b/.test(type)) hasClaims = true;
  }

  return {
    returnType,
    bodyType,
    bodyKind: hasMultipart && !bodyType ? 'multipart-raw' : bodyKind,
    queryType,
    hasClaims,
    hasOptionalClaims,
    hasMultipart,
    body,
  };
}

function parseHandlerFile(file) {
  const raw = readFileSync(file, 'utf8');
  const text = stripComments(raw);
  const mod = file.split('/').pop().replace(/\.rs$/, '');
  handlerInfo[mod] = {};
  handlerAliases[mod] = {};

  const aliasRe = /use\s+crate::models::([a-z_]+)::Model\s+as\s+([A-Za-z_]\w*)\s*;/g;
  let aliasMatch;
  while ((aliasMatch = aliasRe.exec(text)) !== null) {
    handlerAliases[mod][aliasMatch[2]] = `models.${aliasMatch[1]}.Model`;
  }

  const fnRe = /pub\s+async\s+fn\s+([A-Za-z_]\w*)\s*\(/g;
  let m;
  while ((m = fnRe.exec(text)) !== null) {
    const info = signatureFor(text, m[1]);
    if (info) handlerInfo[mod][m[1]] = info;
  }
}

// ---------------------------------------------------------------------------
// Schema generation
// ---------------------------------------------------------------------------

function refFor(key) {
  return { $ref: `#/components/schemas/${componentNameFor(key)}` };
}

function isNullable(schema) {
  if (!schema) return false;
  if (Array.isArray(schema.type) && schema.type.includes('null')) return true;
  return (schema.anyOf || schema.oneOf || []).some((option) => option.type === 'null');
}

function nullable(schema) {
  if (!schema || Object.keys(schema).length === 0) return schema;
  if (isNullable(schema)) return schema;
  if (schema.$ref || schema.allOf || schema.anyOf || schema.oneOf) {
    return { anyOf: [schema, { type: 'null' }] };
  }
  if (schema.type && typeof schema.type === 'string') {
    return { ...schema, type: [schema.type, 'null'] };
  }
  return { anyOf: [schema, { type: 'null' }] };
}

function enumValues(entry) {
  if (entry.untagged) return null;
  return entry.variants.map((v) => {
    if (v.rename) return v.rename;
    return transformCase(v.name, entry.renameAll);
  });
}

function schemaForType(type, fileInfo) {
  const t = stripRef(nakedType(type));
  if (!t) return {};
  if (t === '()') return {};

  if (t.startsWith('(') && t.endsWith(')')) {
    const items = splitTopLevel(t.slice(1, -1)).map((x) => schemaForType(x, fileInfo));
    return {
      type: 'array',
      prefixItems: items,
      minItems: items.length,
      maxItems: items.length,
    };
  }

  const generic = parseGeneric(t);
  if (generic) {
    switch (generic.name) {
      case 'Option':
        return nullable(schemaForType(generic.args[0], fileInfo));
      case 'Vec':
        return { type: 'array', items: schemaForType(generic.args[0], fileInfo) };
      case 'Box':
        return schemaForType(generic.args[0], fileInfo);
      case 'FieldData':
        return { type: 'string', format: 'binary' };
      case 'Paginated':
        return {
          type: 'object',
          properties: {
            items: { type: 'array', items: schemaForType(generic.args[0], fileInfo) },
            page: { type: 'integer' },
            pageSize: { type: 'integer' },
            total: { type: 'integer' },
            totalPages: { type: 'integer' },
          },
          required: ['items', 'page', 'pageSize', 'total', 'totalPages'],
        };
      default:
        break;
    }
  }

  if (/^(String|&str|str)$/.test(t)) return { type: 'string' };
  if (/^bool$/.test(t)) return { type: 'boolean' };
  if (/^(i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize)$/.test(t)) {
    return { type: 'integer' };
  }
  if (/^(f32|f64)$/.test(t)) return { type: 'number' };
  if (/^(DateTime|DateTimeWithTimeZone|NaiveDateTime)$/.test(t)) {
    return { type: 'string', format: 'date-time' };
  }
  if (/^(NamedTempFile|FieldData)$/.test(t)) return { type: 'string', format: 'binary' };
  if (/^(serde_json::)?Value$/.test(t)) return {};
  if (/^(HeaderMap|Multipart|Request|RequestParts|String)$/.test(t)) return {};

  const key = normalizeTypeKey(t, fileInfo);
  if (key && registry.has(key)) return refFor(key);

  return {};
}

function schemaForField(field, fileInfo) {
  if (field.tsType !== null) {
    const ts = field.tsType.trim();
    if (ts === 'string') return { type: 'string' };
    if (ts === 'File') return { type: 'string', format: 'binary' };
    if (ts.startsWith('"')) {
      const values = ts
        .split('|')
        .map((part) => unquote(part.trim()))
        .filter((v) => v !== ts);
      if (values.length) return { type: 'string', enum: values };
    }
  }

  const schema = schemaForType(field.type, fileInfo);
  if (field.description) {
    return { ...schema, description: field.description };
  }
  return schema;
}

function componentSchema(entry) {
  if (entry.kind === 'struct') {
    const properties = {};
    const required = [];
    for (const field of entry.fields) {
      properties[field.name] = schemaForField(field, entry.fileInfo);
      if (!field.optional) required.push(field.name);
    }
    const schema = { type: 'object', properties };
    if (required.length) schema.required = required;
    if (entry.description) schema.description = entry.description;
    return schema;
  }

  // enum
  if (entry.untagged) {
    const options = [];
    const seen = new Set();
    for (const variant of entry.variants) {
      if (variant.skip) continue;
      let option;
      if (variant.payload) {
        option = schemaForType(variant.payload, entry.fileInfo);
      } else {
        option = { type: 'null' };
      }
      const sig = JSON.stringify(option);
      if (seen.has(sig)) continue;
      seen.add(sig);
      options.push(option);
    }
    const schema = { oneOf: options };
    if (entry.description) schema.description = entry.description;
    return schema;
  }

  const values = enumValues(entry);
  const schema = { type: 'string', enum: values };
  if (entry.description) schema.description = entry.description;
  return schema;
}

// ---------------------------------------------------------------------------
// Response resolution
// ---------------------------------------------------------------------------

function unwrapResult(type) {
  const t = type.trim();
  const generic = parseGeneric(t);
  if (generic && generic.name === 'Result') return generic.args[0];
  return t;
}

function schemaForApiResponse(inner, fileInfo) {
  const dataSchema = schemaForType(inner, fileInfo);
  const messageOnly = { type: 'object', properties: { message: { type: 'string' } } };

  if (!dataSchema || Object.keys(dataSchema).length === 0 || inner.trim() === '()') {
    return messageOnly;
  }

  return { allOf: [dataSchema, messageOnly] };
}

function resolveSuccess(returnType, fileInfo, body) {
  const inner = unwrapResult(returnType);
  // A handler that returns a tuple carries the status at runtime; the only
  // signal the type gives us is a bare `StatusCode`, so read the literal the
  // handler actually constructs.
  const status = /StatusCode::CREATED/.test(body || '') ? '201' : '200';

  if (inner.trim() === '()' || inner.trim() === 'StatusCode') return { status, schema: null };

  if (inner.startsWith('(')) {
    const parts = splitTopLevel(inner.slice(1, -1));
    const schemaPart = parts.find((p) => /^(Json|ApiResponse)</.test(p.trim()));
    if (!schemaPart) return { status, schema: null };
    return { status, schema: schemaPart.trim(), isApiResponse: /^ApiResponse</.test(schemaPart.trim()) };
  }


  return { status, schema: inner, isApiResponse: /^ApiResponse</.test(inner) };
}

function responseSchemaFrom(inner, fileInfo, body) {
  if (!inner) return null;

  const api = parseGeneric(inner);
  if (api && api.name === 'ApiResponse') {
    return schemaForApiResponse(api.args[0], fileInfo);
  }

  const json = parseGeneric(inner);
  if (json && json.name === 'Json') {
    return responseSchemaForValue(json.args[0], fileInfo, body);
  }

  const html = parseGeneric(inner);
  if (html && html.name === 'Html') {
    return { __html: true, schema: { type: 'string' } };
  }

  return responseSchemaForValue(inner, fileInfo, body);
}

function responseSchemaForValue(type, fileInfo, body) {
  const t = stripRef(type);

  const parsed = parseGeneric(t);
  if (parsed && parsed.name === 'Json') {
    return responseSchemaForValue(parsed.args[0], fileInfo, body);
  }

  if (/^(serde_json::)?Value$/.test(t)) {
    if (/"count"/.test(body || '')) return { $ref: '#/components/schemas/CountResponse' };
    if (/"message"/.test(body || '')) return { $ref: '#/components/schemas/MessageResponse' };
    return { type: 'object', description: 'Free-form JSON response' };
  }

  return schemaForType(t, fileInfo);
}

// ---------------------------------------------------------------------------
// OpenAPI assembly
// ---------------------------------------------------------------------------

function readCargoVersion() {
  const cargo = readFileSync(join(ROOT, 'Cargo.toml'), 'utf8');
  const m = cargo.match(/^\s*version\s*=\s*"([^"]+)"/m);
  return m ? m[1] : '0.0.0';
}

function loadSources() {
  const dirs = [
    ['src/extractors', false],
    ['src/dto', false],
    ['src/models', false],
    ['src/handlers', false],
    ['src/payment', false],
  ];

  for (const [rel, isModel] of dirs) {
    const dir = join(ROOT, rel);
    if (!existsSync(dir)) continue;
    for (const name of readdirSync(dir)) {
      if (!name.endsWith('.rs')) continue;
      if (name === 'mod.rs') continue;
      const file = join(dir, name);
      const raw = readFileSync(file, 'utf8');
      const text = stripComments(raw);
      const mod = name.replace(/\.rs$/, '');
      const fileInfo = { mod, isModel: rel === 'src/models', aliases: {} };

      const aliasRe = /use\s+crate::models::([a-z_]+)::Model\s+as\s+([A-Za-z_]\w*)\s*;/g;
      let am;
      while ((am = aliasRe.exec(text)) !== null) {
        fileInfo.aliases[am[2]] = `models.${am[1]}.Model`;
      }

      parseTypes(text, fileInfo);
    }
  }

  const handlerDir = join(ROOT, 'src/handlers');
  if (existsSync(handlerDir)) {
    for (const name of readdirSync(handlerDir)) {
      if (name.endsWith('.rs') && name !== 'mod.rs') parseHandlerFile(join(handlerDir, name));
    }
  }
}

function relativeFiles(dir) {
  const full = join(ROOT, dir);
  if (!existsSync(full)) return [];
  return readdirSync(full)
    .filter((n) => n.endsWith('.rs') && n !== 'mod.rs' && n !== 'router.rs')
    .map((n) => join(full, n));
}

function generate() {
  loadSources();

  const endpoints = [];
  for (const file of relativeFiles('src/routes')) {
    for (const ep of parseRouteFile(file)) {
      const tag = TAGS[file.split('/').pop().replace(/\.rs$/, '')] || 'General';
      endpoints.push({ ...ep, tag });
    }
  }

  const paths = {};
  const roots = new Set();
  const operationIds = new Map();

  for (const ep of endpoints) {
    if (IGNORED_PATHS.has(ep.path)) continue;

    const mod = ep.module;
    const info = (handlerInfo[mod] && handlerInfo[mod][ep.handler]) || null;
    if (!info) continue;

    let auth = ep.auth;
    if (info.hasClaims) auth = 'required';
    else if (info.hasOptionalClaims && auth !== 'required') auth = 'optional';

    const fileInfo = { mod, isModel: false, aliases: handlerAliases[mod] || {} };
    const success = resolveSuccess(info.returnType, fileInfo, info.body);
    const responseSchema = responseSchemaFrom(success.schema, fileInfo, info.body);

    if (info.bodyType) {
      const refs = typeRefs(info.bodyType, fileInfo);
      for (const r of refs) roots.add(r);
    }
    if (info.queryType) {
      for (const r of typeRefs(info.queryType, fileInfo)) roots.add(r);
    }
    if (success.schema) {
      for (const r of typeRefs(success.schema, fileInfo)) roots.add(r);
    }
    if (responseSchema && responseSchema.$ref) {
      const m = responseSchema.$ref.match(/schemas\/(.+)$/);
      if (m) roots.add(m[1]);
    }

    let operationId = `${ep.handler}_${mod}`;
    let candidate = operationId;
    let counter = 2;
    while (operationIds.has(candidate)) {
      candidate = `${operationId}_${counter}`;
      counter += 1;
    }
    operationIds.set(candidate, true);

    const operation = {
      tags: [ep.tag],
      summary: humanize(ep.handler),
      operationId: candidate,
    };

    if (auth === 'required') operation.security = [{ bearerAuth: [] }];
    else if (auth === 'optional') operation.description = 'Authentication is optional on this endpoint.';

    const path = normalizePath(ep.path);
    const pathParams = [...path.matchAll(/\{([^}]+)\}/g)].map((m) => m[1]);
    const parameters = pathParams.map((name) => ({
      name,
      in: 'path',
      required: true,
      schema: { type: 'string' },
    }));

    if (info.queryType) {
      const queryEntry = registry.get(normalizeTypeKey(info.queryType, fileInfo));
      if (queryEntry && queryEntry.kind === 'struct') {
        for (const field of queryEntry.fields) {
          parameters.push({
            name: field.name,
            in: 'query',
            required: false,
            schema: schemaForField(field, queryEntry.fileInfo),
          });
        }
      }
    }

    if (parameters.length) operation.parameters = parameters;

    if (info.bodyKind === 'json' && info.bodyType) {
      operation.requestBody = {
        required: true,
        content: { 'application/json': { schema: schemaForType(info.bodyType, fileInfo) } },
      };
    } else if (info.bodyKind === 'multipart' && info.bodyType) {
      operation.requestBody = {
        required: true,
        content: { 'multipart/form-data': { schema: schemaForType(info.bodyType, fileInfo) } },
      };
    } else if (info.bodyKind === 'multipart-raw') {
      operation.requestBody = {
        required: true,
        content: {
          'multipart/form-data': {
            schema: {
              type: 'object',
              properties: { file: { type: 'string', format: 'binary' } },
              required: ['file'],
            },
          },
        },
      };
    }

    const responses = {};
    if (responseSchema === null) {
      responses[success.status] = { description: 'Success', content: {} };
    } else if (responseSchema.__html) {
      responses[success.status] = {
        description: 'Success',
        content: { 'text/html': { schema: responseSchema.schema } },
      };
    } else {
      responses[success.status] = {
        description: 'Success',
        content: { 'application/json': { schema: responseSchema } },
      };
    }

    responses['400'] = {
      description: 'Bad request',
      content: { 'application/json': { schema: { $ref: '#/components/schemas/MessageResponse' } } },
    };
    if (auth === 'required') {
      responses['401'] = {
        description: 'Unauthorized',
        content: { 'application/json': { schema: { $ref: '#/components/schemas/MessageResponse' } } },
      };
    }
    responses['500'] = {
      description: 'Internal server error',
      content: { 'application/json': { schema: { $ref: '#/components/schemas/MessageResponse' } } },
    };

    operation.responses = responses;

    if (!paths[path]) paths[path] = {};
    paths[path][ep.method] = operation;
  }

  // Reachability: only emit components reachable from a request or response.
  const reachable = new Set(['MessageResponse', 'CountResponse']);
  const queue = [...roots];
  while (queue.length) {
    const key = queue.shift();
    if (!key || reachable.has(key)) continue;
    if (!registry.has(key)) continue;
    reachable.add(key);
    const entry = registry.get(key);
    if (entry.kind === 'struct') {
      for (const field of entry.fields) {
        for (const r of typeRefs(field.type, entry.fileInfo)) queue.push(r);
      }
    } else if (entry.kind === 'enum' && entry.untagged) {
      for (const variant of entry.variants) {
        if (variant.payload) {
          for (const r of typeRefs(variant.payload, entry.fileInfo)) queue.push(r);
        }
      }
    }
  }

  const schemas = {
    MessageResponse: { type: 'object', properties: { message: { type: 'string' } } },
    CountResponse: { type: 'object', properties: { count: { type: 'integer' } }, required: ['count'] },
  };

  for (const entry of entryByFile) {
    if (!reachable.has(entry.key)) continue;
    schemas[entry.name] = componentSchema(entry);
  }

  const sortedSchemas = {};
  for (const name of Object.keys(schemas).sort()) sortedSchemas[name] = schemas[name];

  const sortedPaths = {};
  for (const path of Object.keys(paths).sort()) {
    const byMethod = {};
    for (const method of HTTP_METHODS) {
      if (paths[path][method]) byMethod[method] = paths[path][method];
    }
    sortedPaths[path] = byMethod;
  }

  const tagOrder = [];
  for (const ep of endpoints) if (!tagOrder.includes(ep.tag)) tagOrder.push(ep.tag);

  const document = {
    openapi: '3.1.0',
    info: {
      title: 'asterisk API',
      version: readCargoVersion(),
      description: 'Generated from the Axum router and handler signatures. Do not edit by hand.',
    },
    tags: tagOrder.map((name) => ({ name })),
    paths: sortedPaths,
    components: {
      securitySchemes: {
        bearerAuth: { type: 'http', scheme: 'bearer', bearerFormat: 'JWT' },
      },
      schemas: sortedSchemas,
    },
  };

  return JSON.stringify(document, null, 2) + '\n';
}

function main() {
  const check = process.argv.includes('--check');
  const output = generate();

  if (check) {
    const current = existsSync(OUT) ? readFileSync(OUT, 'utf8') : '';
    if (current !== output) {
      console.error('docs/openapi.json is out of date. Run: node scripts/openapi/generate.mjs');
      process.exit(1);
    }
    console.log('docs/openapi.json is up to date.');
    return;
  }

  mkdirSync(dirname(OUT), { recursive: true });
  writeFileSync(OUT, output);
  const endpointCount = (output.match(/"operationId"/g) || []).length;
  console.log(`Wrote docs/openapi.json (${endpointCount} operations).`);
}

main();
