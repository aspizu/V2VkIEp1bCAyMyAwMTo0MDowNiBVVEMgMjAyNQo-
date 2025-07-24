from __future__ import annotations

import asyncio

import shl
from rich import print


async def main() -> None:
    result = shl._parse_command(t"echo src/*")  # pyright: ignore[reportAttributeAccessIssue]
    print(result)


asyncio.run(main())
