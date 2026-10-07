// §FS-distribution.3.2.2: deliberate readonly public records.
export type JsonValue = null | boolean | number | string | readonly JsonValue[] |
  { readonly [key: string]: JsonValue };
export type Site = { readonly path: string; readonly line: number };
export type Finding = {
  readonly code: string; readonly message: string; readonly path: string | null;
  readonly line: number | null; readonly column: number | null;
  readonly sites: readonly Site[]; readonly authority: readonly string[];
} & ({ readonly severity: 'error' | 'warning'; readonly channel: null } |
     { readonly channel: 'suggestion'; readonly severity: null });
export type Report = { readonly errors: readonly Finding[]; readonly warnings: readonly Finding[];
  readonly suggestions: readonly Finding[] };
export type Cautions = { readonly runCautions: readonly Finding[] };
export type ScanError = { readonly path: string; readonly message: string };
export type FailureKind = 'input' | 'query' | 'config' | 'io' | 'operation' | 'load' |
  'worker' | 'native' | 'busy';
export type Failure = {
  readonly kind: FailureKind; readonly code: string; readonly operation: string;
  readonly message: string; readonly path: string | null; readonly line: number | null;
  readonly column: number | null; readonly sites: readonly Site[]; readonly authority: readonly string[];
  readonly causes: readonly string[]; readonly details: Readonly<Record<string, JsonValue>>;
  readonly partial: PartialOutcome | null; readonly runCautions: readonly Finding[];
};
export type PartialOutcome =
  | { readonly operation: 'scan'; readonly result: ScanSnapshot }
  | { readonly operation: 'refs'; readonly result: RefsData }
  | { readonly operation: 'fmt'; readonly result: FmtData }
  | { readonly operation: 'init'; readonly result: InitData }
  | { readonly operation: 'integrations'; readonly result: IntegrationInstallData };
export declare class GrundError extends Error { readonly failure: Failure }
export type RootOptions = { readonly root?: string };
export type CheckOptions = {
  readonly requireGrounding?: boolean; readonly suggestions?: boolean; readonly full?: boolean;
  readonly rule?: string | null; readonly only?: readonly string[]; readonly ignore?: readonly string[];
  readonly onlyRule?: boolean;
};
export type ShowMode = 'lead' | 'brief' | 'toc' | 'full';
export type ShowOptions = RootOptions & { readonly section?: string | null;
  readonly mode?: ShowMode; readonly format?: 'text' | 'md' | 'json' };
export type ShowQuery = { readonly id: string; readonly section?: string | null };
export type BatchOptions = RootOptions & { readonly mode?: ShowMode };
export type RefsOptions = RootOptions & { readonly section?: string | null; readonly descendants?: boolean };
export type ListOptions = RootOptions & { readonly kinds?: readonly string[]; readonly projects?: readonly string[];
  readonly unused?: boolean; readonly selector?: string | null };
export type SizeUnit = 'lines' | 'words' | 'bytes';
export type SizeOptions = ListOptions & { readonly units?: readonly SizeUnit[]; readonly top?: number | null };
export type FmtOptions = RootOptions & { readonly write?: boolean; readonly marker?: boolean; readonly crossRefs?: boolean };
export type IdOptions = RootOptions & { readonly width?: number };
export type InitAgent = 'canonical' | 'claude' | 'gemini' | 'pi' | 'copilot' | 'cursor' | 'windsurf' | 'zed';
export type InitOptions = { readonly name?: string | null; readonly description?: string | null;
  readonly docs?: boolean; readonly force?: boolean; readonly dryRun?: boolean; readonly check?: boolean;
  readonly noVcs?: boolean; readonly agents?: readonly InitAgent[] | null };
export type CompleteOptions = RootOptions & { readonly prefix?: string; readonly sections?: boolean };
export type Client = 'codium' | 'iterm2' | 'kitty' | 'tmux' | 'vscode' | 'wezterm';
export type ConversationTarget = 'file' | 'path' | 'web' | 'vscode' | 'vscodium' | 'cursor';
export type IntegrationAgent = 'codex' | 'claude' | 'gemini' | 'copilot' | 'zed' | 'pi';
export type IntegrationOptions = { readonly client?: Client | null; readonly write?: boolean;
  readonly conversation?: 'plain' | 'link' | null; readonly conversationTarget?: ConversationTarget | null;
  readonly agent?: IntegrationAgent | null };
