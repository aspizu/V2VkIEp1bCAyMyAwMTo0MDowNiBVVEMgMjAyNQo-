from __future__ import annotations

import shl
from rich import print

result = shl._execute_command(t"echo {'Hello, World!'}")  # pyright: ignore[reportAttributeAccessIssue]
print(result)
