# Free Tier Checklist

## Primary Rule

Prefer one simple Worker plus the minimum storage bindings needed for the feature.
Do not introduce a more complex Cloudflare product by default.

## Decision Heuristics

### Worker

- Keep request handlers thin and deterministic.
- Prefer validation, routing, auth, and metadata orchestration in the Worker.
- Avoid long compute, repeated crypto over large payloads, and binary proxying.

### D1

- Store small relational metadata, not blobs.
- Add indexes for repeated lookup paths.
- Batch writes when syncing multiple records.
- Avoid N+1 queries and full-table scans.
- Read only the columns needed for the response.

### R2

- Store binary objects and large payloads in R2, not D1.
- Prefer direct client upload with presigned URLs.
- Deduplicate objects with stable keys or content-derived identifiers when possible.
- Keep object keys deterministic and safe to reconstruct.
- Use short presign TTLs unless there is a concrete reason not to.

### Assets or Pages

- Serve built UI and static files directly from Cloudflare assets when possible.
- Avoid sending SPA routes through custom Worker code unless API interception is required.

### KV

- Use only for simple key-value reads where eventual consistency is acceptable.
- Do not reach for KV when D1 is already the source of truth for relational queries.

### Durable Objects

- Avoid by default.
- Use only when single-writer coordination, room state, or strict ordering is truly required.

### Queues and Cron

- Avoid background systems unless the requirement cannot be handled inline or at sync time.
- Prefer user-triggered sync, build-time generation, or lazy computation before adding recurring jobs.

## Cost-Saving Patterns

### Uploads

- Generate presigned upload URLs in the Worker, then upload directly from the client to R2.
- Keep thumbnails or derivatives small before upload.
- Skip uploads for objects already known by dedupe key.

### Reads

- Return metadata first and fetch blobs separately only when needed.
- Add cache headers for immutable or session-scoped reads where safe.
- Keep list endpoints free of heavyweight joins and large payloads.

### Auth

- Prefer signed cookies or tokens validated in the Worker.
- Avoid persistent session tables unless revocation or auditing requires them.

### Sync and Mutation

- Sync diffs, not full snapshots, when possible.
- Chunk large `IN (...)` queries and batched writes to stay predictable.
- Avoid writing unchanged rows.

## Review Questions

- Can this request path avoid Worker involvement after authorization?
- Can this payload stay out of D1?
- Can this write be batched or skipped?
- Can this feature work without polling or background jobs?
- Can this data be cached or derived lazily?

## Red Flags

- Blob data stored in D1
- Worker acting as a permanent image or file proxy without a reason
- Per-request D1 writes for read-heavy pages
- Background jobs introduced for work that could happen on sync
- Paid products added before measuring simpler options
