import hashlib
import subprocess
import sys
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest import TestCase, main

import workspace


class WorkspaceTest(TestCase):
    def test_scope_links_and_stale_writes(self):
        with TemporaryDirectory() as root:
            source = Path(root, "source.txt")
            source.write_text("before")
            Path(root, "alias.txt").symlink_to(source)
            Path(root, "hard.txt").hardlink_to(source)
            with self.assertRaises(ValueError):
                workspace.safe_parts(".env.fixture")
            workspace.os.mkfifo(Path(root, "pipe.txt"))
            directory = workspace.open_directory(root)
            try:
                for name in ("alias.txt", "pipe.txt"):
                    with self.assertRaises(ValueError):
                        workspace.read_file(directory, name)
                with self.assertRaises(ValueError):
                    workspace.read_file(directory, "hard.txt")
                Path(root, "hard.txt").unlink()
                with self.assertRaises(ValueError):
                    workspace.change_file(directory, "source.txt", "stale", "bad")
                before = hashlib.sha256(b"before").hexdigest()
                workspace.change_file(directory, "source.txt", before, "after")
                self.assertEqual(source.read_text(), "after")
                workspace.change_file(directory, "new.txt", None, "new")
                with self.assertRaises(ValueError):
                    workspace.change_file(directory, "new.txt", None, "bad")
            finally:
                workspace.os.close(directory)

    def test_symlink_in_root_or_inside_is_rejected(self):
        with TemporaryDirectory() as root:
            Path(root, "real", "nested").mkdir(parents=True)
            Path(root, "alias").symlink_to(Path(root, "real"), target_is_directory=True)
            with self.assertRaises(OSError):
                workspace.open_directory(str(Path(root, "alias", "nested")))
            with self.assertRaises(OSError):
                workspace.open_directory(root, ("alias", "nested"))

    def test_live_write_command_is_closed(self):
        with TemporaryDirectory() as root:
            source = Path(root, "source.txt")
            source.write_text("before")
            result = subprocess.run(
                [sys.executable, workspace.__file__, "write", root, "source.txt"],
                input='{"text":"after","expected":null}', text=True, capture_output=True, check=False,
            )
            self.assertEqual(result.returncode, 1)
            self.assertEqual(source.read_text(), "before")


if __name__ == "__main__":
    main()
