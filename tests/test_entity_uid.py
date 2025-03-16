import json

from cedar import EntityUid


def test_init() -> None:
    euid = EntityUid.from_type_name_and_id("a", "b")
    assert euid.entity_type == "a"
    assert euid.entity_id == "b"


def test_eq() -> None:
    assert EntityUid.from_type_name_and_id("a", "b") == EntityUid.from_type_name_and_id(
        "a", "b"
    )


def test_neq_different_entity_id() -> None:
    assert EntityUid.from_type_name_and_id("a", "b") != EntityUid.from_type_name_and_id(
        "a", "c"
    )


def test_neq_different_entity_type() -> None:
    assert EntityUid.from_type_name_and_id("x", "b") != EntityUid.from_type_name_and_id(
        "y", "b"
    )


def test_entity_type_property() -> None:
    assert (
        EntityUid.from_type_name_and_id("MyApp::User", "alice").entity_type
        == "MyApp::User"
    )


def test_entity_id_property() -> None:
    assert EntityUid.from_type_name_and_id("MyApp::User", "alice").entity_id == "alice"


def test_to_string() -> None:
    assert (
        EntityUid.from_type_name_and_id("MyApp::User", "alice").to_string()
        == 'MyApp::User::"alice"'
    )


def test_magic_method_str() -> None:
    assert (
        str(EntityUid.from_type_name_and_id("MyApp::User", "alice"))
        == 'MyApp::User::"alice"'
    )


def test_magic_method_repr() -> None:
    assert (
        repr(EntityUid.from_type_name_and_id("MyApp::User", "alice"))
        == 'EntityUid(entity_type="MyApp::User", entity_id="alice")'
    )


def test_to_json() -> None:
    assert EntityUid.from_type_name_and_id(
        "MyApp::User", "alice"
    ).to_json() == json.dumps(
        {"type": "MyApp::User", "id": "alice"}, separators=(",", ":")
    )


def test_to_dict() -> None:
    assert EntityUid.from_type_name_and_id("MyApp::User", "alice").to_dict() == {
        "type": "MyApp::User",
        "id": "alice",
    }


def test_from_json() -> None:
    assert EntityUid.from_json(
        json.dumps({"type": "MyApp::User", "id": "alice"})
    ) == EntityUid.from_type_name_and_id("MyApp::User", "alice")


def test_from_string() -> None:
    assert EntityUid.from_string(
        'MyApp::User::"alice"'
    ) == EntityUid.from_type_name_and_id("MyApp::User", "alice")


def test_from_dict() -> None:
    assert EntityUid.from_dict(
        {"type": "MyApp::User", "id": "alice"}
    ) == EntityUid.from_type_name_and_id("MyApp::User", "alice")
