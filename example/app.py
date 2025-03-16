import abc
import pathlib
from dataclasses import dataclass
from typing import Any, ClassVar, Protocol

from cedar import (
    Authorizer,
    Entities,
    EntityUid,
    PolicySet,
    Request,
    Response,
    Schema,
)


class Action(metaclass=abc.ABCMeta):
    """Base class for actions."""

    id: ClassVar[str]

    def __init_subclass__(cls, id: str, **kwargs: Any) -> None:
        cls.id = id
        super().__init_subclass__(**kwargs)

    @abc.abstractmethod
    def resource(self) -> tuple[str, str]:
        raise NotImplementedError

    @abc.abstractmethod
    def context(self) -> dict[str, Any]:
        raise NotImplementedError


@dataclass
class Resource:
    """Base class for resources."""

    # Class variables
    type: ClassVar[str]
    # Attributes
    id: str

    def __init_subclass__(cls, type: str, **kwargs: Any) -> None:
        cls.type = type
        super().__init_subclass__(**kwargs)

    def get_parents(self) -> list["Reference"]:
        return []

    def get_attrs(self) -> dict[str, Any]:
        return {}

    def to_entity(self) -> dict[str, Any]:
        entity = {
            "uid": {
                "id": self.id,
                "type": self.type,
            },
            "parents": [
                {"id": parent.id, "type": parent.type} for parent in self.get_parents()
            ],
            "attrs": self.get_attrs(),
        }
        return entity


class EntityStoreProtocol(Protocol):
    def get_entities(self, schema: Schema | None) -> Entities: ...


class AppContext:
    """Application context."""

    def __init__(
        self,
        policies_file: pathlib.Path,
        schema_file: pathlib.Path,
        entity_store: EntityStoreProtocol,
    ) -> None:
        """Create a new app context."""
        self.policies_file = policies_file
        self.schema_file = schema_file
        self.entity_store = entity_store
        self.reload_policies()

    def reload_policies(self) -> None:
        """Reload policies and schema from file."""
        self.schema = Schema.from_string(self.schema_file.read_text())
        self.policies = PolicySet.from_string(self.policies_file.read_text())
        self.authorizer = Authorizer(self.policies, self.schema)

    def is_authorized(
        self,
        principal: tuple[str, str],
        action: tuple[str, Action],
        context: dict[str, Any] | None = None,
    ) -> Response:
        """Check if user is authorized to perform actions on resource."""
        request = Request(
            principal=EntityUid.from_type_name_and_id(*principal),
            action=EntityUid.from_type_name_and_id(action[0], action[1].id),
            resource=EntityUid.from_type_name_and_id(*action[1].resource()),
            context={**action[1].context(), **(context or {})},
        )
        response = self.authorizer.is_authorized(
            request, self.entity_store.get_entities(self.schema)
        )
        return response


@dataclass
class Reference:
    """Reference."""

    # Class variables
    type: str
    # Attributes
    id: str


def resource_to_entities(resources: list[Resource], schema: Schema | None) -> Entities:
    """Make entities from a list of resources."""
    entities = [resource.to_entity() for resource in resources]
    return Entities.from_list(entities, schema=schema)


def reference(resource_type: type[Resource], id: str) -> Reference:
    return Reference(resource_type.type, id)


@dataclass
class User(Resource, type="PhotoFlash::User"):
    id: str
    department: str
    jobLevel: int
    groups: list[str]

    def get_parents(self) -> list[Reference]:
        return [reference(UserGroup, group) for group in self.groups]

    def get_attrs(self) -> dict[str, Any]:
        return {"department": self.department, "jobLevel": self.jobLevel}


@dataclass
class UserGroup(Resource, type="PhotoFlash::UserGroup"):
    id: str


@dataclass
class Account(Resource, type="PhotoFlash::Account"):
    id: str
    admins: list[str]
    owner: str

    def get_attrs(self) -> dict[str, Any]:
        return {
            "owner": reference(User, self.owner),
            "admins": [reference(User, admin) for admin in self.admins],
        }


@dataclass
class Photo(Resource, type="PhotoFlash::Photo"):
    id: str
    album: str
    account: str
    private: bool

    def get_parents(self) -> list[Reference]:
        return [reference(Album, self.album)]

    def get_attrs(self) -> dict[str, Any]:
        return {
            "account": reference(Account, self.account),
            "private": self.private,
        }


