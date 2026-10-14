"""Embed grund locally; PyPI publication is pending (§FS-distribution.3.3.7)."""

from ._api import (agent_setup_instructions, check, complete_ids, cover,
                   effective_config, fetch, fmt, init, integrations, list_ids,
                   list_sizes, propose_id, reference_style, refs, scan, show,
                   show_batch, validate_config)
from .errors import (ConfigError, FilesystemError, GrundError, OperationError,
                     PathEncodingError, QueryError)
from .types import *
