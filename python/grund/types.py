"""Immutable host records (§FS-distribution.3.3.1, §FS-distribution.3.3.2)."""

from dataclasses import dataclass
from collections.abc import Iterator, Mapping
from typing import TypeAlias, Union

Json: TypeAlias = Union[None, bool, int, float, str, tuple["Json", ...], Mapping[str, "Json"]]


@dataclass(frozen=True)
class FindingSite:
    path: str
    line: int


@dataclass(frozen=True)
class Finding:
    severity: str | None
    channel: str | None
    code: str
    path: str | None
    line: int | None
    column: int | None
    message: str
    sites: tuple[FindingSite, ...]
    authority: tuple[str, ...]


@dataclass(frozen=True)
class Report:
    errors: tuple[Finding, ...]
    warnings: tuple[Finding, ...]
    suggestions: tuple[Finding, ...]

    def __iter__(self) -> Iterator[Finding]:
        """Yield channels in engine order (§FS-distribution.3.3.1)."""
        return iter(self.errors + self.warnings + self.suggestions)


@dataclass(frozen=True, kw_only=True)
class Cautioned:
    run_cautions: tuple[Finding, ...]


@dataclass(frozen=True)
class CheckResult(Cautioned):
    report: Report
    selected_report: Report
    had_scan_errors: bool
    output_format: str


@dataclass(frozen=True)
class Failure(Cautioned):
    kind: str
    code: str
    message: str
    path: str | None
    line: int | None
    column: int | None
    sites: tuple[FindingSite, ...]
    authority: tuple[str, ...]
    causes: tuple[str, ...]
    partial_output: Json
    details: Mapping[str, Json]


@dataclass(frozen=True)
class ScanError:
    path: str
    message: str


@dataclass(frozen=True)
class ScanSection:
    path: str
    title: str
    line: int
    heading_level: int


@dataclass(frozen=True)
class ScanDeclaration:
    id: str
    path: str
    line: int
    heading_level: int
    title: str | None
    stub: bool
    defines: str | None
    body_start: int
    body_end: int
    body_has_content: bool
    value_valid: bool | None
    sections: tuple[ScanSection, ...]
    duplicate_sections: tuple[ScanSection, ...]


@dataclass(frozen=True)
class ScanCitation:
    project: str | None
    id: str
    section: str | None
    path: str
    line: int
    column: int
    marker: bool
    text: str
    shorthand: bool
    local_section: bool
    shorthand_rewritable: bool
    numeric_run: bool
    source_kind: str
    enclosing_declaration: str | None
    enclosing_section: str | None


@dataclass(frozen=True)
class ScanResult(Cautioned):
    catalog: tuple[ScanDeclaration, ...]
    citations: tuple[ScanCitation, ...]
    scanned_files: tuple[str, ...]
    scan_errors: tuple[ScanError, ...]


@dataclass(frozen=True)
class ShowQuery:
    id: str
    section: str | None = None


@dataclass(frozen=True)
class ShowSection:
    path: str
    title: str
    depth: int


@dataclass(frozen=True)
class ShowResult(Cautioned):
    body: str
    path: str
    line: int
    json: str | None
    sections: tuple[ShowSection, ...]


@dataclass(frozen=True)
class BatchRecord:
    query: ShowQuery
    result: ShowResult | None
    failure: Failure | None


@dataclass(frozen=True)
class BatchResult(Cautioned):
    records: tuple[BatchRecord, ...]


@dataclass(frozen=True)
class RefHit:
    project: str | None
    path: str
    line: int
    column: int
    id: str
    section: str | None
    marker: bool
    text: str
    enclosing_declaration: str | None
    enclosing_section: str | None


@dataclass(frozen=True)
class FileSummary:
    project: str | None
    path: str
    sites: int


@dataclass(frozen=True)
class RefsResult(Cautioned):
    output_format: str
    workspace: bool
    hits: tuple[RefHit, ...]
    note: str | None
    scan_errors: tuple[ScanError, ...]
    kind_title: str | None
    site_total: int
    file_total: int
    file_summaries: tuple[FileSummary, ...]


@dataclass(frozen=True)
class ValueRoot:
    id: str
    valid: bool


@dataclass(frozen=True)
class ListEntry:
    project: str | None
    id: str
    section: str | None
    section_separator: str
    kind: str
    path: str
    line: int
    title: str | None
    stub: bool
    defines: str | None
    refs: int
    duplicate: bool
    value_roots: tuple[ValueRoot, ...]


@dataclass(frozen=True)
class ListSummary:
    project: str | None
    kind: str
    title: str
    home: str
    count: int


