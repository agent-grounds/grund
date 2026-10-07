// §FS-distribution.3.2.3.3: disposable API rehearsal; #471 owns assembly.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import {flags} from './build.mjs';
const here=path.dirname(fileURLToPath(import.meta.url)),repo=path.resolve(here,'../..');
try {
  const options=flags(process.argv.slice(2));
  if (!options['out-dir'] || !options['target-dir']) throw Error('--out-dir and --target-dir required');
  const out=path.resolve(options['out-dir']),target=path.resolve(options['target-dir']);
  if (out===repo || out.startsWith(repo+path.sep)) throw Error('stage outside the checkout');
  if (fs.existsSync(out) && fs.readdirSync(out).length) throw Error('staging destination must be empty');
  fs.mkdirSync(out,{recursive:true});
  const fragment=JSON.parse(fs.readFileSync(path.join(here,'package-api.json'),'utf8'));
  const workspace=fs.readFileSync(path.join(repo,'Cargo.toml'),'utf8');
  const version=workspace.match(/^version\s*=\s*"([^"]+)"/m)[1];
  fs.writeFileSync(path.join(out,'package.json'),JSON.stringify({
    name:'grund-cli',version,description:'API-only local rehearsal of grund-cli; no CLI launcher',
    license:'MIT',...fragment},null,2)+'\n');
  for (const file of fs.readdirSync(path.join(here,'js')))
    fs.copyFileSync(path.join(here,'js',file),path.join(out,file));
  for (const file of ['README.md','LICENSE'])
    fs.copyFileSync(file==='LICENSE'?path.join(repo,file):path.join(here,file),path.join(out,file));
  const sources=path.join(out,'build','sources');
  fs.mkdirSync(sources,{recursive:true});
  fs.writeFileSync(path.join(sources,'Cargo.toml'),workspace
    .replace(/^members = .*$/m,'members = ["crates/grund-core", "crates/grund-node"]')
    .replace(/^default-members = .*$/m,'default-members = ["crates/grund-node"]'));
  fs.copyFileSync(path.join(repo,'Cargo.lock'),path.join(sources,'Cargo.lock'));
  fs.writeFileSync(path.join(sources,'rust-toolchain.toml'),
    '[toolchain]\nchannel = "1.95.0"\nprofile = "minimal"\n');
  for (const crate of ['grund-core','grund-node']) {
    const dest=path.join(sources,'crates',crate),src=path.join(repo,'crates',crate);
    fs.mkdirSync(dest,{recursive:true});
    for (const item of ['Cargo.toml','README.md','src',...(crate==='grund-core'?['assets','examples']:['build.rs','build.mjs'])])
      fs.cpSync(path.join(src,item),path.join(dest,item),{recursive:true});
  }
  const sha=spawnSync('git',['rev-parse','HEAD'],{cwd:repo,encoding:'utf8'});
  if (sha.status!==0) throw Error('source SHA unavailable');
  fs.writeFileSync(path.join(sources,'source-metadata.json'),JSON.stringify({sourceSha:sha.stdout.trim(),engineVersion:version})+'\n');
  fs.copyFileSync(path.join(here,'source.mjs'),path.join(out,'build','source.mjs'));
  // Lock the adjusted closure once at staging; every actual build is --locked.
  const lock=spawnSync('cargo',['+1.95.0','metadata','--format-version','1',
    '--manifest-path',path.join(sources,'Cargo.toml')],{cwd:sources,
    env:{...process.env,CARGO_TARGET_DIR:target},stdio:['ignore','ignore','inherit']});
  if (lock.status!==0) throw Error('unable to lock the adjusted source closure');
  fs.writeFileSync(path.join(out,'build','rehearsal.json'),JSON.stringify({apiOnly:true,targetDir:target})+'\n');
} catch(error) {process.stderr.write(error.message+'\n');process.exitCode=1;}
