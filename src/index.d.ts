export interface Fingerprint {
  readonly value: string;
  readonly digest: string;
  readonly language: 'javascript' | 'typescript';
  readonly formatVersion: number;
  readonly normalizationVersion: number;
}

/** Initialize the Rust WebAssembly module and Tree-sitter grammars. */
export function init(): Promise<void>;
export function fingerprintJavaScript(source: string): Fingerprint;
export function fingerprintTypeScript(source: string): Fingerprint;
export function fingerprintSource(source: string, language: 'javascript' | 'typescript'): Fingerprint;
export function compareFingerprints(left: Fingerprint, right: Fingerprint): boolean;
