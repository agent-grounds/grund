// §FS-distribution.3.2.3.1 — pending native work itself keeps Node alive.
const api = require('grund-cli');
const native = require('./node_modules/grund-cli/native/grund.node');
native.__test.delayNext('check', 50); // native worker delay; no JS timer or open port
api.check(process.argv[2]).then(() => process.stdout.write('finished\n'));
