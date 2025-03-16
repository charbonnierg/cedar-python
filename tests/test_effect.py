from cedar import Effect


def test_effect() -> None:
    assert Effect.Forbid is Effect.Forbid
    assert Effect.Forbid == Effect.Forbid
    assert Effect.Permit is Effect.Permit
    assert Effect.Permit == Effect.Permit
    assert Effect.Forbid is not Effect.Permit
    assert Effect.Forbid != Effect.Permit


def test_effect_str() -> None:
    assert str(Effect.Forbid) == "forbid"
    assert str(Effect.Permit) == "permit"


def test_effect_repr() -> None:
    assert repr(Effect.Permit) == "Effect.Permit"
    assert repr(Effect.Forbid) == "Effect.Forbid"
