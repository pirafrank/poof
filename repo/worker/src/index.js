export default {
    async fetch(request, env) {
      if (request.method !== 'GET' && request.method !== 'HEAD') {
        return new Response('Method Not Allowed', { status: 405 });
      }

      const url = new URL(request.url);
      // strip leading / and serve index.html for root path
      const key = url.pathname.slice(1) || 'index.html';

      const object = await env.BUCKET.get(key);

      if (!object) {
        return new Response('Not Found', { status: 404 });
      }

      const headers = new Headers();
      object.writeHttpMetadata(headers);
      headers.set('etag', object.httpEtag);

      return new Response(object.body, {
        status: 200,
        headers,
      });
    },
  };
