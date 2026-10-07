// §FS-distribution.3.2.2.1: one ESM/CJS error identity and complete failure fields.
'use strict';
class GrundError extends Error {
  constructor(failure, cause) {
    super(failure.message, cause === undefined ? undefined : { cause });
    this.name='GrundError';
    this.failure={path:null,line:null,column:null,sites:[],authority:[],causes:[],
      details:{},partial:null,runCautions:[],...failure};
  }
}
exports.GrundError=GrundError;