export type CheckResult = Cautions & { readonly report: Report; readonly selectedReport: Report;
  readonly hadScanErrors: boolean; readonly outputFormat: string };
export type ShowSection = { readonly path: string; readonly title: string; readonly depth: number };
export type Manifest = { readonly kind: 'E2E'; readonly args: readonly string[]; readonly expectedExit: number;
  readonly fixtures: readonly string[] };
export type ShowData = { readonly id: string; readonly section: string | null; readonly kindTitle: string | null;
  readonly body: string; readonly path: string; readonly line: number; readonly sections: readonly ShowSection[];
  readonly json: string | null; readonly manifest: Manifest | null };
export type ShowResult = ShowData & Cautions;
export type BatchResult = Cautions & { readonly records: readonly (
  { readonly query: ShowQuery; readonly ok: true; readonly result: ShowData } |
  { readonly query: ShowQuery; readonly ok: false; readonly failure: Failure })[] };
export type RefHit = { readonly project: string | null; readonly path: string; readonly line: number;
  readonly column: number; readonly id: string; readonly section: string | null; readonly marker: boolean;
  readonly text: string; readonly enclosingDeclaration: string | null; readonly enclosingSection: string | null };
export type RefsData = { readonly outputFormat: string; readonly workspace: boolean; readonly kindTitle: string | null;
  readonly note: string | null; readonly hits: readonly RefHit[]; readonly scanErrors: readonly ScanError[];
  readonly summaries: readonly { readonly project: string | null; readonly path: string;
    readonly count: number; readonly lines: readonly number[] }[];
  readonly totals: { readonly sites: number; readonly files: number } };
export type RefsResult = RefsData & Cautions;
export type ListEntry = { readonly project: string | null; readonly id: string; readonly section: string | null;
  readonly sectionSeparator: string; readonly kind: string; readonly path: string; readonly line: number;
  readonly title: string | null; readonly stub: boolean; readonly defines: string | null; readonly refs: number;
  readonly duplicate: boolean; readonly valueRoots: readonly { readonly id: string; readonly valid: boolean }[] };
export type ListSummary = { readonly project: string | null; readonly kind: string; readonly title: string;
  readonly home: string; readonly count: number };
export type ListResult = Cautions & { readonly outputFormat: string; readonly workspace: boolean;
  readonly entries: readonly ListEntry[]; readonly summaries: readonly ListSummary[]; readonly scanErrors: readonly ScanError[] };
export type SizeEntry = Omit<ListEntry, 'title' | 'refs' | 'valueRoots'> & {
  readonly measurements: readonly { readonly unit: SizeUnit; readonly lead: number | null; readonly full: number | null }[] };
export type SizeResult = Cautions & { readonly outputFormat: string; readonly workspace: boolean;
  readonly entries: readonly SizeEntry[]; readonly scanErrors: readonly ScanError[] };
export type CoverResult = Cautions & { readonly outputFormat: string; readonly scanErrors: readonly ScanError[];
  readonly entries: readonly { readonly project: string | null; readonly path: string; readonly citations: readonly RefHit[] }[] };
export type FmtData = { readonly changes: readonly { readonly path: string; readonly line: number; readonly label: string }[];
  readonly scanErrors: readonly ScanError[]; readonly refusedWrites: readonly string[] };
export type FmtResult = FmtData & Cautions;
export type IdResult = Cautions & { readonly id: string; readonly kind: string; readonly number: number | null;
  readonly slug: string; readonly folder: string | null; readonly file: string | null; readonly e2eCaseDir: string | null;
  readonly fileHoldsSingleDeclaration: boolean };
export type Event = { readonly verb: string; readonly path: string };
export type InitData = { readonly events: readonly Event[]; readonly errors: readonly Finding[]; readonly notes: readonly string[];
  readonly next: { readonly docs: boolean; readonly entrypoint: string; readonly scanReadsFile: boolean;
    readonly fsHome: { readonly kind: 'file'; readonly path: string; readonly headingName: string; readonly headingMarker: string } |
      { readonly kind: 'folder'; readonly path: string } } | null; readonly pendingChanges: boolean };
