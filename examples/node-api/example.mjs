// §FS-distribution.3.2.4: captured real package consumer, using the json-report fixture.
import {check,show} from 'grund-cli';
import {deepEqual} from 'node:assert/strict';
import {resolve} from 'node:path';
const root=process.argv[2];
const report=await check(root);
const body=await show('FS-001-alpha',{root,mode:'brief'});
console.log('dangling: '+report.report.errors.some(f=>f.code==='dangling'));
console.log('show: '+body.id);

// §FS-errors.4.1: the pathless caution precedes the located warning.
const warningRoot=process.argv[3] ?? resolve(root,'../../cross-project-citation-without-workspace/repo');
const warningCodes=(await check(warningRoot)).report.warnings.map(f=>f.code);
deepEqual(warningCodes,['config-version-1','unused']);
console.log('warnings: '+warningCodes.join(', '));
