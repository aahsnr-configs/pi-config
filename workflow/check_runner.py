"""Finite checks in a read-only filtered copy. No host fallback."""
import json
import signal
import subprocess
import sys
from tempfile import TemporaryDirectory

from private_workspace import manifest, populate, scan

CHECKS = {
    "node-tests": ("/usr/bin/node", "--test"),
    "python-tests": ("/usr/bin/python3", "-I", "-B", "-m", "unittest", "discover"),
}
ENV = {"PATH": "/usr/bin", "LC_ALL": "C.UTF-8"}
SANDBOX_SCRATCH = "/check-scratch"


def bubblewrap(root, argv, hold_lock):
    retention = ["--sync-fd", "3"] if hold_lock else []
    return ["/usr/bin/bwrap", *retention, "--unshare-all", "--unshare-user", "--unshare-net",
            "--new-session", "--die-with-parent", "--cap-drop", "ALL",
            "--clearenv", "--setenv", "PATH", "/usr/bin", "--setenv", "HOME", SANDBOX_SCRATCH,
            "--setenv", "TMPDIR", SANDBOX_SCRATCH,
            "--ro-bind", "/usr/bin", "/usr/bin", "--ro-bind", "/usr/lib", "/usr/lib",
            "--ro-bind", "/usr/lib64", "/usr/lib64", "--symlink", "usr/bin", "/bin",
            "--symlink", "usr/lib", "/lib", "--symlink", "usr/lib64", "/lib64",
            "--dev", "/dev", "--proc", "/proc", "--tmpfs", SANDBOX_SCRATCH,
            "--ro-bind", root, "/work", "--chdir", "/work", "--", *argv]


def execute(root, argv, seconds, hold_lock, diagnostic=False):
    # Retain the launcher's flock with both the supervisor and Bubblewrap's
    # lifetime-only fd, never in the checked command's tool interface.
    with subprocess.Popen(bubblewrap(root, argv, hold_lock), env=ENV, stdin=subprocess.DEVNULL,
                          stdout=subprocess.DEVNULL, stderr=None if diagnostic else subprocess.DEVNULL,
                          pass_fds=(3,) if hold_lock else ()) as child:
        try:
            return child.wait(timeout=seconds)
        finally:
            if child.poll() is None:
                child.kill()
                child.wait()


def interrupt(_signum, _frame):
    for key in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
        signal.signal(key, signal.SIG_IGN)
    raise InterruptedError


def probe():
    # Operator-only probe: the same isolation profile with an empty fixture,
    # no provider, and no test or project code.
    try:
        with TemporaryDirectory(prefix="managed-probe-") as copy:
            available = execute(copy, ("/usr/bin/true",), 5, False, diagnostic=True) == 0
    except (OSError, subprocess.TimeoutExpired):
        available = False
    if available:
        print("Bubblewrap profile available")
        return 0
    print("NOT TESTED: Bubblewrap profile unavailable; no host fallback", file=sys.stderr)
    return 1


def main():
    if sys.argv[1:] == ["--probe"]:
        sys.exit(probe())
    source, scratch, check, expected, *flags = sys.argv[1:]
    if flags not in ([], ["--hold-lock"]):
        raise ValueError("Invalid check supervisor flags")
    hold_lock = bool(flags)
    argv = CHECKS[check]
    status = "not-tested"
    for key in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
        signal.signal(key, interrupt)
    try:
        with TemporaryDirectory(prefix="isolated-", dir=scratch) as copy:
            contents = scan(source)
            if manifest(contents)["fingerprint"] != expected:
                raise ValueError("Check inputs changed before copying")
            populate(copy, contents)
            if execute(copy, ("/usr/bin/true",), 5, hold_lock) == 0:
                status = "passed" if execute(copy, argv, 90, hold_lock) == 0 else "failed"
    except (OSError, ValueError, subprocess.TimeoutExpired):
        status = "not-tested"
    print(json.dumps({"status": status}))


if __name__ == "__main__":
    main()
