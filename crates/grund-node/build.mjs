// §FS-distribution.3.2.3.3: explicit pinned, locked, unwind-capable source build.
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import {pathToFileURL} from 'node:url';
export function flags(args) {
  const out={};
  for (let i=0;i<args.length;i++) {
    const key=args[i];
    if (key==='--test-seams') out.testSeams=true;
    else {
      if (!['--source-root','--out-dir','--target-dir','--target','--profile'].includes(key) ||
          !args[i+1] || args[i+1].startsWith('--')) throw Error('invalid build option '+key);
      if (Object.hasOwn(out,key.slice(2))) throw Error('duplicate build option '+key);
      out[key.slice(2)]=args[++i];
    }
  }
  return out;
}
export function command(program,args,cwd,env=process.env) {
  const result=spawnSync(program,args,{cwd,env,stdio:'inherit'});
  if (result.error) throw result.error;
  if (result.status!==0) throw Error(program+' failed with status '+result.status);
}
export function build(options) {
  for (const name of ['source-root','out-dir','target-dir'])
    if (!options[name]) throw Error('--'+name+' is required');
  const root=path.resolve(options['source-root']),out=path.resolve(options['out-dir']);
  const targetDir=path.resolve(options['target-dir']);
  if (targetDir===root || targetDir.startsWith(root+path.sep)) throw Error('target-dir must be external to sources');
  const profile=options.profile??'release';
  if (!['release','dev'].includes(profile)) throw Error('profile must be release or dev');
  const compilerFlags=[process.env.RUSTFLAGS,process.env.CARGO_ENCODED_RUSTFLAGS];
  const panicProfile=process.env[profile==='dev'?'CARGO_PROFILE_DEV_PANIC':'CARGO_PROFILE_RELEASE_PANIC'];
  if (panicProfile?.trim()==='abort' ||
      compilerFlags.some(flags => /panic\s*=\s*abort/.test(flags??'')))
    throw Error('grund-node requires panic=unwind');
  const probe=spawnSync('rustc',['+1.95.0','-vV'],{encoding:'utf8',cwd:root});
  if (probe.status!==0 || !probe.stdout.includes('release: 1.95.0')) throw Error('Rust 1.95.0 toolchain required');
  const target=options.target??probe.stdout.match(/^host: (.+)$/m)?.[1];
  if (!target) throw Error('unable to determine Rust target');
  const args=['+1.95.0','build','--manifest-path',path.join(root,'Cargo.toml'),
    '-p','grund-node','--locked','--target',target,'--target-dir',targetDir];
  if (profile==='release') args.push('--release');
  if (options.testSeams) args.push('--features','grund-node/test-node-runtime');
  command('cargo',args,root);
  const library=target.includes('windows')?'grund_node.dll':
    target.includes('apple')?'libgrund_node.dylib':'libgrund_node.so';
  const manifest=fs.readFileSync(path.join(root,'Cargo.toml'),'utf8');
  const version=manifest.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  if (!version) throw Error('workspace package version missing');
  fs.mkdirSync(out,{recursive:true});
  fs.copyFileSync(path.join(targetDir,target,profile==='dev'?'debug':'release',library),
    path.join(out,'grund.node'));
  const metadata={apiSchemaVersion:1,engineVersion:version,packageVersion:version,target,napiVersion:8};
  const provenance=path.join(root,'source-metadata.json');
  if (fs.existsSync(provenance)) metadata.sourceSha=JSON.parse(fs.readFileSync(provenance,'utf8')).sourceSha;
  fs.writeFileSync(path.join(out,'metadata.json'),JSON.stringify(metadata,null,2)+'\n');
}
if (process.argv[1] && import.meta.url===pathToFileURL(path.resolve(process.argv[1])).href) {
  try {build(flags(process.argv.slice(2)));}
  catch(error) {process.stderr.write(error.message+'\n');process.exitCode=1;}
}
