from enum import Enum

def format_policies(s: str, /, *, line_width: int, indent_width: int) -> str: ...

class Policies:
    @staticmethod
    def from_json(s: str, /) -> Policies: ...
    @staticmethod
    def from_string(s: str, /) -> Policies: ...
    def to_json(self) -> str: ...
    def to_string(self) -> str: ...
    def to_pretty_string(
        self, line_width: int | None = None, ident_width: int | None = None
    ) -> str: ...

class Entities:
    @staticmethod
    def from_json(s: str, /) -> Entities: ...
    def to_json(self) -> str: ...
    def to_string(self) -> str: ...

class SchemaValidationResult:
    @property
    def passed(self) -> bool: ...
    @property
    def passed_without_warning(self) -> bool: ...
    @property
    def errors(self) -> list[str]: ...
    @property
    def warnings(self) -> list[str]: ...
    def to_string(self) -> str: ...

class EntityValidationResult:
    @property
    def passed(self) -> bool: ...
    @property
    def error(self) -> str: ...
    @property
    def entities(self) -> Entities: ...

class Schema:
    @staticmethod
    def from_json(s: str, /) -> Schema: ...
    @staticmethod
    def from_string(s: str, /) -> Schema: ...
    def to_json(self) -> str: ...
    def to_string(self) -> str: ...
    def validate_policies(self, policies: Policies) -> SchemaValidationResult: ...
    def validate_entities(self, entities: Entities) -> EntityValidationResult: ...

class Request:
    @staticmethod
    def new(
        principal: str,
        action: str,
        resource: str,
        context: str | None,
        correlation_id: str | None,
    ) -> Request: ...

class Diagnostics:
    @property
    def reasons(self) -> list[str]: ...
    @property
    def errors(self) -> list[str]: ...

class Decision(Enum):
    Allow = ...
    Deny = ...

class Response:
    @property
    def decision(self) -> Decision: ...
    @property
    def correlation_id(self) -> str | None: ...
    @property
    def diagnostics(self) -> Diagnostics: ...

class Authorizer:
    @staticmethod
    def new(
        policies: Policies | None = None, schema: Schema | None = None
    ) -> Authorizer: ...
    def with_schema(self, schema: Schema | None) -> Authorizer: ...
    def with_policies(self, policies: Policies | None) -> Authorizer: ...
    def is_authorized(self, request: Request, entities: Entities) -> Response: ...
