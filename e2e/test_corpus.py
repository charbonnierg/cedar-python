import json
import shutil
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Literal

import pytest

# Expect corpus to be located next to the test file
CORPUS_ARCHIVE = (
    Path(__file__).parent / "cedar-integration-tests" / "corpus-tests.tar.gz"
)


@dataclass
class Case:
    """Each case has the following fields:

    description: Description for the request
    principal: Principal for the request (optional)
    action: Action for the request (optional)
    resource: Resource for the request (optional)
    context: Context for the request
    validateRequest: Whether to enable request validation (true/false)
    decision: Expected decision (Allow/Deny)
    reason: Expected policies that led to the decision
    errors: Expected policies that resulted in errors
    """

    description: str
    principal: dict[str, Any]
    action: dict[str, Any]
    resource: dict[str, Any]
    context: dict[str, Any]
    validate_request: bool
    decision: Literal["allow", "deny"]
    reason: list[str]
    errors: list[str]

    @classmethod
    def from_dict(cls, values: dict[str, Any]) -> "Case":
        values = values.copy()
        values["validate_request"] = values["validateRequest"]
        del values["validateRequest"]
        return Case(**values)


@dataclass
class Scenario:
    id: str
    policies: Path
    entities: Path
    schema: Path
    should_validate: bool
    requests: list[Case]

    @classmethod
    def from_json_file(cls, filepath: Path) -> "Scenario":
        directory = filepath.parent
        data = json.loads(filepath.read_text())
        return Scenario(
            id=filepath.stem,
            policies=directory.joinpath(Path(data["policies"]).name),
            entities=directory.joinpath(Path(data["entities"]).name),
            schema=directory.joinpath(Path(data["schema"]).name),
            should_validate=data["shouldValidate"],
            requests=[Case.from_dict(request) for request in data["requests"]],
        )


def get_test_cases(archive: Path) -> tuple[list[tuple[Scenario, Case]], list[str]]:
    # Uncompress corpus if not present
    target = Path(__file__).parent / "corpus-tests"
    if not target.is_dir():
        shutil.unpack_archive(filename=archive, extract_dir=target.parent)
    # Discover all scenarios
    scenarios = [
        Scenario.from_json_file(path)
        for path in target.glob("*.json")
        if not path.name.endswith(".entities.json")
    ]
    # Build all cases with their id
    cases = [
        (scenario, request) for scenario in scenarios for request in scenario.requests
    ]
    cases_ids = [
        f"{scenario.id}-{request.description}" for (scenario, request) in cases
    ]
    return cases, cases_ids


# Load all test cases and ids
TEST_CASES, TEST_CASES_IDS = get_test_cases(CORPUS_ARCHIVE)


@pytest.mark.parametrize(("test_scenario", "test_case"), TEST_CASES, ids=TEST_CASES_IDS)
def test_scenario(test_scenario: Scenario, test_case: Case) -> None:
    """Run a test case from a scenario provided by cedar-integration-tests corpus."""

    from cedar import Entities, PolicySet, Request, Schema, is_authorized

    # Load policies, schema and entities
    policies = PolicySet.from_string(test_scenario.policies.read_text())
    schema = Schema.from_string(test_scenario.schema.read_text())
    entities = Entities.from_json(test_scenario.entities.read_text())
    # Build request
    request = Request.from_dict(
        {
            "principal": test_case.principal,
            "action": test_case.action,
            "resource": test_case.resource,
            "context": test_case.context,
        }
    )
    # Check authorization
    response = is_authorized(
        request,
        policies,
        entities,
        schema if test_case.validate_request else None,
    )
    # Expect decision
    assert str(response.decision) == test_case.decision  # "allow" or "deny"
    # Check errors found in diagnostics
    assert response.diagnostics.errors == test_case.errors
    # Check reasons found in diagnostics
    assert response.diagnostics.reasons == set(test_case.reason)
