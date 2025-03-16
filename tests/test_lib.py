import json

import pytest
from cedar import (
    Authorizer,
    Decision,
    Entities,
    EntityUid,
    PolicySet,
    Request,
    format_policies,
)

policies: str = """
    permit(
        principal,
        action == Action::"edit",
        resource

    )
    when {
        resource.owner == principal
    };
"""
formatted_policies = """permit (
  principal,
  action == Action::"edit",
  resource
)
when { resource.owner == principal };
"""
generated_policies = """permit (
  principal,
  action == Action::"edit",
  resource
)
when { (resource["owner"]) == principal };
"""
json_policies = json.dumps(
    {
        "templates": {},
        "staticPolicies": {
            "policy0": {
                "effect": "permit",
                "principal": {"op": "All"},
                "action": {"op": "==", "entity": {"type": "Action", "id": "edit"}},
                "resource": {"op": "All"},
                "conditions": [
                    {
                        "kind": "when",
                        "body": {
                            "==": {
                                "left": {
                                    ".": {
                                        "left": {"Var": "resource"},
                                        "attr": "owner",
                                    }
                                },
                                "right": {"Var": "principal"},
                            }
                        },
                    }
                ],
            }
        },
        "templateLinks": [],
    },
    separators=(",", ":"),
)


class TestPolicies:
    def test_to_json(self) -> None:
        assert PolicySet.from_string(policies).to_json() == json_policies

    def test_from_json(self) -> None:
        assert (
            format_policies(PolicySet.from_json(json_policies).to_string())
            == generated_policies
        )

    def test_from_json__init__(self) -> None:
        assert (
            format_policies(PolicySet.from_json(json_policies).to_string())
            == generated_policies
        )


def test_format_policies() -> None:
    assert format_policies("") == "\n"

    assert format_policies(policies) == formatted_policies


def test_format_error_policies() -> None:
    with pytest.raises(ValueError, match="cannot parse input policies"):
        format_policies("permmit()")


def test_policies_to_json() -> None:
    assert PolicySet.from_string(policies).to_json() == json.dumps(
        {
            "templates": {},
            "staticPolicies": {
                "policy0": {
                    "effect": "permit",
                    "principal": {"op": "All"},
                    "action": {"op": "==", "entity": {"type": "Action", "id": "edit"}},
                    "resource": {"op": "All"},
                    "conditions": [
                        {
                            "kind": "when",
                            "body": {
                                "==": {
                                    "left": {
                                        ".": {
                                            "left": {"Var": "resource"},
                                            "attr": "owner",
                                        }
                                    },
                                    "right": {"Var": "principal"},
                                }
                            },
                        }
                    ],
                }
            },
            "templateLinks": [],
        },
        separators=(",", ":"),
    )


def test_policies_from_json() -> None:
    assert (
        format_policies(
            PolicySet.from_json(PolicySet.from_string(policies).to_json()).to_string()
        )
        == generated_policies
    )


def test_error_invalid_policy_from_json() -> None:
    with pytest.raises(ValueError):
        PolicySet.from_json("{")


def test_error_invalid_policy_to_json() -> None:
    with pytest.raises(ValueError):
        PolicySet.from_json("toto")


def test_is_authorized() -> None:
    policies = """permit (
        principal == User::"alice",
        action == Action::"view",
        resource in Album::"jane_vacation"
    );
    """
    auth = Authorizer(policies=PolicySet.from_string(policies), schema=None)
    entities = """[
        {
            "uid": { "type": "User", "id": "alice" },
            "attrs": { "age": 18 },
            "parents": []
        },
        {
            "uid": { "type": "User", "id": "bob" },
            "attrs": { "age": 18 },
            "parents": []
        },
        {
            "uid": { "type": "Photo", "id": "VacationPhoto94.jpg" },
            "attrs": {},
            "parents": [{ "type": "Album", "id": "jane_vacation" }]
        }
    ]
    """
    response = auth.is_authorized(
        Request(
            principal=EntityUid.from_type_name_and_id("User", "alice"),
            action=EntityUid.from_type_name_and_id("Action", "view"),
            resource=EntityUid.from_type_name_and_id("Photo", "VacationPhoto94.jpg"),
        ),
        Entities.from_json(entities),
    )
    assert response.decision == Decision.Allow
    assert response.correlation_id is None
    assert response.diagnostics.reasons == {"policy0"}
    assert response.diagnostics.errors == []


def test_is_not_authorized() -> None:
    policies = """permit (
        principal == User::"alice",
        action == Action::"view",
        resource in Album::"jane_vacation"
    );
    """
    auth = Authorizer(policies=PolicySet.from_string(policies), schema=None)
    entities = """[
        {
            "uid": { "type": "User", "id": "alice" },
            "attrs": { "age": 18 },
            "parents": []
        },
        {
            "uid": { "type": "User", "id": "bob" },
            "attrs": { "age": 18 },
            "parents": []
        },
        {
            "uid": { "type": "Photo", "id": "VacationPhoto94.jpg" },
            "attrs": {},
            "parents": [{ "type": "Album", "id": "jane_vacation" }]
        }
    ]
    """
    response = auth.is_authorized(
        Request(
            principal=EntityUid.from_type_name_and_id("User", "bob"),
            action=EntityUid.from_type_name_and_id("Action", "view"),
            resource=EntityUid.from_type_name_and_id("Photo", "VacationPhoto94.jpg"),
            context={"oidc_scope": "profile"},
        ),
        Entities.from_json(entities),
    )
    assert response.decision == Decision.Deny
    assert response.correlation_id is None
    assert response.diagnostics.reasons == set()
    assert response.diagnostics.errors == []
