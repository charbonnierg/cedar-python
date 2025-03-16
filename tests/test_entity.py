import json

from cedar import Entity

dict_entity = {
    "uid": {"type": "User", "id": "alice"},
    "attrs": {"active": True},
    "parents": [{"type": "Group", "id": "admins"}],
}
json_entity = json.dumps(dict_entity, separators=(",", ":"))


def test_from_dict() -> None:
    entity = Entity.from_dict(dict_entity)
    assert json.loads(entity.to_json()) == dict_entity


def test_from_json() -> None:
    entity = Entity.from_json(json_entity)
    assert json.loads(entity.to_json()) == dict_entity


def test_to_json() -> None:
    entity = Entity.from_json(json_entity)
    assert entity.to_json() == json_entity


def test_to_dict() -> None:
    entity = Entity.from_json(json_entity)
    assert entity.to_dict() == dict_entity
