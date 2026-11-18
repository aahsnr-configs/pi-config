import { McpClient, StreamableHttpTransport } from '../install/releases/1.5.2/node_modules/@earendil-works/pi-mcp/dist/index.js';

// No Pi MCP extension is registered. The evidence broker performs one explicit
// call through Pi's installed client using only operator supplied public input.
export async function callPublicMcp(endpoint: string, name: string, args: Record<string, unknown>, signal: AbortSignal, send = fetch, context7Key?: string) {
  const deadline = AbortSignal.any([signal, AbortSignal.timeout(8000)]);
  const client = new McpClient({ name: 'pi-workflow-public-docs', version: '1', requestTimeoutMs: 8000 });
  try {
    await client.connect(new StreamableHttpTransport({ url: endpoint, fetch: async (url, options = {}) => {
      if (URL.parse(String(url))?.href !== endpoint) throw new Error('Out-of-scope research endpoint');
      const headers = new Headers(options.headers);
      for (const key of ['authorization', 'cookie', 'proxy-authorization', 'referer']) headers.delete(key);
      if (endpoint === 'https://mcp.context7.com/mcp' && context7Key) {
        headers.set('authorization', `Bearer ${context7Key}`);
      }
      const requestSignal = options.signal ? AbortSignal.any([deadline, options.signal]) : deadline;
      const response = await send(endpoint, { ...options, headers, signal: requestSignal, redirect: 'error', credentials: 'omit' });
      if (!response.body) return response;
      let bytes = 0;
      const stream = response.body.pipeThrough(new TransformStream<Uint8Array, Uint8Array>({ transform(chunk, controller) {
        bytes += chunk.byteLength;
        if (bytes > 512 * 1024) controller.error(new Error('Public research response exceeds limit'));
        else controller.enqueue(chunk);
      } }));
      return new Response(stream, { status: response.status, statusText: response.statusText, headers: response.headers });
    } }));
    return await client.callTool(name, args, { signal: deadline });
  } finally {
    await client.close();
  }
}
