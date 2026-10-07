// §FS-distribution.3.2.3.2: lazy, local-first exact native payload selection.
'use strict';
const fs=require('node:fs');
const path=require('node:path');
const {GrundError}=require('./errors.cjs');
let loaded;
function target() {
  const cpu={x64:'x86_64',arm64:'aarch64'}[process.arch];
  if (process.platform==='linux' && cpu && process.report.getReport().header.glibcVersionRuntime)
    return cpu+'-unknown-linux-gnu';
  if (process.platform==='darwin' && cpu) return cpu+'-apple-darwin';
  if (process.platform==='win32' && cpu==='x86_64') return cpu+'-pc-windows-msvc';
  throw Error('unsupported addon host');
}
function compatible(actual,expected) {
  for (const k of ['apiSchemaVersion','engineVersion','packageVersion','target','napiVersion'])
    if (actual?.[k] !== expected[k]) throw Error('incompatible native metadata: '+k);
}
exports.load=function load(operation) {
  if (loaded) return loaded;
  const attempted=[];
  let expected;
  try {
    if (![22,24].includes(Number(process.versions.node.split('.')[0]))) throw Error('Node 22 or 24 required');
    const base=__dirname;
    const manifest=JSON.parse(fs.readFileSync(path.join(base,'package.json'),'utf8'));
    expected={apiSchemaVersion:1,engineVersion:manifest.version,packageVersion:manifest.version,
      target:target(),napiVersion:8};
    let addon=path.join(base,'native','grund.node'), metadata;
    const metadataFile=path.join(base,'native','metadata.json');
    if (fs.existsSync(addon) || fs.existsSync(metadataFile)) {
      attempted.push(addon,metadataFile);
      metadata=JSON.parse(fs.readFileSync(metadataFile,'utf8'));
    } else {
      const name=manifest.grund?.platformPackages?.[expected.target] ??
        '@agent-grounds/grund-cli-'+expected.target;
      attempted.push(name);
      const descriptor=require(name);
      compatible(descriptor.metadata,expected);
      if (require(name+'/package.json').version !== manifest.version) throw Error('platform package version mismatch');
      addon=descriptor.addonPath;
      if (typeof addon!=='string' || !path.isAbsolute(addon)) throw Error('absolute addonPath required');
      metadata=descriptor.metadata;
      attempted.push(addon);
    }
    compatible(metadata,expected);
    const native=require(addon);
    compatible(native.metadata,expected);
    if (typeof native.invoke!=='function') throw Error('native invoke export missing');
    loaded=native;
    return loaded;
  } catch (cause) {
    throw new GrundError({operation,kind:'load',code:'native-load',message:'cannot load grund native addon',
      causes:[String(cause.message)],details:{expected:expected??null,attempted}},cause);
  }
};
