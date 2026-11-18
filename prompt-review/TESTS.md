# Pi prompt optimization tests

## Offline checks

From the Pi agent directory, run:

```sh
node prompt-review/guard.test.mjs
node prompt-review/installed.test.mjs
```

Expect 10 guard tests and 4 installed package tests to pass. These cover tool blocking and branch recovery, Promptsmith settings parsing, and the installed optimizer’s Accept, Modify, Cancel, and end handlers with mock UI and session state.

## Interactive smoke tests

Restart regular Pi in a disposable project. Keep the usual extensions loaded so this also checks their interaction. Use a fresh session and save a copy of each draft before running the next command. Optimization sends draft text to the selected provider, so use synthetic text.

Use this draft:

> Propose a change to `README.md` that adds a Quick Start example containing the literal command `python3 hello.py`. Leave `hello.py` unchanged. Report which files would change.

| Test | Steps | Pass condition |
| --- | --- | --- |
| Settings and commands | Run `/promptsmith status` and check slash completion for `/prompt-optimize`. | Execution-contract mode, preview on, and auto-send off. Both commands are available. |
| Preview and cancel | Enter the draft without submitting. Press **Alt+P** and cancel the preview. | The original draft remains in the editor and no coding turn starts. |
| Accept and undo | Run **Alt+P** again and accept. Copy the resulting text, then invoke `/promptsmith undo` before navigating away. | Accept inserts an execution contract without submitting it. Undo restores the original draft. Paths, the literal command, and the scope limit remain intact. |
| Modify and accept | Run `/prompt-optimize` and provide the copied contract when asked. Choose **Modify**, request a clearer output format, and then choose **Accept** on the revised proposal. | The guard shows that coding tools are blocked during review. Accept returns to the original branch and puts the approved proposal in the editor. The coding task starts only after you inspect the text and press Enter. |
| Cancel or end | In separate runs, choose **Cancel** and invoke `/prompt-optimize end`. | Neither action inserts the proposal or starts a coding turn. Pi returns to the original branch, and the saved draft can be pasted back. |
| Resume review | Start optimization, wait for an idle clarification prompt, then reload or reopen that session. Finish or cancel the review. | Restrictions are restored while the phase is active. The guard clears its status after Pi returns to the original branch and is idle. |
| Recover from a failed rewrite | Use a stub provider to return malformed output twice or exceed the configured timeout. | Promptsmith reports failure. The draft stays recoverable and no coding turn starts. |

For deterministic checks of disallowed tools, use the offline fixtures. Confirm that Bash, edits, writes, unrelated reads, delegation, MCP, codemode, and compression calls are blocked during review and that the guard releases its restriction after returning to the original branch. Existing permission rules still apply.

Record the Pi and package versions, each case’s result, and any observed error. Mark cases you skipped as **NOT RUN**.
