from __future__ import annotations

from typing import Any, final

__all__ = [
    "Authorizer",
    "Decision",
    "Diagnostics",
    "Effect",
    "Entities",
    "Entity",
    "EntityUid",
    "Policy",
    "PolicySet",
    "Request",
    "Response",
    "Schema",
    "ValidationResult",
    "format_policies",
    "is_authorized",
]
"""Objects imported when using a star import"""

def format_policies(
    s: str, /, *, line_width: int | None = None, indent_width: int | None = None
) -> str: ...

# Request / Response

@final
class ValidationResult:
    """Result for policy validation against a schema."""
    @property
    def passed(self) -> bool:
        """Return True if validation passed."""

    @property
    def passed_without_warning(self) -> bool:
        """Return True if validation passed without warning."""
    @property
    def errors(self) -> list[str]: ...
    @property
    def warnings(self) -> list[str]: ...
    def to_dict(self) -> dict[str, Any]: ...
    def to_string(self) -> str: ...

@final
class Request:
    def __init__(
        self,
        principal: EntityUid,
        action: EntityUid,
        resource: EntityUid,
        context: dict[str, Any] | None = None,
        correlation_id: str | None = None,
    ) -> None: ...
    @staticmethod
    def from_json(text: str) -> Request: ...
    @staticmethod
    def from_dict(values: dict[str, Any]) -> Request: ...
    @property
    def principal(self) -> EntityUid: ...
    @property
    def action(self) -> EntityUid: ...
    @property
    def resource(self) -> EntityUid: ...
    @property
    def context(self) -> str | None: ...
    @property
    def correlation_id(self) -> str | None: ...

@final
class Diagnostics:
    def __init__(
        self, reason: list[str] | None = None, errors: list[str] | None = None
    ) -> None: ...
    @property
    def reasons(self) -> set[str]: ...
    @property
    def errors(self) -> list[str]: ...
    def to_json(self) -> str: ...
    def to_dict(self) -> dict[str, Any]: ...
    @staticmethod
    def from_json(text: str) -> Diagnostics: ...
    @staticmethod
    def from_dict(values: dict[str, Any]) -> Diagnostics: ...

@final
class Decision:
    Deny: Decision
    Allow: Decision

@final
class Response:
    def __init__(
        self,
        decision: Decision,
        diagnostics: Diagnostics,
        correlation_id: str | None = None,
    ) -> None: ...
    @property
    def decision(self) -> Decision: ...
    @property
    def correlation_id(self) -> str | None: ...
    @property
    def diagnostics(self) -> Diagnostics: ...
    def to_json(self) -> str: ...
    def to_dict(self) -> dict[str, Any]: ...
    @staticmethod
    def from_json(text: str) -> Response: ...
    @staticmethod
    def from_dict(values: dict[str, Any]) -> Response: ...

# Classes

@final
class EntityUid:
    def __init__(self, entity_type: str, entity_id: str) -> None: ...
    @property
    def entity_type(self) -> str: ...
    @property
    def entity_id(self) -> str: ...
    @staticmethod
    def from_json(text: str, /) -> EntityUid: ...
    @staticmethod
    def from_string(text: str, /) -> EntityUid: ...
    @staticmethod
    def from_dict(values: dict[str, str]) -> EntityUid: ...
    @staticmethod
    def from_type_name_and_id(name: str, id: str) -> EntityUid: ...
    def to_string(self) -> str: ...
    def to_json(self) -> str: ...
    def to_dict(self) -> dict[str, str]: ...

@final
class Entity:
    def __init__(
        self,
        euid: EntityUid,
        parents: list[EntityUid],
        attrs: dict[str, Any],
        schema: Schema | None = None,
    ) -> None: ...
    def to_json(self) -> str: ...
    def to_dict(self) -> dict[str, Any]: ...
    @staticmethod
    def from_json(text: str, /, *, schema: Schema | None = None) -> Entity: ...
    @staticmethod
    def from_dict(
        values: dict[str, Any], /, *, schema: Schema | None = None
    ) -> Entity: ...

@final
class Entities:
    def __init__(
        self, entities: list[Entity], schema: Schema | None = None
    ) -> None: ...
    @staticmethod
    def from_json(text: str, /, *, schema: Schema | None = None) -> Entities: ...
    @staticmethod
    def from_list(
        values: list[dict[str, Any]], /, *, schema: Schema | None = None
    ) -> Entities: ...
    def to_json(self) -> str: ...
    def to_list(self) -> list[dict[str, Any]]: ...

@final
class Effect:
    Forbid: Effect
    Permit: Effect

@final
class Policy:
    @staticmethod
    def from_json(text: str, id: str | None = None) -> Policy: ...
    @staticmethod
    def from_string(text: str, id: str | None = None) -> Policy: ...
    @staticmethod
    def from_dict(values: dict[str, Any], /, *, id: str | None = None) -> Policy: ...
    def to_dict(self) -> dict[str, Any]: ...
    def to_json(self) -> str: ...
    def to_string(self) -> str: ...
    def to_pretty_string(
        self, *, line_width: int | None = None, indent_width: int | None = None
    ) -> str: ...
    @property
    def effect(self) -> Effect: ...
    @property
    def policy_id(self) -> str: ...

@final
class PolicySet:
    def __init__(self, policies: list[Policy], /) -> None: ...
    @staticmethod
    def from_json(text: str, /) -> PolicySet: ...
    @staticmethod
    def from_string(text: str, /) -> PolicySet: ...
    @staticmethod
    def from_dict(values: dict[str, Any], /) -> PolicySet: ...
    def to_dict(self) -> dict[str, Any]: ...
    def to_json(self) -> str: ...
    def to_string(self) -> str: ...
    def to_pretty_string(
        self, *, line_width: int | None = None, indent_width: int | None = None
    ) -> str: ...
    @property
    def policies(self) -> list[Policy]: ...

@final
class Schema:
    @staticmethod
    def from_json(text: str, /) -> Schema: ...
    @staticmethod
    def from_string(text: str, /) -> Schema: ...
    @staticmethod
    def from_dict(values: dict[str, Any], /) -> Schema: ...
    def to_dict(self) -> dict[str, Any]: ...
    def to_json(self) -> str: ...
    def to_string(self) -> str: ...
    def validate_policies(self, policies: PolicySet, /) -> ValidationResult: ...

@final
class Authorizer:
    def __init__(
        self, policies: PolicySet | None = None, schema: Schema | None = None
    ) -> None: ...
    def is_authorized(
        self, request: Request, entities: Entities | None = None
    ) -> Response: ...
    def is_authorized_batch(
        self, requests: list[Request], entities: Entities | None = None
    ) -> list[Response]: ...

def is_authorized(
    request: Request,
    policies: PolicySet,
    entities: Entities | None = None,
    schema: Schema | None = None,
) -> Response: ...
def is_authorized_batch(
    requests: list[Request],
    policies: PolicySet,
    entities: Entities | None = None,
    schema: Schema | None = None,
) -> list[Response]: ...
