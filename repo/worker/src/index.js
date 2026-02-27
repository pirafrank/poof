export default {
    async fetch(request, env) {
      if (request.method !== 'GET' && request.method !== 'HEAD') {
        return new Response('Method Not Allowed', { status: 405 });
      }

      const url = new URL(request.url);
      // strip leading / and serve index.html for root path
      const key = url.pathname.slice(1) || 'index.html';

      // Don't pull object bodies for HEAD requests.
      // Less latency, less money cost.
      const object =
        request.method === 'HEAD'
          ? await env.BUCKET.head(key)
          : await env.BUCKET.get(key);

      if (!object) {
        return new Response('Not Found', { status: 404 });
      }

      const headers = new Headers();
      object.writeHttpMetadata(headers);
      headers.set('etag', object.httpEtag);

      // do not return the body for HEAD requests
      return new Response(request.method === 'HEAD' ? null : object.body, {
        status: 200,
        headers,
      });
    },
  };