@dataclass(frozen=True)
class ListResult(Cautioned):
    output_format: str
    workspace: bool
    entries: tuple[ListEntry, ...]
    summaries: tuple[ListSummary, ...]
    scan_errors: tuple[ScanError, ...]


@dataclass(frozen=True)
class SizeMeasurement:
    unit: str
    lead: int | None
    full: int | None


@dataclass(frozen=True)
class SizeEntry:
    project: str | None
    id: str
    section: str | None
    section_separator: str
    kind: str
    path: str
    line: int
    stub: bool
    defines: str | None
    duplicate: bool
    measurements: tuple[SizeMeasurement, ...]


@dataclass(frozen=True)
class SizesResult(Cautioned):
    output_format: str
    workspace: bool
    entries: tuple[SizeEntry, ...]
    scan_errors: tuple[ScanError, ...]


@dataclass(frozen=True)
class CoverEntry:
    project: str | None
    path: str
    citations: tuple[RefHit, ...]


@dataclass(frozen=True)
class CoverTextCitation:
    line: int
    column: int
    text: str


@dataclass(frozen=True)
class CoverTextEntry:
    path: str
    citations: tuple[CoverTextCitation, ...]


@dataclass(frozen=True)
class CoverResult(Cautioned):
    output_format: str
    entries: tuple[CoverEntry, ...]
    scan_errors: tuple[ScanError, ...]


@dataclass(frozen=True)
class CoverTextResult(Cautioned):
    output_format: str
    entries: tuple[CoverTextEntry, ...]
    scan_errors: tuple[ScanError, ...]


@dataclass(frozen=True)
class CoverLineSection:
    section: str | None
    start: int
    end: int


@dataclass(frozen=True)
class CoverLineOwner:
    declaration: str
    start: int
    end: int
    sections: tuple[CoverLineSection, ...]


@dataclass(frozen=True)
class CoverLines:
    project: str | None
    path: str
    start: int
    end: int
    owners: tuple[CoverLineOwner, ...]


@dataclass(frozen=True)
class CoverLinesResult(Cautioned):
    output_format: str
    records: tuple[CoverLines, ...]
    scan_errors: tuple[ScanError, ...]


@dataclass(frozen=True)
class FmtChange:
    path: str
    line: int
    label: str


@dataclass(frozen=True)
class FmtResult(Cautioned):
    changes: tuple[FmtChange, ...]
    scan_errors: tuple[ScanError, ...]
    refused_writes: tuple[str, ...]


@dataclass(frozen=True)
class IdProposal(Cautioned):
    id: str
    kind: str
    number: int | None
    slug: str
    folder: str | None
    file: str | None
    e2e_case_dir: str | None
    file_holds_single_declaration: bool


@dataclass(frozen=True)
class InitEvent:
    verb: str
    path: str


@dataclass(frozen=True)
class InitFsHome:
    kind: str
    path: str
    heading_name: str | None
    heading_marker: str | None


@dataclass(frozen=True)
class InitNext:
    docs: bool
    entrypoint: str
    fs_home: InitFsHome
    scan_reads_file: bool


@dataclass(frozen=True)
class InitResult(Cautioned):
    events: tuple[InitEvent, ...]
    errors: tuple[Finding, ...]
    notes: tuple[str, ...]
    next: InitNext | None
    has_pending_changes: bool


@dataclass(frozen=True)
class ConfigResult(Cautioned):
    config: Mapping[str, Json]
    warnings: tuple[str, ...]


@dataclass(frozen=True)
class CompletionResult(Cautioned):
    candidates: tuple[str, ...]


@dataclass(frozen=True)
class ReferenceStyle(Cautioned):
    marker: str
    trigger: str


@dataclass(frozen=True)
class FetchResult(Cautioned):
    pass


@dataclass(frozen=True)
class IntegrationClient:
    client: str
    detected: bool
    installed: bool | None
    install_kind: str
    install: str
    config_target: str


@dataclass(frozen=True)
class IntegrationArtifact:
    client: str
    snippet: str | None
    resolver: str | None
    package_json: str | None
    extension_js: str | None


@dataclass(frozen=True)
class IntegrationEvent:
    path: str
    verb: str
    note: str | None


@dataclass(frozen=True)
class IntegrationsResult(Cautioned):
    detected: tuple[str, ...]
    clients: tuple[IntegrationClient, ...]
    artifact: IntegrationArtifact | None
    events: tuple[IntegrationEvent, ...]
    manual_steps: tuple[str, ...]


@dataclass(frozen=True)
class SetupInstructions(Cautioned):
    text: str
