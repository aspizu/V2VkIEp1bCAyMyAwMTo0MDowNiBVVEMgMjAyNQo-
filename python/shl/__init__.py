from __future__ import annotations

from typing import TYPE_CHECKING, cast

from .shl import *  # noqa: F403

if TYPE_CHECKING:
    from collections.abc import Generator
    from string.templatelib import Template

__doc__ = shl.__doc__
if hasattr(shl, "__all__"):
    __all__ = shl.__all__  # pyright: ignore[reportUnsupportedDunderAll, reportAttributeAccessIssue]


class Command[T = Command]:
    def __init__(self, command: Template) -> None:
        self._command: Template = command
        self._quiet: bool = False
        "If set, the command will not print its output to stdout/stderr."
        self._text: str | None = None
        "If set, its the encoding for capturing stdout."

    def text(self, encoding: str = "utf-8") -> Command[str]:
        """Returns the stdout of the command as a string when awaited."""
        self._text = encoding
        self._quiet = True  # text implies quiet
        return cast("Command[str]", self)

    def bytes(self) -> Command[bytes]:
        """Returns the stdout of the command as bytes when awaited."""
        self._text = "[bytes]"
        self._quiet = True
        return cast("Command[bytes]", self)

    def quiet(self) -> Command[T]:
        """Don't display the command output in the terminal."""
        self._quiet = True
        return self

    def __await__(self) -> Generator[None, None, CompletedCommand]:
        return _execute_command(self).__await__()  # pyright: ignore[reportUndefinedVariable]


class CompletedCommand:
    def __init__(self, returncode: int = 0) -> None:
        self.returncode: int = returncode


def sh(command: Template) -> Command:
    """Create a command to be executed."""
    return Command(command)
