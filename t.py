import typing as t

@t.overload
def a(b: t.Literal[True]) -> str: ...

@t.overload
def a(b: t.Literal[False]) -> str | None: ...

def a(b: bool) -> str | None:
    if b:
        return "s"
    return None
