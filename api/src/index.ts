interface Env {
  DB: D1Database;
  R2: R2Bucket;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const path = url.pathname;

    // CORS headers
    const corsHeaders = {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET, OPTIONS",
      "Access-Control-Allow-Headers": "Content-Type",
    };

    if (request.method === "OPTIONS") {
      return new Response(null, { headers: corsHeaders });
    }

    try {
      // GET /packages — list all packages, optional search
      if (path === "/packages" && request.method === "GET") {
        const q = url.searchParams.get("q");
        const ecosystem = url.searchParams.get("ecosystem");

        let query = `
          SELECT p.name, p.description, p.ecosystem,
                 v.version AS latest_version, v.entry_count, v.size_bytes
          FROM packages p
          LEFT JOIN versions v ON v.package = p.name
            AND v.rowid = (
              SELECT v2.rowid FROM versions v2 WHERE v2.package = p.name
              ORDER BY v2.published_at DESC LIMIT 1
            )
          WHERE 1=1
        `;
        const params: string[] = [];

        if (q) {
          query += ` AND (p.name LIKE ?1 OR p.description LIKE ?1)`;
          params.push(`%${q}%`);
        }

        if (ecosystem) {
          const idx = params.length + 1;
          query += ` AND p.ecosystem = ?${idx}`;
          params.push(ecosystem);
        }

        query += ` ORDER BY p.name`;

        const result = await env.DB.prepare(query).bind(...params).all();
        return Response.json(result.results, { headers: corsHeaders });
      }

      // GET /packages/:name — package details with all versions
      const packageMatch = path.match(/^\/packages\/([^/]+)$/);
      if (packageMatch && request.method === "GET") {
        const name = packageMatch[1];

        const pkg = await env.DB.prepare(
          "SELECT name, description, ecosystem FROM packages WHERE name = ?1"
        ).bind(name).first();

        if (!pkg) {
          return Response.json({ error: "Package not found" }, {
            status: 404,
            headers: corsHeaders,
          });
        }

        const versions = await env.DB.prepare(
          "SELECT version, entry_count, size_bytes, published_at FROM versions WHERE package = ?1 ORDER BY published_at DESC"
        ).bind(name).all();

        return Response.json({
          ...pkg,
          versions: versions.results,
        }, { headers: corsHeaders });
      }

      // GET /packages/:name/latest — resolve latest version
      const latestMatch = path.match(/^\/packages\/([^/]+)\/latest$/);
      if (latestMatch && request.method === "GET") {
        const name = latestMatch[1];

        const version = await env.DB.prepare(
          "SELECT version FROM versions WHERE package = ?1 ORDER BY published_at DESC LIMIT 1"
        ).bind(name).first();

        if (!version) {
          return Response.json({ error: "Package not found" }, {
            status: 404,
            headers: corsHeaders,
          });
        }

        return Response.json(version, { headers: corsHeaders });
      }

      // GET /stats — total version count across all packages
      if (path === "/stats" && request.method === "GET") {
        const result = await env.DB.prepare(
          "SELECT COUNT(*) as total_versions FROM versions"
        ).first();
        return Response.json(result, { headers: corsHeaders });
      }

      return Response.json({ error: "Not found" }, {
        status: 404,
        headers: corsHeaders,
      });
    } catch (e) {
      return Response.json({ error: "Internal server error" }, {
        status: 500,
        headers: corsHeaders,
      });
    }
  },
};
