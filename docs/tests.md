# Tests

This project passes the integration tests on the [cedar-integration-tests](https://github.com/cedar-policy/cedar-integration-tests) project.

The following parametrized test is used for all scenarios found within `cedar-integration-tests` corpus:

```py
@pytest.mark.parametrize(("test_scenario", "test_case"), TEST_CASES, ids=TEST_CASES_IDS)
def test_scenario(test_scenario: Scenario, test_case: Case) -> None:
    """Run a test case from a scenario provided by cedar-integration-tests corpus."""

    from cedar import Entities, PolicySet, Request, Schema, is_authorized

    # Load policies, schema and entities from files
    policies = PolicySet.from_string(test_scenario.policies.read_text())
    schema = Schema.from_string(test_scenario.schema.read_text())
    entities = Entities.from_json(test_scenario.entities.read_text())
    # Build request from test case attributes
    request = Request.from_dict(
        {
            "principal": test_case.principal,
            "action": test_case.action,
            "resource": test_case.resource,
            "context": test_case.context,
        }
    )
    # Evaluate request (only use schema if test case indicates that request should be validated)
    response = is_authorized(
        request,
        policies,
        entities,
        schema if test_case.validate_request else None,
    )
    # Check response
    assert str(response.decision) == test_case.decision  # "allow" or "deny"
    assert response.diagnostics.errors == test_case.errors
    assert response.diagnostics.reasons == set(test_case.reason)
```

Check out [e2e/test_corpus.py](https://github.com/charbonnierg/cedar-python/tree/main/e2e/test_corpus.py) file from the project repository.
