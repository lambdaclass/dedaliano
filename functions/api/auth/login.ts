/**
 * POST /api/auth/login
 * Logs Google sign-in events to Cloudflare KV for user tracking.
 *
 * Env bindings required:
 *   - AUTH_USERS (KV namespace) — stores user login records
 */

interface Env {
  AUTH_USERS?: KVNamespace;
}

interface LoginBody {
  email: string;
  name: string;
  sub: string;
}

const ALLOWED_ORIGINS = [
  'https://dedaliano.com',
  'https://www.dedaliano.com',
  'https://dedaliano.pages.dev',
];

function getCorsHeaders(request: Request): Record<string, string> {
  const origin = request.headers.get('Origin') || '';
  const isLocalhost = origin.startsWith('http://localhost:') || origin.startsWith('http://127.0.0.1:');
  const allowed = ALLOWED_ORIGINS.includes(origin) || isLocalhost;
  return {
    'Access-Control-Allow-Origin': allowed ? origin : ALLOWED_ORIGINS[0],
    'Access-Control-Allow-Methods': 'POST, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type',
    'Access-Control-Max-Age': '86400',
  };
}

function jsonResponse(status: number, data: unknown, request: Request): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { 'Content-Type': 'application/json', ...getCorsHeaders(request) },
  });
}

// CORS preflight
export const onRequestOptions: PagesFunction<Env> = async (context) => {
  return new Response(null, { status: 204, headers: getCorsHeaders(context.request) });
};

// POST /api/auth/login
export const onRequestPost: PagesFunction<Env> = async (context) => {
  try {
    const body = (await context.request.json()) as LoginBody;

    if (!body.email || !body.sub) {
      return jsonResponse(400, { error: 'Missing email or sub' }, context.request);
    }

    const kv = context.env.AUTH_USERS;
    if (kv) {
      const key = `user:${body.sub}`;
      const existing = await kv.get(key, 'json') as { loginCount: number; firstSeen: string } | null;

      const record = {
        email: body.email,
        name: body.name || '',
        sub: body.sub,
        firstSeen: existing?.firstSeen || new Date().toISOString(),
        lastSeen: new Date().toISOString(),
        loginCount: (existing?.loginCount || 0) + 1,
        ip: context.request.headers.get('CF-Connecting-IP') || '',
        country: context.request.headers.get('CF-IPCountry') || '',
      };

      await kv.put(key, JSON.stringify(record));

      // Also maintain a recent-logins list (last 100)
      const recentKey = 'recent-logins';
      const recentRaw = await kv.get(recentKey, 'json') as Array<{ email: string; name: string; ts: string }> | null;
      const recent = recentRaw || [];
      recent.unshift({ email: body.email, name: body.name, ts: new Date().toISOString() });
      if (recent.length > 100) recent.length = 100;
      await kv.put(recentKey, JSON.stringify(recent));
    }

    return jsonResponse(200, { ok: true }, context.request);
  } catch (err) {
    console.error('Auth login error:', err);
    return jsonResponse(500, { error: 'Internal error' }, context.request);
  }
};
