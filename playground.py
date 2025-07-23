from __future__ import annotations

import shl
from rich import print  # noqa: A004

command = 'echo    "hello world"'
result: str = shl._parse_command(command)  # noqa: SLF001  # pyright: ignore[reportAttributeAccessIssue]
print(result)
