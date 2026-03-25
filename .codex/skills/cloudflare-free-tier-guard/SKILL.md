---
name: cloudflare-free-tier-guard
description: Cost-aware Cloudflare implementation and review guidance for Workers, D1, R2, KV, Pages, Wrangler, and edge-hosted APIs. Use when Codex designs, edits, reviews, debugs, or deploys Cloudflare-related code or config, especially `wrangler.toml`, Worker entrypoints, storage bindings, upload flows, caching, or infrastructure choices where staying inside free-tier or low-cost limits matters.
---

# Cloudflare Free Tier Guard

## Overview

Apply a free-tier-first design review before making Cloudflare changes.
Favor architectures that keep Workers thin, avoid unnecessary paid products, and make request, storage, and egress costs explicit.

## Quick Start

1. Read [references/free-tier-checklist.md](./references/free-tier-checklist.md).
2. If working in this repository's `server/` directory, read [references/launcherg-server-stack.md](./references/launcherg-server-stack.md).
3. Identify which Cloudflare primitives are involved: Workers, D1, R2, KV, Assets, Queues, Durable Objects, Cron, or custom domains.
4. State the expected hot path in one sentence: request source, Worker logic, storage access, response path.
5. Prefer the cheapest primitive that satisfies the requirement, then explain why.

## Workflow

### 1. Inventory the cost surface

- Count likely request paths.
- Separate metadata reads/writes from binary transfer.
- Note whether traffic is public, authenticated, bursty, or background.
- Call out which operations hit D1, R2, or Worker CPU.

### 2. Choose the lowest-cost architecture

- Prefer: static assets via Assets/Pages, metadata in D1, blobs in R2, direct upload/download where possible.
- Avoid adding Durable Objects, Queues, Cron, or extra Workers unless the requirement needs coordination, background execution, or fan-out.
- Keep auth stateless when possible. Signed cookies or signed tokens are cheaper than session tables.
- Keep the Worker off the binary data path unless authorization or transformation is required.

### 3. Apply free-tier techniques

- Deduplicate uploads before generating storage writes.
- Batch D1 writes and chunk large lookups.
- Return only required columns.
- Put TTLs on signed URLs and session cookies.
- Cache safe reads aggressively.
- Move heavy transforms to the client or build step.
- Avoid polling loops and worker-to-worker hops.

### 4. Explicitly review risk before coding

Before finalizing a design or patch, answer these questions:

- Does this increase request count on every user action?
- Does this push binary payloads through the Worker unnecessarily?
- Does this add a paid Cloudflare product without proving it is needed?
- Can the same behavior be achieved with Assets, D1, R2, cache headers, or client-side work?
- What is the likely free-tier bottleneck first: request count, CPU, D1 operations, R2 storage, or egress?

If the change is likely to exceed free-tier comfort, say so plainly and propose a cheaper alternative.

## Output Requirements

When this skill is used, include a short cost note in the answer:

- Architecture choice
- Main free-tier pressure points
- Concrete mitigations applied
- Any remaining unknowns that need measurement

## References

- [references/free-tier-checklist.md](./references/free-tier-checklist.md)
- [references/launcherg-server-stack.md](./references/launcherg-server-stack.md)