export type InitResult = InitData & Cautions;
export type CompleteResult = Cautions & { readonly ids: readonly string[] };
export type FetchResult = Cautions & { readonly id: string };
export type StyleResult = Cautions & { readonly marker: string; readonly trigger: string };
export type SetupResult = Cautions & { readonly instructions: string };
export type ClientDescriptor = { readonly client: Client; readonly kind: 'terminal' | 'editor';
  readonly detected: boolean; readonly installed: boolean | null; readonly installKind: 'block' | 'extension' | 'manual';
  readonly configTarget: string; readonly resolverTarget: string | null };
export type IntegrationInstallData = { readonly mode: 'install'; readonly events: readonly Event[];
  readonly notes: readonly string[]; readonly manualSteps: readonly string[];
  readonly preferences: { readonly conversation: 'plain' | 'link'; readonly conversationTarget: ConversationTarget;
    readonly agentOverrides: Readonly<Partial<Record<IntegrationAgent, ConversationTarget>>> } };
export type IntegrationResult = Cautions & (
  { readonly mode: 'detection'; readonly detected: readonly Client[]; readonly clients: readonly ClientDescriptor[] } |
  { readonly mode: 'artifact'; readonly client: ClientDescriptor; readonly artifacts: readonly {
    readonly path: string | null; readonly content: string }[]; readonly manualSteps: readonly string[] } |
  IntegrationInstallData);
export type ConfigValues = {
  readonly version: number; readonly project_name: string | null; readonly project_description: string | null;
  readonly reference: { readonly marker: string; readonly trigger: string; readonly strict: boolean;
    readonly shorthand: string; readonly require_grounding: boolean; readonly grounding_level: number;
    readonly conversation: string | null; readonly lead_size_warning: { readonly max: number; readonly unit: SizeUnit } | null;
    readonly inline_style: string; readonly inline_note_suggested_lines: number; readonly inline_note_max_lines: number;
    readonly inline_note_max_columns: number; readonly inline_note_layout: string; readonly inline_note_layout_check: string;
    readonly warn_on_suggested: boolean };
  readonly id: { readonly format: string; readonly section_separator: string; readonly number_pattern: string;
    readonly slug_pattern: string; readonly named_sections: boolean; readonly section_heading_levels: string };
  readonly scan: { readonly include: readonly string[] | null; readonly exclude: readonly string[]; readonly extensions: readonly string[];
    readonly comment_prefixes: readonly string[]; readonly docstring_python: boolean;
    readonly respect_gitignore: boolean };
  readonly fmt: { readonly exclude: readonly string[]; readonly cross_refs: { readonly enabled: boolean; readonly anchor_format: string } };
  readonly output: { readonly format: string; readonly relative_paths: boolean; readonly color: string };
  readonly workspace: { readonly members: readonly string[]; readonly optional_members: readonly string[]; readonly include_root: boolean };
  readonly kinds: readonly { readonly kind: string; readonly folder: string | null; readonly file: string | null;
    readonly title: string | null; readonly index: string | boolean; readonly citable: boolean;
    readonly scan: boolean; readonly require_grounding: boolean | null; readonly grounding_level: number | null;
    readonly values: boolean; readonly value_chapter: string | null; readonly rules: boolean;
    readonly format: string | null; readonly resolve: string | null; readonly fetch: string | null }[];
  readonly citations: Readonly<Record<string, JsonValue>>;
};
export type ConfigResult = Cautions & { readonly root: string; readonly configFile: string | null; readonly values: ConfigValues };
export type DeclarationSource = { readonly kind: 'text' } |
  { readonly kind: 'json'; readonly memberSlice: string; readonly keyColumn: number; readonly keyText: string };
export type ValueComponent = { readonly decoded: string; readonly kind: 'number' | 'string'; readonly sourceSlice: string; readonly column: number };
export type Section = { readonly path: string; readonly title: string; readonly line: number; readonly headingLevel: number;
  readonly value: ValueComponent | null; readonly valueRoot: { readonly valid: boolean;
    readonly origin: { readonly kind: 'chapter' } | { readonly kind: 'marker'; readonly column: number } } | null };
