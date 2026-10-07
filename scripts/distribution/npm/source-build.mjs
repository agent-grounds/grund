// §FS-distribution-candidate.4.1: `npm run build:source` builds this package's own
// commands from its bundled, locked sources into `native/`, and only when asked.
// §FS-distribution-candidate.4.3: a missing prerequisite is one `error:` line naming it.
// In `grund-cli` the addon is built first, by the binding's own builder, unchanged.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(here, '..');
const sources = path.join(here, 'sources');
const manifest = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8'));
const command = Object.keys(manifest.bin)[0];
const crate = command === 'grund' ? 'grund' : 'grund-lsp';
const toolchain = '1.95.0';

function fail(message) {
  process.stderr.write(`error: ${message}\n`);
  process.exit(1);
}

function targetDir(args) {
  const at = args.indexOf('--target-dir');
  if (at >= 0 && args[at + 1]) return path.resolve(args[at + 1]);
  return process.env.CARGO_TARGET_DIR || path.join(os.tmpdir(), `${manifest.name}-source-target`);
}

function run(program, args, options = {}) {
  const result = spawnSync(program, args, {stdio: 'inherit', ...options});
  if (result.error) fail(`${program} could not start: ${result.error.message}`);
  if (result.status !== 0) fail(`${program} ${args[0]} failed with status ${result.status}`);
}

const cargo = spawnSync('cargo', ['--version'], {encoding: 'utf8'});
if (cargo.error || cargo.status !== 0)
  fail(`cargo was not found: a source build of ${manifest.name} needs Rust ${toolchain} and ` +
    'Cargo (https://rustup.rs), a linker and the platform SDK');
const rustc = spawnSync('rustc', [`+${toolchain}`, '-vV'], {encoding: 'utf8', cwd: sources});
const host = rustc.status === 0 ? rustc.stdout.match(/^host: (.+)$/m)?.[1] : undefined;
if (!host) fail(`the Rust ${toolchain} toolchain was not found: rustup toolchain install ${toolchain}`);

const target = targetDir(process.argv.slice(2));
const native = path.join(root, 'native');
if (crate === 'grund')
  run(process.execPath, [path.join(here, 'addon-source.mjs'), '--target-dir', target]);
run('cargo', [`+${toolchain}`, 'build', '--manifest-path', path.join(sources, 'Cargo.toml'),
  '-p', crate, '--release', '--locked', '--target', host, '--target-dir', target], {cwd: sources});
const binary = command + (host.includes('windows') ? '.exe' : '');
fs.mkdirSync(native, {recursive: true});
fs.copyFileSync(path.join(target, host, 'release', binary), path.join(native, binary));
if (crate === 'grund-lsp') {
  const recorded = JSON.parse(fs.readFileSync(path.join(sources, 'source-metadata.json'), 'utf8'));
  fs.writeFileSync(path.join(native, 'metadata.json'), JSON.stringify({
    engineVersion: recorded.engineVersion, packageVersion: manifest.version, target: host,
    sourceSha: recorded.sourceSha}, null, 2) + '\n');
}
