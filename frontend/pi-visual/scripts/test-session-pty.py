#!/usr/bin/env python3
"""Normal P04.1 CLI + real Pi, isolated config, no prompts/provider credentials."""
import os,sys,pty,subprocess,select,time,fcntl,termios,struct,json,signal,tempfile,pathlib
from terminal_test_support import screen_text
exe=str(pathlib.Path('target/debug/pi-visual').resolve())
pi=str(pathlib.Path(sys.argv[1]).absolute())
with tempfile.TemporaryDirectory(prefix='pi-visual-session-pty-') as name:
 root=pathlib.Path(name)
 for directory in ['home','agent','agent/extensions','project','config','cache']:(root/directory).mkdir()
 (root/'agent/extensions/transitions.ts').write_text(pathlib.Path('tests/fixtures/session-transitions.ts').read_text())
 (root/'agent/extensions/ansi-status.ts').write_text(pathlib.Path('tests/fixtures/ansi-status.ts').read_text())
 # The ordinary startup path keeps discovery enabled; the only discovered code
 # in this private agent root is the reviewed, test-owned veto/status code.
 for file,identity,label in [('a.jsonl','fixture-a','A ACTIVE'),('b.jsonl','fixture-b','B ACTIVE')]:
  content='\n'.join(['DEMO scroll row '+str(i) for i in range(100)]+[label]) if identity=='fixture-a' else label
  records=[dict(type='session',version=3,id=identity,timestamp='2026-11-16T00:00:00Z',cwd=str(root/'project')),
   dict(type='message',id='a0000001',parentId=None,timestamp='2026-11-16T00:00:00Z',message=dict(role='user',content=content,timestamp=1794787200000))]
  if identity=='fixture-b':
   records.append(dict(type='message',id='b0000002',parentId='a0000001',timestamp='2026-11-16T00:00:01Z',message=dict(role='user',content='/quit',timestamp=1794787201000)))
  (root/file).write_text(''.join(json.dumps(r)+'\n' for r in records))
 env=dict(PATH='/usr/local/bin:/usr/bin:/bin',TERM='xterm-256color',HOME=str(root/'home'),PI_CODING_AGENT_DIR=str(root/'agent'),PI_OFFLINE='1',XDG_CONFIG_HOME=str(root/'config'),XDG_CACHE_HOME=str(root/'cache'))
 blocked=subprocess.run([exe,'--pi',pi,'--cwd',str(root/'agent')],env=env,capture_output=True)
 assert blocked.returncode!=0 and b'refusing Pi startup' in blocked.stderr
 missing=subprocess.run([exe,'--pi',str(root/'missing-pi'),'--cwd',str(root/'project')],env=env,capture_output=True)
 assert missing.returncode!=0 and b'Cannot launch Pi RPC' in missing.stderr
 master,slave=pty.openpty();before=termios.tcgetattr(slave)
 fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',24,100,0,0))
 process=subprocess.Popen([exe,'--pi',pi,'--cwd',str(root/'project')],stdin=slave,stdout=slave,stderr=slave,env=env)
 output=bytearray()
 def wait_for(predicate):
  end=time.monotonic()+12
  while time.monotonic()<end:
   if select.select([master],[],[],.03)[0]:
    try:output.extend(os.read(master,65536))
    except OSError:break
   screen=screen_text(output)
   if predicate(screen):return screen
  raise AssertionError('expected screen not reached; exit='+str(process.poll())+'; screen='+repr(screen_text(output)))
 def send(text):os.write(master,text)
 def resume(file,identity,label):
  send(('\x15/resume '+str(root/file)+'\r').encode())
  return wait_for(lambda s:identity in s.splitlines()[0] and label in s)
 def child_pid():
  children=pathlib.Path(f'/proc/{process.pid}/task/{process.pid}/children').read_text().split()
  assert len(children)==1,children
  return int(children[0])
 try:
  s=wait_for(lambda s:'RPC ready' in s and 'DEMO lens ready' in s and 'DEMO extmgr latest' in s)
  assert s.splitlines()[0].startswith('RPC ready') and 'Cost — · Context —' in s.splitlines()[-1]
  assert '> ' in s.splitlines()[-3],'basic input is not in its reserved region'
  assert '[38;' not in s and '[39m' not in s and 'DEMO obsolete' not in s
  transcript='\n'.join(s.splitlines()[1:20])
  assert transcript.count('DEMO extmgr latest')==1 and transcript.count('DEMO lens ready')==1
  resume('a.jsonl','fixture-a','A ACTIVE')
  send(b'\x1b[5~')
  scrolled=wait_for(lambda s:'DEMO scroll row' in s and 'A ACTIVE' not in s)
  send(b'\x1b[<64;5;5M')
  wait_for(lambda s:s.splitlines()[1]!=scrolled.splitlines()[1] and 'DEMO scroll row' in s)
  send(b'\x1b[1;5F')
  wait_for(lambda s:'A ACTIVE' in s and 'DEMO lens ready' in s)
  send(b'\x1b[17~') # F6: transcript focus
  send(b'x') # must not insert while transcript-focused
  send(b'\x1b[17~local draft') # F6 then type, never submit
  s=wait_for(lambda s:'local draft' in s)
  assert 'xlocal draft' not in s
  original_child=child_pid()
  s=resume('b.jsonl','fixture-b','B ACTIVE');assert 'A ACTIVE' not in s
  s=wait_for(lambda s:'/quit' in s and 'B ACTIVE' in s)
  assert process.poll() is None and child_pid()==original_child,'historical /quit executed'
  assert '[38;' not in s and '[39m' not in s
  send(('\x15/resume '+str(root/'veto.jsonl')+'\r').encode())
  s=wait_for(lambda s:'Session unchanged' in s);assert 'B ACTIVE' in s and 'A ACTIVE' not in s
  # Staged navigation text is retained on refusal; unsent draft survives crash/restart.
  send(b'\x15unsent draft')
  wait_for(lambda s:'unsent draft' in s)
  old=child_pid();os.kill(old,signal.SIGKILL)
  wait_for(lambda s:'RPC disconnected' in s)
  send(b'\x1b[15~') # F5: explicit restart, never a prompt
  s=wait_for(lambda s:'RPC ready' in s and 'fixture-b' in s.splitlines()[0]);assert 'B ACTIVE' in s and 'unsent draft' in s
  assert child_pid()!=old and not pathlib.Path(f'/proc/{old}').exists()
  send(b'\x03');process.wait(timeout=8)
  assert process.returncode==0,process.returncode
  assert termios.tcgetattr(slave)==before,'raw terminal flags not restored'
  print('PASS normal CLI real Pi: ANSI status replacement, header/input/footer, keyboard/wheel/latest/focus, inert historical /quit; hydration, A/B switch, veto, no stale messages, draft, disconnect/F5 recovery, owned child reaping and terminal restoration; no provider credentials/prompts')
 finally:
  if process.poll() is None:process.terminate();process.wait(timeout=8)
  os.close(master);os.close(slave)
