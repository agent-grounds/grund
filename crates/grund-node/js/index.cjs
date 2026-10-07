// §FS-distribution.3.2.1: all operations validate and snapshot before first await.
'use strict';
const {GrundError}=require('./errors.cjs');
const {request}=require('./validation.cjs');
const {load}=require('./loader.cjs');
const {convert}=require('./records.cjs');
exports.GrundError=GrundError;
for (const operation of ['check','scan','show','showBatch','refs','list','listSizes','cover',
  'fmt','proposeId','init','completeIds','effectiveConfig','validateConfig','fetch',
  'integrations','referenceStyle','agentSetupInstructions']) {
  exports[operation]=async function(...args) {
    const owned=request(operation,args);
    const native=load(operation);
    let raw;
    try {raw=await native.invoke(JSON.stringify(owned));}
    catch (cause) {
      if (cause instanceof GrundError) throw cause;
      let nativeFailure;
      try {nativeFailure=convert(JSON.parse(cause.message));} catch {}
      if (nativeFailure?.failure)
        throw new GrundError({...nativeFailure.failure,operation,runCautions:nativeFailure.runCautions},cause);
      throw new GrundError({operation,kind:'worker',code:'worker-failed',
        message:String(cause.message),causes:[String(cause)]},cause);
    }
    const envelope=convert(JSON.parse(raw));
    if (envelope.failure) throw new GrundError({...envelope.failure,runCautions:envelope.runCautions});
    return {...envelope.result,runCautions:envelope.runCautions};
  };
}
