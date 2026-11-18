import { callPublicMcp } from './child-mcp.ts';

// Only the operator supplies outbound text. Evidence roles select an action by ID.
export const researchIds = Object.freeze(['keenable-search', 'keenable-fetch', 'exa-search', 'exa-fetch', 'context7-docs']);

async function fetchEvidence(connect, endpoint, tool, args, signal) {
  let result;
  try { result = await connect(endpoint, tool, { ...args }, signal); }
  catch { throw new Error('Public research unavailable'); }
  if (result.isError) {
    throw new Error('Public research unavailable');
  }
  return (result.content ?? []).filter(item => item.type === 'text' && typeof item.text === 'string')
    .map(item => item.text).join('\n').slice(0, 12000);
}

export function publicResearch(keenable, { query, url, libraryId }, connect = callPublicMcp) {
  if (keenable !== 'https://api.keenable.ai/mcp-public') {
    throw new Error('Keenable endpoint is not on the allowlist');
  }
  const exa = 'https://mcp.exa.ai/mcp?tools=web_search_exa,web_fetch_exa';
  const topics = {
    'keenable-search': query && { endpoint: keenable, tool: 'search_web_pages', args: { query, max_results: 3, snippet_max_length: 320 } },
    'keenable-fetch': url && { endpoint: keenable, tool: 'fetch_page_content', args: { url, max_chars: 8000 } },
    'exa-search': query && { endpoint: exa, tool: 'web_search_exa', args: { query, numResults: 3 } },
    'exa-fetch': url && { endpoint: exa, tool: 'web_fetch_exa', args: { urls: [url], maxCharacters: 8000 } },
    'context7-docs': libraryId && query && { endpoint: 'https://mcp.context7.com/mcp', tool: 'query-docs', args: { libraryId, query } },
  };
  const attempted = new Set();
  const collected = new Map();
  const research = async (id, signal) => {
    if (!Object.hasOwn(topics, id)) throw new Error('Public research topic is not allowed');
    const topic = topics[id];
    if (!topic) throw new Error('Public research input is not configured');
    const key = id.endsWith('-fetch') ? url : id;
    if (collected.has(key)) return collected.get(key);
    if (attempted.has(id)) throw new Error('Public research budget was spent');
    attempted.add(id);
    const evidence = await fetchEvidence(connect, topic.endpoint, topic.tool, topic.args, signal);
    const value = { source: id.endsWith('-fetch') ? url : topic.args.libraryId ?? new URL(topic.endpoint).hostname,
      server: new URL(topic.endpoint).hostname, publicUntrustedEvidence: evidence };
    collected.set(key, value);
    return value;
  };
  return Object.assign(research, { ids: Object.freeze(researchIds.filter(id => Boolean(topics[id]))) });
}
