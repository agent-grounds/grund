// §FS-distribution.3.2.2: fixed transport names; authored schema/details are opaque.
'use strict';
const fields=[
  'runCautions','selectedReport','hadScanErrors','outputFormat','kindTitle','expectedExit',
  'scanErrors','enclosingDeclaration','enclosingSection','sectionSeparator','valueRoots',
  'refusedWrites','e2eCaseDir','fileHoldsSingleDeclaration','pendingChanges','scanReadsFile',
  'fsHome','headingName','headingMarker','configFile','manualSteps','agentOverrides',
  'conversationTarget','installKind','configTarget','resolverTarget','escapedCitations',
  'scannedFiles','walkedDirs','fileStructures','valueBindings','invalidValueDeclarations',
  'invalidValueBindings','sectionHeadingsOutsideDeclarations','unmarkedHeadings','nearMissHeadings',
  'headingLevel','duplicateSections','e2eCase','bodyStart','bodyEnd','bodyHasContent','valueValid',
  'localSection','shorthandRewritable','numericRun','inlineSite','memberSlice','keyColumn','keyText',
  'sourceKind','suggestedPath','sourceSlice','valueRoot','bindingNamespace','bindingSection',
  'startLine','endLine','isDocComment','isDocstring','lineCount','canonicalRelativePath',
  'firstLine','lastLine','maxColumns','hasNote','layoutViolations','docComments','totalLines',
];
const names=Object.fromEntries(fields.map(k=>[k.replace(/[A-Z]/g,c=>'_'+c.toLowerCase()),k]));
exports.convert=function convert(v,opaque=false) {
  if (Array.isArray(v)) return v.map(x=>convert(x,opaque));
  if (v===null || typeof v!=='object') return v;
  return Object.fromEntries(Object.entries(v).map(([k,x])=>[
    opaque?k:names[k]??k,convert(x,opaque||k==='values'||k==='details')]));
};
