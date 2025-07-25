from __future__ import annotations

import asyncio

import shl
from rich import print


async def main() -> None:
    result = await shl._execute_command(t"cat playground.py | wc -l")  # pyright: ignore[reportAttributeAccessIssue]
    print(result)


asyncio.run(main())
