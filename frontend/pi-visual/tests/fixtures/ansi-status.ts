// Synthetic startup-only UI fixture. No tools, providers, network or host actions.
export default function(pi) {
 pi.on('session_start', async (_event,ctx) => {
  ctx.ui.setStatus('extmgr','\x1b[38;2;126;136;142mDEMO obsolete\x1b[39m');
  ctx.ui.setStatus('extmgr','\x1b[38;2;126;136;142mDEMO extmgr latest\x1b[39m');
  ctx.ui.setStatus('pi-lens-lsp','\x1b[1m\x1b[38;2;126;136;142mDEMO lens ready\x1b[0m');
 });
}
