from __future__ import annotations

import asyncio

import shl
from rich import print


async def main() -> None:
    result = await shl._execute_command(t"echo output is $(python -c 'print(input())')")  # pyright: ignore[reportAttributeAccessIssue]
    print(result)


asyncio.run(main())
