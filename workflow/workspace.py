"""Descriptor-relative access for the managed Pi workspace tools."""
import hashlib
import json
import os
import re
import stat
import sys
import tempfile
from contextlib import suppress

PROTECTED = re.compile(
    r"^(?:\.|(?:auth|oauth|mcp-auth|mcp|models(?:-store)?)\.json(?:\.|$)|"
    r"(?:package(?:-lock)?\.json|pnpm-lock\.yaml|yarn\.lock|bun\.lockb?|Cargo\.(?:toml|lock)|"
    r"pyproject\.toml|uv\.lock|requirements(?:-[a-z0-9_-]+)?\.txt|go\.(?:mod|sum)|AGENTS\.md|permission-mode\.json|policy\.json)$|"
    r"settings\.json(?:\.|$)|sandbox\.json$|keybindings\.json$|sessions$|"
    r"permission-mode$|pi-subagents$|extensions$|workflow$|agents$|bin$|install$|npm$|node_modules$|mcp\.log|id_rsa|id_ed25519|known_hosts)"
    r"|(?:credential|secret|token)|\.(?:key|p12|pem|pfx|crt)$", re.I
)
SEGMENT = re.compile(r"[a-zA-Z0-9_][a-zA-Z0-9_. @+-]*")
DIRECTORY = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
LIMIT = 262144


def safe_parts(path):
    parts = path.split("/")
    if len(path) > 512 or any(not SEGMENT.fullmatch(p) or PROTECTED.search(p) for p in parts):
        raise ValueError("Path is outside the permitted workspace surface")
    return parts


def mount_id(fd):
    try:
        with open(f"/proc/self/fdinfo/{fd}", encoding="ascii") as stream:
            for line in stream:
                if line.startswith("mnt_id:"):
                    return int(line.partition(":")[2])
    except (OSError, ValueError) as error:
        raise ValueError("Mount identity unavailable") from error
    raise ValueError("Mount identity unavailable")


def open_directory(root, inside=()):
    if not root.startswith("/") or root in ("/", "//") or os.path.normpath(root) != root:
        raise ValueError("Expected a canonical absolute workspace root")
    base = os.open("/", DIRECTORY)
    try:
        workspace = descend(base, root[1:].split("/"), same_mount=False)
    finally:
        os.close(base)
    try:
        return descend(workspace, inside)
    finally:
        os.close(workspace)


def descend(base, parts, same_mount=True):
    directory = os.dup(base)
    try:
        allowed_mount = mount_id(directory) if same_mount else None
        for part in parts:
            child = os.open(part, DIRECTORY, dir_fd=directory)
            os.close(directory)
            directory = child
            if allowed_mount is not None and mount_id(directory) != allowed_mount:
                raise ValueError("Nested mount denied")
        return directory
    except BaseException:
        os.close(directory)
        raise


def read_file(directory, name):
    # Inspect a pinned inode before opening it for data. O_PATH neither follows
    # the final symlink nor opens a FIFO or device for I/O.
    fd = os.open(name, os.O_PATH | os.O_NOFOLLOW, dir_fd=directory)
    try:
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or info.st_size > LIMIT or mount_id(fd) != mount_id(directory):
            raise ValueError("Expected a small, unlinked regular file")
        with open(f"/proc/self/fd/{fd}", "rb") as stream:
            data = stream.read(LIMIT + 1)
            after = os.fstat(stream.fileno())
        if len(data) > LIMIT or any(getattr(info, field) != getattr(after, field) for field in (
            "st_size", "st_mode", "st_nlink", "st_ctime_ns", "st_mtime_ns",
        )):
            raise ValueError("File changed during read or exceeds the read limit")
        return data, info
    finally:
        os.close(fd)


def change_file(directory, name, expected, content):
    """Staging-copy helper only: the caller must own an exclusive private copy."""
    data = content.encode("utf-8")
    if len(data) > LIMIT:
        raise ValueError("File exceeds write limit")
    try:
        previous, info = read_file(directory, name)
        actual = hashlib.sha256(previous).hexdigest()
    except FileNotFoundError:
        actual, info = None, None
    if actual != expected:
        raise ValueError("File changed since approval")
    fd, temporary = tempfile.mkstemp(prefix=".pi-stage-", dir=f"/proc/self/fd/{directory}")
    try:
        with os.fdopen(fd, "wb") as stream:
            if info:
                os.fchmod(stream.fileno(), info.st_mode & 0o777)
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        if info is None:
            os.link(temporary, name, dst_dir_fd=directory, follow_symlinks=False)
        else:
            current, latest = read_file(directory, name)
            if latest.st_ino != info.st_ino or hashlib.sha256(current).hexdigest() != expected:
                raise ValueError("File changed during edit")
            os.replace(temporary, name, dst_dir_fd=directory)
        return hashlib.sha256(data).hexdigest()
    finally:
        with suppress(FileNotFoundError):
            os.unlink(temporary)


def main():
    operation, root, path = sys.argv[1:]
    *parents, name = safe_parts(path)
    if operation != "read":
        raise ValueError("Live writes are unavailable without a safe publication boundary")
    directory = open_directory(root, parents)
    try:
        data, _ = read_file(directory, name)
        return {"text": data.decode("utf-8"), "sha256": hashlib.sha256(data).hexdigest()}
    finally:
        os.close(directory)


if __name__ == "__main__":
    try:
        print(json.dumps(main()))
    except FileNotFoundError:
        print(json.dumps({"missing": True}))
    except (OSError, ValueError):
        print(json.dumps({"error": "Workspace operation rejected"}))
        sys.exit(1)
