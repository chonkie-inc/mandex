interface Env {
  DB: D1Database;
  R2: R2Bucket;
  PUBLISH_KEY: string;
}

function timingSafeEqual(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  const encoder = new TextEncoder();
  const bufA = encoder.encode(a);
  const bufB = encoder.encode(b);
  let result = 0;
  for (let i = 0; i < bufA.length; i++) {
    result |= bufA[i] ^ bufB[i];
  }
  return result === 0;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const path = url.pathname;

    // CORS headers
    const corsHeaders = {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
      "Access-Control-Allow-Headers": "Content-Type, Authorization",
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
          SELECT p.name, p.description, p.ecosystem, p.source_url, p.downloads,
                 v.version AS latest_version, v.entry_count, v.size_bytes,
                 (SELECT COUNT(*) FROM versions v3 WHERE v3.package = p.name) AS version_count
          FROM packages p
          LEFT JOIN versions v ON v.package = p.name
            AND v.rowid = (
              SELECT v2.rowid FROM versions v2 WHERE v2.package = p.name
              ORDER BY v2.sort_key DESC LIMIT 1
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
          "SELECT name, description, ecosystem, source_url, downloads FROM packages WHERE name = ?1"
        ).bind(name).first();

        if (!pkg) {
          return Response.json({ error: "Package not found" }, {
            status: 404,
            headers: corsHeaders,
          });
        }

        const versions = await env.DB.prepare(
          "SELECT version, entry_count, size_bytes, published_at FROM versions WHERE package = ?1 ORDER BY sort_key DESC"
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
          "SELECT version FROM versions WHERE package = ?1 ORDER BY sort_key DESC LIMIT 1"
        ).bind(name).first();

        if (!version) {
          return Response.json({ error: "Package not found" }, {
            status: 404,
            headers: corsHeaders,
          });
        }

        // Increment download counter
        await env.DB.prepare(
          "UPDATE packages SET downloads = downloads + 1 WHERE name = ?1"
        ).bind(name).run();

        return Response.json(version, { headers: corsHeaders });
      }

      // GET /stats — total version count across all packages
      if (path === "/stats" && request.method === "GET") {
        const result = await env.DB.prepare(
          "SELECT COUNT(*) as total_versions FROM versions"
        ).first();
        return Response.json(result, { headers: corsHeaders });
      }

      // POST /packages/:name/:version — publish a package
      const publishMatch = path.match(/^\/packages\/([^/]+)\/([^/]+)$/);
      if (publishMatch && request.method === "POST") {
        const name = publishMatch[1];
        const version = publishMatch[2];

        // Auth check
        const authHeader = request.headers.get("Authorization");
        const expected = `Bearer ${env.PUBLISH_KEY}`;
        if (!authHeader || !timingSafeEqual(authHeader, expected)) {
          return Response.json({ error: "Unauthorized" }, {
            status: 401,
            headers: corsHeaders,
          });
        }

        // Read the .mandex file from request body
        const body = await request.arrayBuffer();
        if (body.byteLength === 0) {
          return Response.json({ error: "Empty body" }, {
            status: 400,
            headers: corsHeaders,
          });
        }

        // Parse metadata from query params
        const ecosystem = url.searchParams.get("ecosystem") || "";
        const description = url.searchParams.get("description") || "";
        const sourceUrl = url.searchParams.get("source_url") || "";
        const entryCount = parseInt(url.searchParams.get("entry_count") || "0", 10);

        // Compute sort_key from semver: major * 1_000_000 + minor * 1_000 + patch
        const versionParts = version.match(/^(\d+)\.(\d+)\.(\d+)/);
        const sortKey = versionParts
          ? parseInt(versionParts[1]) * 1_000_000 + parseInt(versionParts[2]) * 1_000 + parseInt(versionParts[3])
          : 0;

        // Upload to R2
        await env.R2.put(`v1/${name}/${version}.mandex`, body);

        // Upsert package in D1
        await env.DB.prepare(
          `INSERT INTO packages (name, description, ecosystem, source_url, downloads)
           VALUES (?1, ?2, ?3, ?4, 0)
           ON CONFLICT(name) DO UPDATE SET
             description = CASE WHEN ?2 != '' THEN ?2 ELSE packages.description END,
             ecosystem = CASE WHEN ?3 != '' THEN ?3 ELSE packages.ecosystem END,
             source_url = CASE WHEN ?4 != '' THEN ?4 ELSE packages.source_url END`
        ).bind(name, description, ecosystem, sourceUrl).run();

        // Insert or replace version
        await env.DB.prepare(
          `INSERT OR REPLACE INTO versions (package, version, entry_count, size_bytes, sort_key)
           VALUES (?1, ?2, ?3, ?4, ?5)`
        ).bind(name, version, entryCount, body.byteLength, sortKey).run();

        return Response.json({
          package: name,
          version,
          entry_count: entryCount,
          size_bytes: body.byteLength,
          sort_key: sortKey,
        }, { status: 201, headers: corsHeaders });
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
