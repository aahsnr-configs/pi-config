# Loopback-only deterministic OpenAI-compatible fixture. No model or external IO.
from http.server import HTTPServer, BaseHTTPRequestHandler
import json
class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args): pass
    def do_POST(self):
        n=int(self.headers['Content-Length'])
        request=json.loads(self.rfile.read(n))
        assert request['model']=='fixture'
        self.send_response(200)
        self.send_header('Content-Type','text/event-stream')
        self.end_headers()
        for delta,finish in [({'role':'assistant','content':''},None),({'content':'local '},None),({'content':'fixture'},None),({},'stop')]:
            chunk={'id':'fixture','object':'chat.completion.chunk','created':1,'model':'fixture','choices':[{'index':0,'delta':delta,'finish_reason':finish}]}
            self.wfile.write(('data: '+json.dumps(chunk)+'\n\n').encode());self.wfile.flush()
        self.wfile.write(b'data: [DONE]\n\n');self.wfile.flush()
server=HTTPServer(('127.0.0.1',0),Handler)
print(server.server_port,flush=True)
server.serve_forever()
