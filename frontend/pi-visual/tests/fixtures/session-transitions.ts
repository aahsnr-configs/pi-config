// Reviewed test-only extension. No tools/providers or host effects.
export default function(pi) {
 pi.on('session_before_switch', async event => {
  if (event.targetSessionFile?.endsWith('veto.jsonl')) return {cancel:true};
 });
}
