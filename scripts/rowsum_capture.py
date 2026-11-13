from __future__ import annotations

import hashlib
import json
from pathlib import Path

from scripts.stop_capture import Invocation, Mode, Run, run
from scripts.stop_capture import invocation as base_invocation
from scripts.stop_reference import environment


def environment_hash() -> str:
    settings = dict(environment())
    _ = settings.pop("LC_ALL", None)
    settings.update(LC_CTYPE="en_US.UTF-8", LC_COLLATE="C")
    return hashlib.sha256(json.dumps(settings, sort_keys=True).encode()).hexdigest()


def with_locale(request: Invocation) -> Invocation:
    return Invocation(
        ("env", "-u", "LC_ALL", "LC_CTYPE=en_US.UTF-8", "LC_COLLATE=C", *request.command),
        request.stdin,
    )


def invocation(mode: Mode, executables: tuple[str, str], source: Path) -> Invocation:
    return with_locale(base_invocation(mode, executables, source))


def locale_profile() -> tuple[Run, ...]:
    program = 'cat(Sys.getlocale("LC_COLLATE"),Sys.getlocale("LC_CTYPE"),sep="|")'
    results: list[Run] = []
    for executable in ("R", "Rscript"):
        request = Invocation((executable, "--vanilla", "--slave", "-e", program), b"")
        result = run(with_locale(request))
        assert result.status == 0 and result.stdout == "C|en_US.UTF-8" and not result.stderr
        results.append(result)
    return tuple(results)
