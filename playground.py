from __future__ import annotations

import asyncio
from io import BytesIO

import shl
from rich import print


async def main() -> None:
    file = BytesIO()
    cmd = t"ls -la"
    print(await shl._execute_command(cmd))  # pyright: ignore[reportAttributeAccessIssue]  # noqa: SLF001
    print(file.read())


asyncio.run(main())
