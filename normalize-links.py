#!/usr/bin/env python3
"""Normalize owned Stow links after a checkout moves behind a path alias."""
import os
from pathlib import Path
import subprocess
import sys


def normalize(repo: Path, home: Path) -> None:
    home = home.resolve()
    for package, target in (("xdg-configs", home / ".config"),
                            ("cargo-config", home / ".cargo"),
                            ("bin", home / "bin")):
        source = repo / package
        files = subprocess.check_output(
            ["git", "-C", str(repo), "ls-files", "-z", "--cached", "--others",
             "--exclude-standard", "--", package]
        ).decode().split("\0")
        entries = set()
        for name in filter(None, files):
            relative = Path(name).relative_to(package)
            entries.add(relative)
            entries.update(parent for parent in relative.parents if parent != Path("."))
        for relative in sorted(entries, key=lambda path: (len(path.parts), str(path))):
            destination = target / relative
            expected = source / relative
            if destination.parent.resolve() == expected.parent.resolve():
                continue
            if destination.is_symlink() and destination.resolve() == expected.resolve():
                link = os.path.relpath(expected, destination.parent)
                if os.readlink(destination) != link:
                    destination.unlink()
                    destination.symlink_to(link, target_is_directory=expected.is_dir())


if __name__ == "__main__":
    normalize(Path(__file__).resolve().parent, Path(sys.argv[1]))
