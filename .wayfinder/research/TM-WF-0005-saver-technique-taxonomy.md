# Research: Build an evidence-backed saver and steering taxonomy

Ticket: [Build an evidence-backed saver and steering taxonomy](../issues/TM-WF-0005-saver-technique-taxonomy.md)

## Taxonomy

| Candidate | Role | V1 posture |
|---|---|---|
| RTK | Compress shell output locally | Strong candidate; deterministic A/B evaluation |
| AST and Tree-sitter | Preserve code structure while reducing bodies or selecting symbols | Strong candidate; parser and exactness checks required |
| Repomix | Pack repositories with include/ignore rules and optional code compression | Strong candidate; manifest and security checks required |
| Indexing and graph analysis | Retrieve relevant symbols and relationships instead of sending whole repositories | Candidate after a freshness and recall contract |
| ML compression | Score or compress prose and other dense context | Higher-risk experiment; local model and quality gates required |
| Context pruning | Reduce history or live context | Higher-risk; protect task-critical context and measure task success |
| Routing | Select saver/provider/model path and fallback behavior | Core control-plane concern, not itself a compression method |
| Output steering | Ask for terser output or lower effort | Separate from input compression; must measure quality and safety |
| Caveman | Terse-output or proxy compression behavior from the cited project | Later experiment until workload evidence exists |
| Ponytail | YAGNI-first output steering | Later experiment; guard against under-implementation |
| Honey | No authoritative identity established | Unknown; do not implement or treat as a requirement |
| Graphify | No single authoritative identity established | Unknown; require an exact source |

## Measurement requirements

Measure input savings, output savings, cache effects, latency, transform failures, and task success separately.

Record parser/model/version, selected and omitted context, redaction status, and whether counts are exact, estimated, or counterfactual.

Keep raw prompts, code, and telemetry local by default.

## Sources

- https://github.com/rtk-ai/rtk
- https://tree-sitter.github.io/tree-sitter/
- https://repomix.com/guide/code-compress
- https://github.com/yamadashy/repomix
- https://docs.headroomlabs.ai/docs/proxy#kompress-ml-compression
- https://github.com/JuliusBrussee/caveman
- https://github.com/DietrichGebert/ponytail
- https://docs.headroomlabs.ai/docs/context-management
- https://github.com/oraios/serena
- https://github.com/decolua/9router
- https://github.com/headroomlabs-ai/headroom