@dataclass
class Album(Resource, type="PhotoFlash::Album"):
    id: str
    account: str
    private: bool

    def get_attrs(self) -> dict[str, Any]:
        return {
            "account": reference(Account, self.account),
            "private": self.private,
        }


class EntityStore:
    def __init__(self) -> None:
        self.users: dict[str, User] = {}
        self.groups: dict[str, UserGroup] = {}
        self.accounts: dict[str, Account] = {}
        self.albums: dict[str, Album] = {}

    def add_group(self, group: UserGroup) -> None:
        if group.id in self.groups:
            raise ValueError("group already exists")
        self.groups[group.id] = group

    def add_user(self, user: User) -> None:
        if user.id in self.users:
            raise ValueError("user id already exists")
        self.users[user.id] = user

    def update_user(self, user: User) -> None:
        for existing_user in self.users:
            if user.id == existing_user:
                self.users[user.id] = user
                break
        else:
            raise ValueError("user does not exist")

    def get_entities(self, schema: Schema | None) -> Entities:
        entities = resource_to_entities(
            [
                *self.groups.values(),
                *self.users.values(),
                *self.accounts.values(),
                *self.albums.values(),
            ],
            schema=schema,
        )
        return entities


@dataclass
class ViewPhoto(Action, id="viewPhoto"):
    authenticated: bool
    photo: str

    def resource(self) -> tuple[str, str]:
        return Photo.type, self.photo

    def context(self) -> dict[str, Any]:
        return {"authenticated": self.authenticated}


@dataclass
class UploadPhoto(Action, id="uploadPhoto"):
    album: str
    file_size: int
    file_type: str
    authenticated: bool

    def resource(self) -> tuple[str, str]:
        return Album.type, self.album

    def context(self) -> dict[str, Any]:
        return {
            "authenticated": self.authenticated,
            "photo": {"file_size": self.file_size, "file_type": self.file_type},
        }


store = EntityStore()
policies_file = pathlib.Path(__file__).parent.joinpath("policies.cedar")
schema_file = policies_file.parent.joinpath("schema.cedar")
ctx = AppContext(
    policies_file=policies_file, schema_file=schema_file, entity_store=store
)

store.add_group(UserGroup(id="test-group"))

store.add_group(UserGroup(id="default-group"))

print("Can alice read a single photo ?")
response = ctx.is_authorized(
    principal=("PhotoFlash::User", "alice"),
    action=(
        "PhotoFlash::Action",
        ViewPhoto(photo="VacationPhoto94.jpg", authenticated=True),
    ),
)
print(response.to_json())

print("Can alice upload a single photo ?")
response = ctx.is_authorized(
    principal=("PhotoFlash::User", "alice"),
    action=(
        "PhotoFlash::Action",
        UploadPhoto(
            album="Vacation",
            authenticated=True,
            file_size=2,
            file_type="txt",
        ),
    ),
)
print(response.to_json())

store.add_user(
    User(
        id="alice",
        department="A",
        jobLevel=1,
        groups=["default-group", "test-group"],
    )
)

print("Can alice upload a single photo ?")
response = ctx.is_authorized(
    principal=("PhotoFlash::User", "alice"),
    action=(
        "PhotoFlash::Action",
        UploadPhoto(
            album="Vacation",
            authenticated=True,
            file_size=2,
            file_type="txt",
        ),
    ),
)
print(response.to_json())

print("Can alice upload a single photo ?")
response = ctx.is_authorized(
    principal=("PhotoFlash::User", "alice"),
    action=(
        "PhotoFlash::Action",
        UploadPhoto(
            album="Vacation",
            authenticated=True,
            file_size=20,
            file_type="txt",
        ),
    ),
)
print(response.to_json())

# ctx.entity_store.albums["Vacation"] = Album(
#     "Vacation", account="bob-account", private=True
# )
# ctx.entity_store.add_user(User("bob", "depB", 2, ["default-group"]))
# ctx.is_user_authorized(
#     user="bob",
#     action=UploadPhoto(
#         album="Vacation",
#         authenticated=True,
#         file_size=20,
#         file_type="txt",
#     ),
# )
# print(asdict(response))
