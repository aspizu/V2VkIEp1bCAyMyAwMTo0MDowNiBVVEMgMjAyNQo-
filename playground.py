from __future__ import annotations

import asyncio

import shl
from rich import print


async def main() -> None:
    result = shl._lex_command(t"echo {'Hello, World!'}")  # pyright: ignore[reportAttributeAccessIssue]
    print(result)


asyncio.run(main())
