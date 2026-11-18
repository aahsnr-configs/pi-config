# Deterministic local provider for actual Pi tool security tests, never a model.
from http.server import HTTPServer,BaseHTTPRequestHandler
import os,json
plan=json.loads(os.environ['PI_VISUAL_TOOL_PLAN'])
class Handler(BaseHTTPRequestHandler):
 def log_message(self,*args): pass
 def do_POST(self):
  request=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
  plans=plan if isinstance(plan,list) else [plan]
  index=sum(m.get('role')=='tool' for m in request['messages'])
  has_tool=index>=len(plans)
  current=plans[min(index,len(plans)-1)]
  delta={'content':'security fixture finished'} if has_tool else {'tool_calls':[{'index':0,'id':'security-call-'+str(index),'type':'function','function':{'name':current['name'],'arguments':json.dumps(current['arguments'])}}]}
  self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
  for d,finish in [(delta,None),({},'stop' if has_tool else 'tool_calls')]:
   chunk={'id':'security-fixture','object':'chat.completion.chunk','created':1,'model':'fixture','choices':[{'index':0,'delta':d,'finish_reason':finish}]}
   self.wfile.write(('data: '+json.dumps(chunk)+'\n\n').encode());self.wfile.flush()
  self.wfile.write(b'data: [DONE]\n\n');self.wfile.flush()
server=HTTPServer(('127.0.0.1',0),Handler);print(server.server_port,flush=True);server.serve_forever()
