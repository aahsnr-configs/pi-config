# Real stdio MCP server fixture. Only writes inside the supplied disposable root.
import sys,json,pathlib
root=pathlib.Path(sys.argv[1]).resolve()
for line in sys.stdin:
 r=json.loads(line);method=r.get('method');result={}
 if method=='initialize':result={'protocolVersion':r['params']['protocolVersion'],'capabilities':{'tools':{}},'serverInfo':{'name':'permission-fixture','version':'1'}}
 elif method=='tools/list':result={'tools':[{'name':'write_marker','description':'Write disposable marker','inputSchema':{'type':'object','properties':{'path':{'type':'string'},'content':{'type':'string'}},'required':['path','content']},'annotations':{'destructiveHint':True,'readOnlyHint':False}},{'name':'opaque_write','description':'Demonstrate trusted server side effect','inputSchema':{'type':'object','properties':{}},'annotations':{'destructiveHint':True,'readOnlyHint':False}}]}
 elif method=='tools/call':
  args=r['params'].get('arguments',{});opaque=r['params']['name']=='opaque_write';target=(root/'outside'/'marker.txt') if opaque else pathlib.Path(args['path']).resolve()
  assert target.is_relative_to(root),'fixture never accesses outside its disposable root'
  target.write_text('changed' if opaque else args['content'])
  with (root/'mcp-executions.jsonl').open('a') as f:f.write(json.dumps({'tool':r['params']['name']})+'\n')
  result={'content':[{'type':'text','text':'fixture marker changed'}]}
 if 'id' in r:print(json.dumps({'jsonrpc':'2.0','id':r['id'],'result':result}),flush=True)
