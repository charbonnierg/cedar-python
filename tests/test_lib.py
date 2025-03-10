import json

import pytest
from cedar import (
    AuthorizationDecision,
    AuthorizationRequest,
    Entities,
    Policies,
    format_policies,
    is_authorized,
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
        assert Policies(policies).to_json() == json_policies

    def test_from_json(self) -> None:
        assert (
            format_policies(Policies(json_policies).to_string()) == generated_policies
        )


def test_format_policies() -> None:
    assert format_policies("") == "\n"

    assert format_policies(policies) == formatted_policies


def test_format_error_policies() -> None:
    with pytest.raises(ValueError, match="cannot parse input policies"):
        format_policies("permmit()")


def test_policies_to_json() -> None:
    assert Policies(policies).to_json() == json.dumps(
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
        format_policies(Policies(Policies(policies).to_json()).to_string())
        == generated_policies
    )


def test_error_invalid_policy_from_json() -> None:
    with pytest.raises(ValueError):
        Policies("{")


def test_error_invalid_policy_to_json() -> None:
    with pytest.raises(ValueError):
        Policies("toto")


def test_is_authorized() -> None:
    policies = """permit (
        principal == User::"alice",
        action == Action::"view",
        resource in Album::"jane_vacation"
    );
    """
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
    response = is_authorized(
        request=AuthorizationRequest(
            principal='User::"alice"',
            action='Action::"view"',
            resource='Photo::"VacationPhoto94.jpg"',
        ),
        policies=Policies(policies),
        entities=Entities(entities),
    )
    assert response.decision == AuthorizationDecision.ALLOW
    assert response.correlation_id is None
    assert response.diagnostics.reasons == ["policy0"]
    assert response.diagnostics.errors == []


def test_is_not_authorized() -> None:
    policies = """permit (
        principal == User::"alice",
        action == Action::"view",
        resource in Album::"jane_vacation"
    );
    """
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
    response = is_authorized(
        request=AuthorizationRequest(
            principal='User::"bob"',
            action='Action::"view"',
            resource='Photo::"VacationPhoto94.jpg"',
            context={"oidc_scope": "profile"},
        ),
        policies=Policies(policies),
        entities=Entities(entities),
    )
    assert response.decision == AuthorizationDecision.DENY
    assert response.correlation_id is None
    assert response.diagnostics.reasons == []
    assert response.diagnostics.errors == []
