#!/usr/bin/env python3
"""Actual terminal input smoke test. Uses only the explicit isolated UI fixture."""
import os,sys,pty,subprocess,select,time,fcntl,termios,struct
master,slave=pty.openpty()
fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',24,100,0,0))
p04='--p04' in sys.argv
args=['./target/debug/pi-visual','--rpc-fixture',sys.argv[1]] if p04 else ['./target/debug/pi-visual','--demo','dialogs',sys.argv[1]]
process=subprocess.Popen(args,stdin=slave,stdout=slave,stderr=slave,env={'PATH':'/usr/local/bin:/usr/bin:/bin','TERM':'xterm-256color'})
os.close(slave)
output=bytearray()
from terminal_test_support import screen_text as project_screen
def screen_text():return project_screen(output)
def wait_for(token):
 end=time.monotonic()+12
 while time.monotonic()<end:
  if select.select([master],[],[],.1)[0]:
   try: output.extend(os.read(master,65536))
   except OSError: break
  if token.decode() in screen_text(): return
 raise AssertionError('terminal did not show '+repr(token)+'; exit='+str(process.poll())+'; terminal tail='+repr(bytes(output[-2000:])))
try:
 if p04:
  wait_for(b'ready');os.write(master,b'/p03-dialogs\r')
 wait_for(b'P03 demo: confirmation');os.write(master,b'n')
 wait_for(b'selection');os.write(master,b'\x1b[B\x1b[B\r')
 wait_for(b'input');os.write(master,b'typed\r')
 wait_for(b'editor');os.write(master,b'\x1bOQ') # F2 submits the prefilled editor
 wait_for(b'confirmed');wait_for(b'false');wait_for(b'second');wait_for(b'typed')
 if not p04:wait_for(b'Preserved local draft')
 os.write(master,b'\x03');process.wait(timeout=5)
 assert process.returncode==0,process.returncode
 print('PASS: real Pi four-dialog PTY keyboard flow, explicit denial, selection, input/editor and draft restoration')
finally:
 if process.poll() is None: process.kill();process.wait()
 os.close(master)
