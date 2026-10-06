"""Operational failures and path rejection (§FS-distribution.3.3.2)."""

from .types import Failure


class GrundError(Exception):
    """An operation could not complete; .failure retains its engine payload."""

    def __init__(self, failure: Failure) -> None:
        self.failure = failure
        super().__init__(failure.message)


class ConfigError(GrundError):
    """Configuration discovery, parsing or validation failed."""


class FilesystemError(GrundError):
    """A required filesystem operation failed."""


class QueryError(GrundError):
    """A single query could not resolve."""


class OperationError(GrundError):
    """Another engine operation failed."""


class PathEncodingError(ValueError):
    """The supplied path or an unnamed workspace alias cannot cross this API."""
