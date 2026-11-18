"""Trusted host only: bounded manifest, private snapshot, and patch operations."""
import difflib
import hashlib
import json
import os
import stat
import sys
from collections.abc import Iterator
from contextlib import ExitStack, suppress

from workspace import (
    DIRECTORY,
    LIMIT,
    change_file,
    descend,
    mount_id,
    open_directory,
    read_file,
    safe_parts,
)

MAX_ENTRIES = 4096
MAX_BYTES = 12 * 1024 * 1024


def filtered_files(directory, prefix, budget, device, mount) -> Iterator[tuple[str, str, int]]:
    with os.scandir(directory) as entries:
        for entry in entries:
            name = entry.name
            path = f"{prefix}{name}"
            try:
                safe_parts(path)
            except ValueError:
                continue
            budget[0] += 1
            if budget[0] > MAX_ENTRIES:
                raise ValueError("Workspace exceeds scan limit")
            info = entry.stat(follow_symlinks=False)
            if info.st_dev != device:
                continue
            if stat.S_ISDIR(info.st_mode):
                child = os.open(name, DIRECTORY, dir_fd=directory)
                try:
                    opened = os.fstat(child)
                    if (opened.st_dev, opened.st_ino) != (info.st_dev, info.st_ino):
                        raise ValueError("Directory changed during scan")
                    if mount_id(child) == mount:
                        yield from filtered_files(child, path + "/", budget, device, mount)
                finally:
                    os.close(child)
            elif stat.S_ISREG(info.st_mode) and info.st_size <= LIMIT and info.st_nlink == 1:
                data, pinned = read_file(directory, name)
                if (pinned.st_dev, pinned.st_ino) != (info.st_dev, info.st_ino):
                    raise ValueError("File changed during scan")
                budget[1] += len(data)
                if budget[1] > MAX_BYTES:
                    raise ValueError("Workspace exceeds byte limit")
                try:
                    text = data.decode("utf-8")
                except UnicodeDecodeError:
                    continue
                if "\0" not in text:
                    yield path, text, pinned.st_mode & 0o111


def scan(root):
    fd = open_directory(root)
    try:
        return {path: (text, mode) for path, text, mode in filtered_files(fd, "", [0, 0], os.fstat(fd).st_dev, mount_id(fd))}
    finally:
        os.close(fd)


def manifest(files):
    items = {path: {"sha256": hashlib.sha256(text.encode()).hexdigest(), "executable": bool(mode)}
             for path, (text, mode) in sorted(files.items())}
    return {"files": items, "fingerprint": hashlib.sha256(json.dumps(items).encode()).hexdigest()}


def private_directory(root):
    fd = open_directory(root)
    info = os.fstat(fd)
    if info.st_uid == os.getuid() and info.st_mode & 0o077 == 0:
        return fd
    os.close(fd)
    raise ValueError("Expected an owner-only private directory")


def populate(root, files):
    fd = private_directory(root)
    try:
        if os.listdir(fd):
            raise ValueError("Expected an empty private destination")
        for path, (text, mode) in sorted(files.items()):
            *parents, name = safe_parts(path)
            with ExitStack() as stack:
                directory = fd
                for parent in parents:
                    with suppress(FileExistsError):
                        os.mkdir(parent, 0o700, dir_fd=directory)
                    directory = os.open(parent, DIRECTORY, dir_fd=directory)
                    stack.callback(os.close, directory)
                out = os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                              0o700 if mode else 0o600, dir_fd=directory)
                with os.fdopen(out, "wb") as stream:
                    stream.write(text.encode())
    finally:
        os.close(fd)


def newline_lines(text):
    *lines, tail = text.split("\n")
    return [line + "\n" for line in lines] + ([tail] if tail else [])


def patch_for(before, after, paths):
    chunks = []
    for path in paths:
        if path not in after:
            raise ValueError("The patch surface does not support deletion")
        old, old_mode = before.get(path, ("", 0))
        new, new_mode = after[path]
        if bool(old_mode) != bool(new_mode):
            raise ValueError("The patch surface does not support executable bit changes")
        if path not in before:
            chunks.extend([f"diff --git a/{path} b/{path}\n", "new file mode 100644\n"])
        lines = difflib.unified_diff(newline_lines(old), newline_lines(new),
                                     "/dev/null" if path not in before else f"a/{path}", f"b/{path}", lineterm="\n")
        for line in lines:
            chunks.append(line if line.endswith("\n") else line + "\n\\ No newline at end of file\n")
    result = "".join(chunks)
    if len(result.encode()) > 512 * 1024:
        raise ValueError("Patch exceeds output limit")
    return result


def run(argv):
    operation, root, *extra = argv
    if operation == "manifest" and not extra:
        return manifest(scan(root))
    if operation == "copy" and len(extra) == 1:
        contents = scan(root)
        populate(extra[0], contents)
        return manifest(contents)
    if operation == "write" and len(extra) == 1:
        *parents, name = safe_parts(extra[0])
        private = private_directory(root)
        try:
            fd = descend(private, parents)
            try:
                req = json.load(sys.stdin)
                return {"sha256": change_file(fd, name, req["expected"], req["text"])}
            finally:
                os.close(fd)
        finally:
            os.close(private)
    if operation == "patch" and len(extra) == 1:
        before, after = scan(root), scan(extra[0])
        paths = sorted(path for path in before.keys() | after.keys() if before.get(path) != after.get(path))
        return {"paths": paths, "patch": patch_for(before, after, paths)}
    raise ValueError("Unknown private workspace operation")


if __name__ == "__main__":
    try:
        print(json.dumps(run(sys.argv[1:])))
    except (OSError, ValueError, TypeError, KeyError, RecursionError):
        print('{"error":"Private workspace operation rejected"}')
        sys.exit(1)
