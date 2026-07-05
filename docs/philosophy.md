# Flint Philosophy

## Predictability over flexibility

Flint is designed around one principle:

> The editor should never surprise the user.

If there is one correct way to perform an action, Flint exposes only that way.

A strict structure is easier to understand than multiple equivalent solutions.

---

## One correct project structure

Every project has a single canonical layout.

The editor never searches for files in alternative locations and never guesses user intent.

If something is placed incorrectly, Flint reports an error instead of silently accepting it.

---

## No hidden behavior

Flint never silently fixes configuration mistakes.

Every configuration file is either:

* valid;
* or rejected with a diagnostic.

There are no partially loaded modules.

---

## Diagnostics instead of silent fallback

Errors are considered part of the user interface.

Diagnostics should explain:

* what happened;
* where it happened;
* why it happened;
* how to fix it.

Whenever possible, diagnostics should provide concrete suggestions.

---

## Explicit contracts

Every Rhai module implements a predefined contract.

Missing required fields, unknown properties, incompatible API versions, or invalid locations are treated as errors.

Modules either satisfy the contract completely or are not loaded.

---

## System modules and user plugins

Flint distinguishes two kinds of extensions.

System modules customize built-in functionality such as themes, templates, and note behavior.

User plugins extend the editor with new functionality.

Both use the same plugin infrastructure and API versioning.

---

## Rust owns correctness

The Rust codebase defines the architecture, contracts, and safety guarantees.

Rhai customizes behavior without changing the integrity of the editor.

No script can alter the editor's core invariants.

---

## Simplicity before convenience

Convenience that introduces ambiguity is rejected.

The editor prefers one understandable solution over multiple configurable ones.

The goal is not maximum flexibility, but maximum clarity.

---

## Long-term maintainability

Every architectural decision should make the project easier to understand years later.

If a feature cannot be explained simply, its design should be reconsidered.
