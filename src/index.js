import { Language, Parser } from 'web-tree-sitter';
import initWasm, {
  compareFingerprints,
  fingerprintSyntaxNodes,
} from '../pkg/structaura_wasm.js';

const TREE_SITTER_RUNTIME_URL = new URL('../pkg/grammars/tree-sitter.wasm', import.meta.url);
const JAVASCRIPT_GRAMMAR_URL = new URL('../pkg/grammars/tree-sitter-javascript.wasm', import.meta.url);
const TYPESCRIPT_GRAMMAR_URL = new URL('../pkg/grammars/tree-sitter-typescript.wasm', import.meta.url);
const assetPath = (url) => import.meta.url.startsWith('file:')
  ? decodeURIComponent(url.pathname)
  : url.href;
let initializedRuntime;
let runtimePromise;

async function initializeRustWasm() {
  // wasm-bindgen's web target defaults to fetch(), which cannot load file://
  // URLs in Node. Read the generated binary directly for the Node test runtime;
  // browsers continue using the normal URL/fetch initialization path.
  if (typeof process !== 'undefined' && process.versions?.node) {
    const fs = process.getBuiltinModule?.('node:fs');
    if (!fs) {
      throw new Error('Node.js must expose process.getBuiltinModule() to load StructAura WASM in tests');
    }
    const wasmPath = new URL('../pkg/structaura_wasm_bg.wasm', import.meta.url);
    await initWasm(fs.readFileSync(wasmPath));
    return;
  }

  await initWasm();
}

function initializeRuntime() {
  return (async () => {
    await initializeRustWasm();
    await Parser.init({
      locateFile: (fileName) => {
        if (fileName !== 'web-tree-sitter.wasm' && fileName !== 'tree-sitter.wasm') {
          throw new Error(`unexpected Tree-sitter runtime asset: ${fileName}`);
        }
        return assetPath(TREE_SITTER_RUNTIME_URL);
      },
    });
    const [javascript, typescript] = await Promise.all([
      Language.load(assetPath(JAVASCRIPT_GRAMMAR_URL)),
      Language.load(assetPath(TYPESCRIPT_GRAMMAR_URL)),
    ]);
    return { javascript, typescript };
  })();
}

/** Initialize Rust WASM and both Tree-sitter grammar modules. Safe to call repeatedly. */
export async function init() {
  if (!runtimePromise) {
    runtimePromise = initializeRuntime().then((runtime) => {
      initializedRuntime = runtime;
      return runtime;
    });
  }
  await runtimePromise;
}

export function fingerprintJavaScript(source) {
  return fingerprint(source, 'javascript');
}

export function fingerprintTypeScript(source) {
  return fingerprint(source, 'typescript');
}

export function fingerprintSource(source, language) {
  if (language !== 'javascript' && language !== 'typescript') {
    throw new TypeError(`unsupported language '${language}'; expected 'javascript' or 'typescript'`);
  }
  return fingerprint(source, language);
}

function fingerprint(source, language) {
  if (typeof source !== 'string') {
    throw new TypeError('source must be a string');
  }
  const grammars = runtimeReady();
  const parser = new Parser();
  let tree;
  try {
    parser.setLanguage(grammars[language]);
    tree = parser.parse(source);
    if (!tree) throw new Error('Tree-sitter parsing was cancelled');
    const root = tree.rootNode;
    if (root.hasError) {
      throw new SyntaxError(formatSyntaxDiagnostics(root));
    }
    const records = flattenSyntaxTree(root, source);
    return fingerprintSyntaxNodes(records, language);
  } finally {
    tree?.delete();
    parser.delete();
  }
}

// Parsing is intentionally synchronous after init; this makes worker offloading explicit.
function runtimeReady() {
  if (!initializedRuntime) {
    throw new Error('StructAura is not initialized. Call and await init() before fingerprinting.');
  }
  return initializedRuntime;
}

function flattenSyntaxTree(root, source) {
  const records = [];
  const pending = [root];

  while (pending.length > 0) {
    const node = pending.pop();
    const children = [];
    for (let index = 0; index < node.childCount; index += 1) {
      const child = node.child(index);
      if (child) children.push(child);
    }

    records.push({
      syntax_kind: node.type,
      text: node.type === 'string' || node.childCount === 0
        ? (node.text ?? source.slice(node.startIndex, node.endIndex))
        : '',
      is_named: node.isNamed,
      child_count: children.length,
    });
    for (let index = children.length - 1; index >= 0; index -= 1) {
      pending.push(children[index]);
    }
  }

  return records;
}

function formatSyntaxDiagnostics(root) {
  const diagnostics = [];
  const pending = [root];
  while (pending.length > 0) {
    const node = pending.pop();
    if (node.isError || node.isMissing) {
      diagnostics.push(`${node.isMissing ? `expected ${node.type}` : 'unexpected syntax'} at bytes ${node.startIndex}..${node.endIndex}`);
      continue;
    }
    for (let index = node.childCount - 1; index >= 0; index -= 1) {
      const child = node.child(index);
      if (child && (child.hasError || child.isError || child.isMissing)) pending.push(child);
    }
  }
  return `${diagnostics.length} syntax diagnostic(s): ${diagnostics.join('; ') || 'unknown parser error'}`;
}

export { compareFingerprints };
