#!/usr/bin/env python3
"""P04 actual terminal lifecycle checks; no Pi/model/network for offline runs."""
import os,sys,pty,subprocess,select,time,fcntl,termios,struct,json,signal,tempfile,pathlib
ENV={'PATH':'/usr/local/bin:/usr/bin:/bin','TERM':'xterm-256color'}
def exercise(args,steps,expected=0):
 master,slave=pty.openpty();before=termios.tcgetattr(slave)
 fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',24,100,0,0))
 process=subprocess.Popen(args,stdin=slave,stdout=slave,stderr=slave,env=ENV);output=bytearray()
 def read_for(seconds):
  end=time.monotonic()+seconds
  while time.monotonic()<end:
   if select.select([master],[],[],.02)[0]:
    try:output.extend(os.read(master,65536))
    except OSError:break
 try:
  read_for(.4)
  for data in steps:
   if data=='resize':
    fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',12,40,0,0));process.send_signal(signal.SIGWINCH)
   elif data=='terminate':process.terminate()
   elif data:os.write(master,data)
   read_for(.1)
  process.wait(timeout=8);read_for(.1)
  assert process.returncode==expected,(process.returncode,bytes(output[-1000:]))
  assert termios.tcgetattr(slave)==before,'terminal flags were not restored'
  if b'\x1b[?1049h' in output:
   assert b'\x1b[?1049l' in output,'alternate screen not restored'
   assert b'\x1b[?1006l' in output,'mouse reporting not disabled'
   assert b'\x1b[?2004l' in output,'paste mode not disabled'
   assert b'\x1b[<1u' in output,'keyboard enhancement not popped'
   assert b'\x1b[?25h' in output,'cursor not restored'
  return bytes(output)
 finally:
  if process.poll() is None:process.kill();process.wait()
  os.close(master);os.close(slave)
exe='./target/debug/pi-visual'
out=exercise([exe,'--demo','transcript'],[b'draft',b'\x1b[5~',b'\x1b[<64;2;3M',b'\x1b[<0;2;3M',b'\x1b[<32;4;5M',b'\x1b[<0;4;5m','resize',b'\x1bOQ',b'\x1bOS',b'\x19',b'\x1b',b'\x1bOS',b'\x1b',b'\x03'])
assert b'OFFLINE' in out and b'DEMO permission' in out,'offline viewport/modal not displayed'
exercise([exe,'--demo','transcript'],['terminate'])
with tempfile.TemporaryDirectory(prefix='pi-visual-replay-error-') as root:
 path=pathlib.Path(root)/'bad.jsonl';path.write_text('invalid\n')
 exercise([exe,'--replay',str(path)],[],expected=1)
 fifo=pathlib.Path(root)/'pipe.jsonl';os.mkfifo(fifo)
 exercise([exe,'--replay',str(fifo)],[],expected=1)
 saved=pathlib.Path(root)/'saved.jsonl'
 saved.write_text('\n'.join(json.dumps(r) for r in [
  dict(type='response',id='state',command='get_state',success=True,data=dict(sessionId='saved-fixture')),
  dict(type='response',id='history',command='get_entries',success=True,data=dict(entries=[dict(type='message',id='entry-1',parentId=None,message=dict(role='assistant',content=[dict(type='text',text='SAVED HISTORY FIXTURE')]))],leafId='entry-1')),
 ])+'\n')
 restored=exercise([exe,'--replay',str(saved)],[b'\x03'])
 assert b'SAVED HISTORY FIXTURE' in restored,'durable get_entries replay not displayed'
# Cargo locates the test binary; no production panic hook/flag added.
run=subprocess.run(['cargo','test','-p','pi-visual','--no-run','--message-format=json'],capture_output=True,text=True,check=True)
artifacts=[json.loads(line) for line in run.stdout.splitlines() if line.startswith('{')]
testexe=next(a['executable'] for a in artifacts if a.get('reason')=='compiler-artifact' and a.get('profile',{}).get('test') and a.get('executable'))
exercise([testexe,'--ignored','--exact','terminal::tests::panic_restores_modes','--nocapture'],[])
exercise([testexe,'--ignored','--exact','shell::regression_tests::panic_with_child_and_reader_is_contained_and_joined','--nocapture'],[])
print('PASS P04 PTY: offline viewport, mouse/resize/stream/modal input, Ctrl+C, SIGTERM, invalid replay and panic cleanup; termios/cursor/alternate/mouse/paste/keyboard restored')
