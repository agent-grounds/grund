// §FS-distribution.3.2.2.4: closed host inputs copied before dispatch.
'use strict';
const path = require('node:path');
const { GrundError } = require('./errors.cjs');
const roots = { root: 'string' };
const schemas = {
  check: { requireGrounding:'boolean', suggestions:'boolean', full:'boolean', rule:'nullable',
    only:'strings', ignore:'strings', onlyRule:'boolean' },
  show: { ...roots, section:'nullable', mode:['lead','brief','toc','full'], format:['text','md','json'] },
  showBatch: { ...roots, mode:['lead','brief','toc','full'] },
  refs: { ...roots, section:'nullable', descendants:'boolean' },
  list: { ...roots, kinds:'strings', projects:'strings', unused:'boolean', selector:'nullable' },
  listSizes: { ...roots, kinds:'strings', projects:'strings', unused:'boolean', selector:'nullable',
    units:['array','lines','words','bytes'], top:'positiveNullable' },
  cover: roots, fmt: { ...roots, write:'boolean', marker:'boolean', crossRefs:'boolean' },
  proposeId: { ...roots, width:'integer' },
  init: { name:'nullable', description:'nullable', docs:'boolean', force:'boolean', dryRun:'boolean',
    check:'boolean', noVcs:'boolean', agents:['arrayNullable','canonical','claude','gemini',
      'pi','copilot','cursor','windsurf','zed'] },
  completeIds: { ...roots, prefix:'string', sections:'boolean' },
  effectiveConfig: roots, validateConfig: roots, fetch: roots,
  integrations: { client:['nullable','codium','iterm2','kitty','tmux','vscode','wezterm'],
    write:'boolean', conversation:['nullable','plain','link'],
    conversationTarget:['nullable','file','path','web','vscode','vscodium','cursor'],
    agent:['nullable','codex','claude','gemini','copilot','zed','pi'] },
};
function bad(op, message, code='invalid-argument') {
  throw new GrundError({operation:op,kind:'input',code,message});
}
function string(op,v) {
  if (typeof v !== 'string' || !v.isWellFormed() || v.includes('\0')) bad(op,'expected a Unicode string without NUL');
  return v;
}
function plain(op,v) {
  if (v === null || typeof v !== 'object' || ![Object.prototype,null].includes(Object.getPrototypeOf(v)))
    bad(op,'expected a plain options object');
  for (const key of Reflect.ownKeys(v)) {
    if (typeof key !== 'string' || !Object.hasOwn(Object.getOwnPropertyDescriptor(v,key),'value'))
      bad(op,'options must contain string keys and data properties');
  }
}
function field(op,value,shape) {
  if (shape === 'nullable' && value === null) return null;
  if (shape === 'string' || shape === 'nullable') return string(op,value);
  if (shape === 'boolean') { if (typeof value !== 'boolean') bad(op,'expected boolean'); return value; }
  if (shape === 'strings') {
    if (!Array.isArray(value)) bad(op,'expected string array');
    return Array.from(value,v=>string(op,v));
  }
  if (shape === 'integer' || shape === 'positiveNullable') {
    if (shape === 'positiveNullable' && value === null) return null;
    if (!Number.isSafeInteger(value) || value < (shape === 'integer' ? 0 : 1)) bad(op,'expected safe integer in range');
    return value;
  }
  const [kind,...allowed] = shape;
  if (value === null && (kind === 'nullable' || kind === 'arrayNullable')) return null;
  if (kind === 'array' || kind === 'arrayNullable') {
    if (!Array.isArray(value) || !value.length) bad(op,'expected nonempty array');
    const copied=Array.from(value,v=>string(op,v));
    if (copied.some(v=>!allowed.includes(v)) || new Set(copied).size !== copied.length)
      bad(op,'invalid or duplicate array value');
    return copied;
  }
  string(op,value);
  if (!shape.includes(value)) bad(op,'invalid enum value');
  return value;
}
function options(op,v) {
  if (v === undefined) return {};
  plain(op,v);
  const schema=schemas[op], out={};
  for (const key of Reflect.ownKeys(v)) {
    if (!Object.hasOwn(schema,key)) bad(op,'unknown option '+key,'unknown-option');
    // Explicit undefined has the same default semantics as omission.
    if (v[key] !== undefined) out[key]=field(op,v[key],schema[key]);
  }
  return out;
}
exports.request = function request(op,args) {
  const cwd=process.cwd();
  let opts={}, operands=[], root, explicit=false;
  const optionalRoot = op==='check' || op==='init';
  if (op==='agentSetupInstructions') {
    if (args.length) bad(op,'no operands accepted');
  } else if (op==='scan' || op==='referenceStyle') {
    if (args.length !== 1) bad(op,'one path is required');
    root=string(op,args[0]); explicit=true;
  } else if (optionalRoot) {
    if (args.length>2) bad(op,'too many operands');
    if (args[0] !== undefined) {root=string(op,args[0]);explicit=true;}
    opts=options(op,args[1]);
  } else if (['show','showBatch','refs','fetch','proposeId'].includes(op)) {
    const n=op==='proposeId'?2:1;
    if (args.length<n || args.length>n+1) bad(op,'invalid operand count');
    if (op==='showBatch') {
      const queries=args[0];
      if (queries===null) operands=[null];
      else {
        if (!Array.isArray(queries)) bad(op,'queries must be an array or null');
        operands=[Array.from(queries,q=>{
          if (typeof q==='string') return string(op,q);
          plain(op,q);
          if (Reflect.ownKeys(q).some(k=>!['id','section'].includes(k))) bad(op,'unknown query key');
          return {id:string(op,q.id),section:q.section==null?null:string(op,q.section)};
        })];
      }
    } else operands=args.slice(0,n).map(v=>string(op,v));
    opts=options(op,args[n]);
  } else {
    if (args.length>1) bad(op,'too many operands');
    opts=options(op,args[0]);
  }
  if (opts.onlyRule && !opts.rule) bad(op,'onlyRule requires rule','conflicting-options');
  if (op==='integrations') {
    if ((!opts.write && (opts.conversation!=null || opts.conversationTarget!=null || opts.agent!=null)) ||
        (opts.agent!=null && opts.conversationTarget==null) ||
        (opts.write && opts.client==null && opts.conversation==null && opts.conversationTarget==null))
      bad(op,'conflicting integration preferences','conflicting-options');
    operands=[opts.client??''];
  }
  if (op==='completeIds') operands=[opts.prefix??''];
  if (Object.hasOwn(opts,'root')) {root=opts.root;explicit=true;delete opts.root;}
  root=path.resolve(cwd,root??'.');
  const names={requireGrounding:'require_grounding',onlyRule:'only_rule',crossRefs:'cross_refs',
    dryRun:'dry_run',noVcs:'no_vcs',conversationTarget:'conversation_target'};
  opts=Object.fromEntries(Object.entries(opts).map(([k,v])=>[names[k]??k,v]));
  return {operation:op,root,explicit,args:operands,options:opts};
};