export type E2eCase = { readonly dir: string; readonly args: readonly string[]; readonly expectedExit: number;
  readonly fixtures: readonly string[]; readonly spec_refs: readonly { readonly namespace: string | null; readonly kind: string }[] };
export type Declaration = { readonly id: string; readonly kind: string; readonly number: number | null; readonly slug: string | null;
  readonly path: string; readonly line: number; readonly headingLevel: number; readonly title: string | null;
  readonly sections: readonly Section[]; readonly duplicateSections: readonly Section[]; readonly stub: boolean;
  readonly defines: string | null; readonly e2eCase: E2eCase | null; readonly bodyStart: number; readonly bodyEnd: number;
  readonly bodyHasContent: boolean; readonly source: DeclarationSource; readonly valueValid: boolean | null };
export type Citation = Omit<RefHit, 'project'> & { readonly namespace: string | null;
  readonly shorthand: boolean; readonly localSection: boolean; readonly shorthandRewritable: boolean; readonly numericRun: boolean;
  readonly sourceKind: string; readonly inlineSite: { readonly firstLine: number; readonly lastLine: number;
    readonly maxColumns: number; readonly hasNote: boolean; readonly layoutViolations: readonly number[] } | null };
export type InvalidValueSite = { readonly id: string | null; readonly path: string; readonly line: number;
  readonly column: number | null; readonly message: string; readonly source: DeclarationSource;
  readonly bindingNamespace: string | null; readonly bindingSection: string | null };
export type ScanSnapshot = {
  readonly declarations: readonly Declaration[]; readonly citations: readonly Citation[]; readonly escapedCitations: readonly Citation[];
  readonly scannedFiles: readonly string[]; readonly walkedDirs: readonly string[];
  readonly fileStructures: readonly { readonly path: string; readonly totalLines: number;
    readonly headings: readonly { readonly line: number; readonly level: number; readonly text: string }[];
    readonly docComments: readonly { readonly start: number; readonly end: number; readonly indented: boolean }[] }[];
  readonly valueBindings: readonly { readonly namespace: string | null; readonly id: string; readonly section: string | null;
    readonly authored: ValueComponent; readonly path: string; readonly line: number; readonly column: number }[];
  readonly invalidValueDeclarations: readonly InvalidValueSite[]; readonly invalidValueBindings: readonly InvalidValueSite[];
  readonly sectionHeadingsOutsideDeclarations: readonly { readonly path: string; readonly line: number; readonly section: string }[];
  readonly unmarkedHeadings: readonly { readonly path: string; readonly line: number; readonly column: number; readonly heading: string;
    readonly headingLevel: number; readonly title: string; readonly owner: string; readonly suggestedPath: string }[];
  readonly nearMissHeadings: readonly { readonly path: string; readonly line: number; readonly text: string; readonly format: string }[];
};
export type ScanResult = Cautions & { readonly snapshot: ScanSnapshot };
export declare function check(root?: string, options?: CheckOptions): Promise<CheckResult>;
export declare function scan(root: string): Promise<ScanResult>;
export declare function show(id: string, options?: ShowOptions): Promise<ShowResult>;
export declare function showBatch(queries: readonly (string | ShowQuery)[] | null, options?: BatchOptions): Promise<BatchResult>;
export declare function refs(id: string, options?: RefsOptions): Promise<RefsResult>;
export declare function list(options?: ListOptions): Promise<ListResult>;
export declare function listSizes(options?: SizeOptions): Promise<SizeResult>;
export declare function cover(options?: RootOptions): Promise<CoverResult>;
export declare function fmt(options?: FmtOptions): Promise<FmtResult>;
export declare function proposeId(kind: string, title: string, options?: IdOptions): Promise<IdResult>;
export declare function init(target?: string, options?: InitOptions): Promise<InitResult>;
export declare function completeIds(options?: CompleteOptions): Promise<CompleteResult>;
export declare function effectiveConfig(options?: RootOptions): Promise<ConfigResult>;
export declare function validateConfig(options?: RootOptions): Promise<ConfigResult>;
export declare function fetch(id: string, options?: RootOptions): Promise<FetchResult>;
export declare function integrations(options?: IntegrationOptions): Promise<IntegrationResult>;
export declare function referenceStyle(path: string): Promise<StyleResult>;
export declare function agentSetupInstructions(): Promise<SetupResult>;
