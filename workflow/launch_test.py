import os
import subprocess
import sys
from tempfile import TemporaryDirectory
from unittest import TestCase, main

from launch import acquire


class LockTest(TestCase):
    def test_no_queue_and_descriptor_inheritance_hold_the_lock(self):
        with TemporaryDirectory() as runtime:
            descriptor = acquire(runtime)
            # The owning process can exit before its worker. The worker still
            # holds the same open file description and no new run can enter.
            supervisor = (
                "import subprocess, sys; "
                "subprocess.Popen([sys.executable, '-I', '-c', 'import sys; sys.stdin.read()'], "
                "pass_fds=(int(sys.argv[1]),))"
            )
            with subprocess.Popen([sys.executable, "-I", "-c", supervisor, str(descriptor)],
                                  stdin=subprocess.PIPE, stdout=subprocess.PIPE, pass_fds=(descriptor,)) as holder:
                try:
                    self.assertEqual(holder.wait(timeout=5), 0)
                    os.close(descriptor)
                    with self.assertRaises(BlockingIOError):
                        acquire(runtime)
                finally:
                    holder.communicate(timeout=5)
            descriptor = acquire(runtime)
            os.close(descriptor)


if __name__ == "__main__":
    main()
