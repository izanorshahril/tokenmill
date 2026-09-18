# Tokenmill Domain Context

## Purpose

Tokenmill is a local-first effort to specify a tool that helps developers reduce and evaluate AI request context in the GitHub Copilot ecosystem.

## Terms

- **Context package**: The prompt, conversation history, repository material, instructions, command output, and tool output assembled for one AI request.
- **Saver**: A transformation that reduces or reorganizes a context package while preserving the information needed for the task.
- **Routing**: Selecting or exposing a model/provider path and, where supported, passing a request through an observable adapter.
- **Adapter**: A boundary component that translates an external protocol or user surface to the core vocabulary and owns that surface's side effects.
- **Policy**: Explicit run choices that control saver state, routing state, integration mode, and measurement requirements.
- **Observation**: A structured record of route, saver, measurement, and outcome state that does not include raw content by default.
- **User-controlled retention**: Retention governed by an explicit local setting and user deletion/export controls, rather than an imposed automatic expiry.
- **Strict mode**: A mode that fails closed when the requested integration cannot be verified or observed.
- **Compatible mode**: A mode that uses an explicitly labelled alternative workflow when the preferred integration is unavailable; it must report what was and was not measured.
- **Evaluation**: A paired comparison of a task with and without selected savers, measuring context reduction and task success together.
- **Task success**: The agreed evidence that a developer task remains useful after transformation, rather than token reduction alone.
- **Local-first**: Raw prompts, source code, and telemetry remain on the user's machine; V1 persists only redacted observations and aggregate counters, with raw-content persistence deferred until a reviewed ML use case exists.

## Scope Language

- **V1**: The GitHub Copilot ecosystem available in the target workplace environment, with exact supported surfaces still to be decided.
- **Later integration**: Direct integration with farseer, peon-py, or other agent harnesses is outside the V1 product boundary.
