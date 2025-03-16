import json

from cedar import Effect, Policy

raw_policy = """permit (
  principal,
  action == Action::"edit",
  resource
)
when { (resource["owner"]) == principal };
"""
raw_policy_with_id = """@id("my-policy")
permit (
  principal,
  action == Action::"edit",
  resource
)
when { (resource["owner"]) == principal };
"""
dict_policy = {
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
json_policy = json.dumps(dict_policy, separators=(",", ":"))


def test_from_str() -> None:
    policy = Policy.from_string(raw_policy)
    assert policy.to_string() == raw_policy


def test_from_json() -> None:
    policy = Policy.from_json(json_policy)
    assert policy.to_pretty_string() == raw_policy


def test_from_dict() -> None:
    policy = Policy.from_dict(dict_policy)
    assert policy.to_pretty_string() == raw_policy


def test_to_json() -> None:
    policy = Policy.from_string(raw_policy)
    assert policy.to_json() == json_policy


def test_effect_property() -> None:
    policy = Policy.from_string(raw_policy)
    assert policy.effect == Effect.Permit


def test_to_dict() -> None:
    policy = Policy.from_string(raw_policy)
    assert policy.to_dict() == dict_policy


def test_default_policy_id() -> None:
    policy = Policy.from_string(raw_policy)
    assert policy.policy_id == "policy0"


def test_policy_id() -> None:
    policy = Policy.from_string(raw_policy_with_id)
    assert policy.policy_id == "my-policy"
