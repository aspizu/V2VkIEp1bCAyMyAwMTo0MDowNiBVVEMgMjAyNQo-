from __future__ import annotations

import pytest
import shl


@pytest.mark.asyncio
async def test_sh() -> None:
    p = await shl.sh(t'echo "Hello, World!"')
    assert isinstance(p, shl.CompletedCommand)
    assert p.returncode == 0
