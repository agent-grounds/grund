// §FS-distribution.3.2.4: captured real package consumer, using the json-report fixture.
import {check,show} from 'grund-cli';
const root=process.argv[2];
const report=await check(root);
const body=await show('FS-001-alpha',{root,mode:'brief'});
console.log('dangling: '+report.report.errors.some(f=>f.code==='dangling'));
console.log('show: '+body.id);
