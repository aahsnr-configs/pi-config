"""Capture only our temporary Kitty window; never a desktop-sized screenshot."""
from pathlib import Path
import json
import subprocess
import time

evidence = Path(__file__).parent
control = Path('/tmp/opencode-ui-kitty')
control.mkdir(exist_ok=True)
for name in ['ready', 'state', 'exit', 'fatal', 'error']:
    (control / name).write_text('pending')
(control / 'command').write_text('home')
result = []
with (control / 'kitty-launch.log').open('w') as log:
    process = subprocess.Popen([
        'kitty', '--class', 'opencode-ui-check', '--title', 'Pi UI isolated test fixture',
        '--override', 'initial_window_width=100c', '--override', 'initial_window_height=30c',
        '--override', 'background_opacity=1.0',
        '--directory', '/tmp/opencode-ui-check', 'node', str(evidence / 'kitty-fixture.mjs'),
    ], stdout=log, stderr=log)
    try:
        for scenario in ['home', 'popup', 'file-popup', 'chat', 'dialog']:
            (control / 'command').write_text(scenario)
            deadline = time.monotonic() + 12
            while time.monotonic() < deadline:
                if (control / 'ready').read_text() == 'ready' and (control / 'state').read_text() == scenario:
                    break
                if process.poll() is not None:
                    raise RuntimeError(f'Kitty fixture exited {process.returncode}; state={scenario}')
                time.sleep(0.1)
            else:
                raise RuntimeError(f'Fixture timed out: {scenario}; fatal={(control / "fatal").read_text()}; error={(control / "error").read_text()}')
            time.sleep(0.4)
            clients = json.loads(subprocess.check_output(['hyprctl', '-j', 'clients']))
            matches = [client for client in clients if client.get('class') == 'opencode-ui-check']
            if len(matches) != 1:
                raise RuntimeError('Unique test window required; no screenshot captured')
            window = matches[0]
            x, y = window['at']
            width, height = window['size']
            target = evidence / f'kitty-{scenario}.png'
            subprocess.run(['grim', '-g', f'{x},{y} {width}x{height}', str(target)], check=True)
            result.append({'scenario': scenario, 'path': target.name, 'logicalWidth': width, 'logicalHeight': height})
    finally:
        (control / 'command').write_text('stop')
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            process.terminate()
    (evidence / 'kitty-captures.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result))
