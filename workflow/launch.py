#!/usr/bin/python3 -I
"""Staged launcher. Never uses the live Pi executable or live agent directory."""
import argparse
import fcntl
import json
import os
import re
import signal
import stat
import subprocess
import sys
import tempfile
from pathlib import Path
from urllib.parse import urlsplit

HERE = Path(__file__).resolve().parent
MODEL_ENV = ("ANTHROPIC_API_KEY", "OPENAI_API_KEY", "GEMINI_API_KEY", "GROQ_API_KEY", "MISTRAL_API_KEY",
             "OPENROUTER_API_KEY", "DEEPSEEK_API_KEY", "XAI_API_KEY")


def acquire(runtime):
    directory = os.open(runtime, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        details = os.fstat(directory)
        if details.st_uid != os.getuid() or details.st_mode & 0o077:
            raise ValueError("A private per-user /run/user directory is required")
        descriptor = os.open("pi-managed-workflow.lock", os.O_RDONLY | os.O_CREAT | os.O_NOFOLLOW | os.O_NONBLOCK,
                             0o600, dir_fd=directory)
    finally:
        os.close(directory)
    try:
        info = os.fstat(descriptor)
        if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or info.st_nlink != 1 or info.st_mode & 0o077:
            raise ValueError("Invalid workflow lock inode")
        fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        return descriptor
    except BaseException:
        os.close(descriptor)
        raise


def operator_args():
    parser = argparse.ArgumentParser(prog="pi --workflow", description="Managed Pi terminal with scoped Plan, Build, and Orchestrator workflows")
    parser.add_argument("workspace", help="directory of the project to work on")
    parser.add_argument("core", help="pinned provider/model id")
    parser.add_argument("ingestion", help="comma-separated eligible provider/model ids")
    parser.add_argument("--advisor", help="eligible stronger reasoning model; per-pass approval remains required")
    parser.add_argument("--research", action="store_true", help="allow evidence roles to use the public inputs below")
    parser.add_argument("--keenable", choices=["https://api.keenable.ai/mcp-public"],
                        help="keyless Keenable endpoint (the default with --research)")
    parser.add_argument("--public-query", help="public search terms for Keenable, Exa, and optional Context7")
    parser.add_argument("--public-url", help="public HTTPS documentation URL for Keenable and Exa fetch")
    parser.add_argument("--context7-library", help="Context7 library ID, optionally pinned to a version")
    args = parser.parse_args()
    if args.research:
        if not (args.public_query or args.public_url):
            parser.error("--research requires --public-query or --public-url")
        if args.public_query is not None and (not args.public_query.strip() or len(args.public_query) > 300 or
                                              not args.public_query.isprintable()):
            parser.error("--public-query must be 1 to 300 printable characters")
        if args.public_url is not None and not public_url(args.public_url):
            parser.error("--public-url must be a public HTTPS URL without credentials, query, or fragment")
        if args.context7_library is not None and (not args.public_query or not re.fullmatch(
                r"/[A-Za-z0-9_-][\w.-]{0,63}/[A-Za-z0-9_-][\w.-]{0,63}(?:/[A-Za-z0-9_-][\w.-]{0,63})?",
                args.context7_library, re.ASCII)):
            parser.error("--context7-library requires --public-query and a /namespace/library[/version] ID")
        args.keenable = args.keenable or "https://api.keenable.ai/mcp-public"
    elif any(value is not None for value in (args.keenable, args.public_query, args.public_url, args.context7_library)):
        parser.error("public research options require --research")
    return args


def public_url(value):
    if not re.fullmatch(r"https://[A-Za-z0-9.-]+(?:/[A-Za-z0-9._~/%+\-]*)?", value) or len(value) > 2048:
        return False
    host = urlsplit(value).hostname
    return bool(re.fullmatch(r"(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z]{2,63}", host)
                and host.rsplit(".", 1)[1] not in {"local", "localhost", "internal", "lan", "test", "invalid", "example"})


def workspace_path(value):
    source = Path(os.path.abspath(value))
    home = Path.home().resolve()
    resolved = source.resolve(strict=True)
    if len(source.parts) < 3 or source != resolved or source == home or source in home.parents:
        raise ValueError("Select a canonical, narrow project directory")
    if any(part.startswith(".") for part in source.parts[1:]) or source.parts[1] in ("run", "proc", "sys", "dev"):
        raise ValueError("Hidden and system runtime paths are excluded")
    if not source.is_dir() or source.stat().st_uid != os.getuid():
        raise ValueError("Select a user-owned directory")
    return str(source)


def run(args):
    source = workspace_path(args.workspace)
    runtime = f"/run/user/{os.getuid()}"
    fd = acquire(runtime)
    try:
        with tempfile.TemporaryDirectory(prefix="pi-managed-", dir=runtime) as root:
            paths = {name: str(Path(root, name)) for name in ("home", "agent", "tmp", "parent")}
            for path in paths.values():
                os.mkdir(path, 0o700)
            env = {"PATH": "/usr/bin", "LC_ALL": "C.UTF-8", "HOME": paths["home"],
                   "XDG_CONFIG_HOME": paths["home"], "XDG_DATA_HOME": paths["home"],
                   "XDG_CACHE_HOME": paths["home"], "XDG_RUNTIME_DIR": paths["tmp"],
                   "TMPDIR": paths["tmp"], "PI_CODING_AGENT_DIR": paths["agent"], "JITI_FS_CACHE": "false"}
            env.update({name: os.environ[name] for name in MODEL_ENV if name in os.environ})
            if args.context7_library and "CONTEXT7_API_KEY" in os.environ:
                env["CONTEXT7_API_KEY"] = os.environ["CONTEXT7_API_KEY"]
            research_inputs = json.dumps({"query": args.public_query, "url": args.public_url,
                                          "libraryId": args.context7_library})
            argv = ["/usr/bin/node", str(HERE / "sdk-host.mjs"), source, args.core, args.ingestion,
                    args.advisor or "", "yes" if args.research else "no", args.keenable or "", str(fd), paths["parent"],
                    research_inputs]
            with subprocess.Popen(argv, cwd=paths["parent"], env=env, pass_fds=(fd,), start_new_session=False) as child:
                # The node process handles SIGINT/TERM and drains. The check
                # supervisor also inherits the flock until its namespace exits.
                previous = {key: signal.signal(key, lambda signum, _frame: child.send_signal(signum))
                            for key in (signal.SIGTERM, signal.SIGHUP)}
                try:
                    while True:
                        try:
                            return child.wait()
                        except KeyboardInterrupt:
                            # The foreground process group has already received SIGINT.
                            continue
                finally:
                    for key, handler in previous.items():
                        signal.signal(key, handler)
    finally:
        os.close(fd)


if __name__ == "__main__":
    try:
        sys.exit(run(operator_args()))
    except BlockingIOError:
        print("Another managed Pi workflow holds the per-user lock; not queued.", file=sys.stderr)
        sys.exit(75)
    except (OSError, ValueError):
        print("Managed Pi launch rejected: check the workspace and private per-user runtime directory.", file=sys.stderr)
        sys.exit(1)
