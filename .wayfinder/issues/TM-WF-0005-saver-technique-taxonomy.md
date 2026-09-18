---
id: TM-WF-0005
title: Build an evidence-backed saver and steering taxonomy
kind: ticket
labels:
  - wayfinder:research
mode: AFK
status: closed
parent: TM-WF-0001
blocked_by: []
assignee: copilot
created: 2026-09-18
---

## Question

Which deterministic, AST-based, retrieval/indexing, machine-learning, routing, and output-steering techniques should Tokenmill recognize, and what do the brief's references to RTK, ML, AST, honey, caveman, ponytails, repomix, and graph analysis mean in precise, evidence-backed terms?

Create a taxonomy that distinguishes an MVP candidate from a later experiment. For every candidate, state the required inputs, expected savings or quality risk, observability needs, and whether it can be evaluated without sending source data to a hosted service. Do not treat an unverified name as a requirement until its meaning is established.

## Research asset

[Research: Build an evidence-backed saver and steering taxonomy](../research/TM-WF-0005-saver-technique-taxonomy.md)

## Resolution

The strongest local-first V1 candidates are deterministic command-output reduction, AST or Tree-sitter structure, repository packing, and later indexing with an explicit freshness and recall contract. ML compression, history pruning, and output steering are higher-risk experiments and require task-quality gates.

Routing is a control-plane concern, not a compression method. Honey and Graphify remain unresolved names and are not requirements until exact primary sources are supplied.

Evidence: [research asset](../research/TM-WF-0005-saver-technique-taxonomy.md)
