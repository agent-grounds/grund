"""Lossless dataclass conversion, with no engine decisions (§AR-bindings.6)."""

from dataclasses import fields, is_dataclass
from collections.abc import Mapping
from types import MappingProxyType, UnionType
from typing import Any, TypeVar, Union, get_args, get_origin, get_type_hints

T = TypeVar("T")


def immutable(value: Any) -> Any:
    """Freeze opaque schema/details payloads recursively (§FS-distribution.3.3.1)."""
    if isinstance(value, dict):
        return MappingProxyType({k: immutable(v) for k, v in value.items()})
    if isinstance(value, list):
        return tuple(immutable(v) for v in value)
    return value


def convert(cls: type[T], value: Any) -> T:
    """Enumerate the declared type; every field is mandatory in native data."""
    return _convert(cls, value)


def _convert(hint: Any, value: Any) -> Any:
    if value is None:
        return None
    origin = get_origin(hint)
    args = get_args(hint)
    if origin in (Union, UnionType):
        alternatives = [a for a in args if a is not type(None)]
        if len(alternatives) == 1:
            return _convert(alternatives[0], value)
        return immutable(value)
    if origin is tuple:
        return tuple(_convert(args[0], v) for v in value)
    if origin is Mapping:
        return immutable(value)
    if is_dataclass(hint):
        hints = get_type_hints(hint)
        expected = {f.name for f in fields(hint)}
        if expected != set(value):
            raise RuntimeError(f"native {hint.__name__} fields differ: {expected ^ set(value)}")
        return hint(**{k: _convert(hints[k], v) for k, v in value.items()})
    return immutable(value)
