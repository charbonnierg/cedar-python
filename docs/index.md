# cedar python

## Example usage

This example assumes that:

- a [cedar policies file](https://docs.cedarpolicy.com/policies/syntax-policy.html) exists in working directory with the name `policies.cedar`
- a [cedar schema file](https://docs.cedarpolicy.com/schema/human-readable-schema.html) exists in working directory with the name `schema.cedar`

```py { .annotate }
from pathlib import Path
from cedar import Entities, PolicySet, Request, Schema, is_authorized, EntityUid

policies = PolicySet.from_string(Path("policies.cedar").read_text())  # (1)

schema = Schema.from_string(Path("schema.cedar").read_text())         # (2)

entities = Entities.from_list([                                       # (3)
    {
        "uid": {
            "type": "Group",
            "id": "admins"
        },
        "parents": [],
        "attrs": {}
    },
    {
        "uid": {
            "type": "User",
            "id": "alice",
        },
        "parents": [
            { "type": "Group", "id": "admin" }
        ],
        "attrs": {}
    },
])

request = Request(                                              # (4)
    principal=EntityUid.from_type_name_and_id("User", "alice"),
    action=EntityUid.from_type_name_and_id("Action", "doSomething"),
    resource=EntityUid.from_type_name_and_id("MyCustomResource", "some-unique-id"),
    context={"active": True},
)

response = is_authorized(request, policies, entities, schema)   # (5)
```

1. Parse policy set from file in Cedar format
2. Parse schema from file in Cedar format
3. Build entities to use with the requests
4. Build authorization request
5. Evaluate request and obtain authorization response
