// Explicit offline UI fixture. No tool/schema registration, policies or host actions.
export default function(pi) {
 pi.registerCommand('p03-dialogs', {description:'P03 isolated dialog fixture',handler:async(_,ctx)=>{
  ctx.ui.notify('P03 demo data: dialogs are real RPC requests','info');
  ctx.ui.setStatus('p03-fixture','Waiting for explicit user answers');
  ctx.ui.setWidget('p03-fixture',['Text-only RPC widget']);
  ctx.ui.setTitle('P03 isolated fixture');
  const confirmed=await ctx.ui.confirm('P03 demo: confirmation','Explicit y/n or Escape. No host operation follows.',{timeout:30000});
  const selected=await ctx.ui.select('P03 demo: selection',['first','second'],{timeout:30000});
  const input=await ctx.ui.input('P03 demo: input','Enter text',{timeout:30000});
  const editor=await ctx.ui.editor('P03 demo: editor','line one\nline two');
  ctx.ui.notify(JSON.stringify({confirmed,selected,input,editor}),'info');
  ctx.ui.setStatus('p03-fixture',undefined);
 }});
}
