import json
from dataclasses import dataclass
from enum import Enum
from typing import Any, overload

from . import _lib


class Policies:
    """Policies."""

    @overload
    def __init__(self, text: str, /) -> None: ...

    @overload
    def __init__(self, *, policies: _lib.Policies) -> None: ...

    def __init__(
        self,
        text: str | None = None,
        *,
        policies: _lib.Policies | None = None,
    ) -> None:
        if (text is None and policies is None) or (
            text is not None and policies is not None
        ):
            raise TypeError(
                "either policies or text argument must be provided at __init__"
            )
        if policies is not None:
            self._policies = policies
        elif text is not None:
            if text.startswith("{"):
                self._policies = _lib.Policies.from_json(text)
            else:
                self._policies = _lib.Policies.from_string(text)

    def to_json(self) -> str:
        """Format policies to JSON string."""
        return self._policies.to_json()

    def to_string(self) -> str:
        """Format policies to text."""
        return self._policies.to_string()

    def __str__(self) -> str:
        """String representation of policies."""
        return self._policies.to_string()

    def __repr__(self) -> str:
        """Human friendly string representation of policies."""
        return f"Policies({self._policies.to_string()!r})"


class Schema:
    """Schema."""

    @overload
    def __init__(self, text: str, /) -> None: ...

    @overload
    def __init__(self, *, schema: _lib.Schema) -> None: ...

    def __init__(
        self,
        text: str | None = None,
        *,
        schema: _lib.Schema | None = None,
    ) -> None:
        if (text is None and schema is None) or (
            text is not None and schema is not None
        ):
            raise TypeError(
                "either schema or text argument must be provided at __init__"
            )
        if schema is not None:
            self._schema = schema
        elif text is not None:
            if text.startswith("{"):
                self._schema = _lib.Schema.from_json(text)
            else:
                self._schema = _lib.Schema.from_string(text)

    def to_json(self) -> str:
        """Format schema to JSON string."""
        return self._schema.to_json()

    def to_string(self) -> str:
        """Format schema to text."""
        return self._schema.to_string()

    def __str__(self) -> str:
        """String representation of schema."""
        return self._schema.to_string()

    def __repr__(self) -> str:
        """Human friendly string representation of schema."""
        return f"Policies({self._schema.to_string()!r})"


class Entities:
    """Entities."""

    @overload
    def __init__(self, text: str, /) -> None: ...

    @overload
    def __init__(self, *, entities: _lib.Entities) -> None: ...

    def __init__(
        self,
        text: str | None = None,
        *,
        entities: _lib.Entities | None = None,
    ) -> None:
        if (text is None and entities is None) or (
            text is not None and entities is not None
        ):
            raise TypeError(
                "either entities or text argument must be provided at __init__"
            )
        if entities is not None:
            self._entities = entities
        elif text is not None:
            self._entities = _lib.Entities.from_json(text)

    def to_json(self) -> str:
        """Format entities to JSON string."""
        return self._entities.to_json()

    def to_string(self) -> str:
        """Format entities to text."""
        return self._entities.to_string()

    def __str__(self) -> str:
        """String representation of entities."""
        return self._entities.to_json()

    def __repr__(self) -> str:
        """Human friendly string representation of entities."""
        return f"Policies({self._entities.to_json()!r})"


@dataclass
class AuthorizationRequest:
    principal: str
    action: str
    resource: str
    correlation_id: str | None = None
    context: dict[str, Any] | None = None


class AuthorizationDecision(str, Enum):
    DENY = "deny"
    ALLOW = "allow"


@dataclass
class AuthorizationDiagnostics:
    reasons: list[str]
    errors: list[str]


@dataclass
class AuthorizationResponse:
    decision: AuthorizationDecision
    correlation_id: str | None
    diagnostics: AuthorizationDiagnostics


def format_policies(text: str) -> str:
    """Format given policies."""
    return _lib.format_policies(text, line_width=88, indent_width=2)


def is_authorized(
    request: AuthorizationRequest,
    policies: Policies,
    entities: Entities,
    schema: Schema | None = None,
) -> AuthorizationResponse:
    """Return decision."""
    auth = Authorizer(policies, schema)
    return auth.is_authorized(request, entities)


class Authorizer:
    def __init__(self, policies: Policies, schema: Schema | None = None) -> None:
        self._authorizer = _lib.Authorizer.new(
            policies._policies, schema._schema if schema else None
        )

    def update_schema(self, schema: Schema | None) -> None:
        self._authorizer = self._authorizer.with_schema(
            schema._schema if schema else None
        )
        self.schema = schema

    def update_policies(self, policies: Policies | None) -> None:
        self._authorizer = self._authorizer.with_policies(
            policies._policies if policies else None
        )

    def is_authorized(
        self,
        request: AuthorizationRequest,
        entities: Entities,
    ) -> AuthorizationResponse:
        response = self._authorizer.is_authorized(
            _lib.Request.new(
                principal=request.principal,
                action=request.action,
                resource=request.resource,
                context=json.dumps(request.context) if request.context else None,
                correlation_id=request.correlation_id,
            ),
            entities._entities,
        )
        return AuthorizationResponse(
            decision=AuthorizationDecision.ALLOW
            if response.decision == _lib.Decision.Allow
            else AuthorizationDecision.DENY,
            correlation_id=response.correlation_id,
            diagnostics=AuthorizationDiagnostics(
                reasons=response.diagnostics.reasons, errors=response.diagnostics.errors
            ),
        )
