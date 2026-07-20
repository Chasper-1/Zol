# Zol Philosophy

*Translation of the Russian original.*

## Predictability over Flexibility

Zol is designed around one principle:

> The editor should never surprise the user.

If there is one correct way to perform an action, Zol exposes only that way.

A strict structure is easier to understand than multiple equivalent solutions.

---

## One Correct Project Structure

Every project has a single canonical layout.

The editor never searches for files in alternative locations and never guesses user intent.

If something is placed incorrectly, Zol reports an error instead of silently accepting it.

---

## No Hidden Behavior

Zol never silently fixes configuration mistakes.

Every configuration file is either:
* valid;
* or rejected with a diagnostic.

There are no partially loaded modules.

---

## Diagnostics over Silent Fallback

Errors are considered part of the user interface.

Diagnostics should explain:
* what happened;
* where it happened;
* why it happened;
* how to fix it.

Whenever possible, diagnostics should provide concrete suggestions.

---

## Explicit Contracts

Every Rhai module implements a predefined contract.

Missing required fields, unknown properties, incompatible API versions, or invalid locations are treated as errors.

Modules either satisfy the contract completely or are not loaded.

---

## System Modules and User Plugins

Zol distinguishes two kinds of extensions.

System modules customize built-in functionality such as themes, templates, and note behavior.

User plugins extend the editor with new functionality.

Both use the same plugin infrastructure and API versioning.

---

## Rust Owns Correctness

The Rust codebase defines the architecture, contracts, and safety guarantees.

Rhai customizes behavior without changing the integrity of the editor.

No script can alter the editor's core invariants.

---

## Simplicity over Convenience

Convenience that introduces ambiguity is rejected.

The editor prefers one understandable solution over multiple configurable ones.

The goal is not maximum flexibility, but maximum clarity.

---

## Long-Term Maintainability

Every architectural decision should make the project easier to understand years later.

If a feature cannot be explained simply, its design should be reconsidered.
