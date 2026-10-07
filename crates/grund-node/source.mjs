// §FS-distribution.3.2.3.3: installed fallback uses only its bundled source closure.
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import os from 'node:os';
import {build,flags} from './sources/crates/grund-node/build.mjs';
const here=path.dirname(fileURLToPath(import.meta.url));
try {
  const options=flags(process.argv.slice(2));
  build({...options,'source-root':path.join(here,'sources'),
    'out-dir':path.join(here,'..','native'),
    'target-dir':options['target-dir']??path.join(os.homedir(),'ag','tmp','grund-node-source-target')});
} catch(error) {process.stderr.write(error.message+'\n');process.exitCode=1;}
