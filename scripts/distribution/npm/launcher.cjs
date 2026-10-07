#!/usr/bin/env node
// §FS-distribution-candidate.3.2: start this row's payload and add nothing to it.
// §FS-distribution-candidate.3.3: a missing payload is one `error:` line and exit 2.
'use strict';
const fs = require('node:fs');
const path = require('node:path');
const {spawn} = require('node:child_process');
const {constants} = require('node:os');

const here = __dirname;
const manifest = JSON.parse(fs.readFileSync(path.join(here, 'package.json'), 'utf8'));
const command = Object.keys(manifest.bin)[0];
const binary = command + (process.platform === 'win32' ? '.exe' : '');
const FORWARDED = ['SIGINT', 'SIGTERM', 'SIGHUP'];

function refuse(problem) {
  process.stderr.write(`error: ${command} ${manifest.version} on ${process.platform} ` +
    `${process.arch}: ${problem}; install the matching platform package, or run ` +
    `\`npm run build:source\` in ${here}\n`);
  process.exit(2);
}

// The same host-to-target rule as the binding's loader (§FS-distribution.3.2.3.2).
function target() {
  const cpu = {x64: 'x86_64', arm64: 'aarch64'}[process.arch];
  if (process.platform === 'linux' && cpu) {
    process.report.excludeNetwork = true;
    return process.report.getReport().header.glibcVersionRuntime ? `${cpu}-unknown-linux-gnu` : null;
  }
  if (process.platform === 'darwin' && cpu) return `${cpu}-apple-darwin`;
  if (process.platform === 'win32' && cpu === 'x86_64') return `${cpu}-pc-windows-msvc`;
  return null;
}

function sourceBuild(rust) {
  const metadataFile = path.join(here, 'native', 'metadata.json');
  const built = path.join(here, 'native', binary);
  if (!fs.existsSync(built)) return null;
  let metadata;
  try { metadata = JSON.parse(fs.readFileSync(metadataFile, 'utf8')); } catch { metadata = null; }
  if (metadata?.packageVersion !== manifest.version || metadata?.target !== rust)
    refuse(`the source build in ${path.join(here, 'native')} is not ${manifest.version} for ${rust}`);
  return built;
}

function platformPackage(rust) {
  const name = manifest.grund.platformPackages[rust];
  let found;
  try {
    found = require.resolve(`${name}/package.json`, {paths: [here]});
  } catch {
    refuse(`${name}@${manifest.version} is not installed`);
  }
  const version = JSON.parse(fs.readFileSync(found, 'utf8')).version;
  if (version !== manifest.version)
    refuse(`${name} is ${version}, not ${manifest.version}`);
  const executable = require(path.dirname(found)).executablePath;
  if (typeof executable !== 'string' || !fs.existsSync(executable))
    refuse(`${name}@${version} holds no ${binary}`);
  return executable;
}

const rust = target();
if (!rust || !manifest.grund.platformPackages[rust])
  refuse('no supported row of the support matrix matches this host');
const payload = sourceBuild(rust) ?? platformPackage(rust);

const child = spawn(payload, process.argv.slice(2), {stdio: 'inherit'});
const forward = (signal) => child.kill(signal);
for (const signal of FORWARDED) process.on(signal, forward);
child.on('error', (error) => refuse(`${payload} could not start (${error.code})`));
child.on('exit', (code, signal) => {
  if (signal === null) process.exit(code);
  // Die of the payload's signal. Node ignores SIGPIPE; a listener removed restores the default.
  for (const name of FORWARDED) process.removeListener(name, forward);
  const restore = () => {};
  try {
    process.on(signal, restore);
    process.removeListener(signal, restore);
  } catch {
    // SIGKILL and SIGSTOP take no listener, and keep their default anyway.
  }
  process.kill(process.pid, signal);
  process.exit(128 + (constants.signals[signal] ?? 0));
});
