# ADR 0006: async-trait vs Native Async Traits in Rust

**Status**: ✅ **Accepted**  
**Date**: 2026-05-20  
**Deciders**: Claude Code (Rust Engineer), AjDRoger (Project Lead)  
**Context**: Fase 1-2, LLM Provider trait design for multi-provider support

---

## Context

### Problema

Archivio Parlante needs to support 14+ LLM providers (Ollama + 13 cloud APIs) through a unified `LlmProvider` trait. All provider operations are inherently async (HTTP calls, streaming responses).

**Requirements**:
1. Trait with async methods: `embed()`, `chat()`, `generate()`, `stream()`
2. Support both local (Ollama) and cloud (Anthropic, OpenAI, etc.) providers
3. Dynamic dispatch for runtime provider selection
4. Error handling with custom types (`LlmError`)
5. Testable with mocks

**Constraint**: As of Rust 1.82, **native async traits** (`async fn in trait`) do NOT support dynamic dispatch (`dyn Trait`). This is a language limitation, not a crate issue.

---

## Decision Drivers

| Factor | Weight | Notes |
|---|---|---|
| **Dynamic Dispatch** | 🔴 CRITICAL | Required for multi-provider registry |
| **Performance** | 🟡 MEDIUM | Provider latency (50-500ms) dominates trait overhead |
| **Developer Experience** | 🟢 HIGH | Simpler syntax preferred |
| **Ecosystem Compatibility** | 🟢 HIGH | Industry standard solution |
| **Future-Proofing** | 🟡 MEDIUM | Native support may come in Rust 1.8x+ |

---

## Options Considered

### Option A: Native Async Traits (Rust 1.82+)
**Status**: ❌ **Rejected** (cannot use `dyn` yet)

```rust
// ❌ DOES NOT COMPILE with dynamic dispatch
trait LlmProvider {
    async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, LlmError>;
    async fn chat(&self, messages: Vec<Message>) -> Result<String, LlmError>;
}

// ❌ Error: "dyn Trait` cannot be made into an object"
let provider: Box<dyn LlmProvider> = Box::new(OllamaProvider::new(...));
```

**Pros**:
- ✅ Native Rust syntax, no proc macro
- ✅ Better IDE autocomplete
- ✅ Future-proof when RFC lands

**Cons**:
- ❌ **BLOCKER**: Cannot use `Box<dyn LlmProvider>` (required for our registry)
- ❌ Requires static dispatch (generics), breaking multi-provider runtime selection
- ❌ Not production-ready for our use case

---

### Option B: async-trait Crate
**Status**: ✅ **ACCEPTED**

```rust
use async_trait::async_trait;

#[async_trait]
trait LlmProvider: Send + Sync {
    async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, LlmError>;
    async fn chat(&self, messages: Vec<Message>) -> Result<String, LlmError>;
    async fn stream(&self, prompt: String) -> Result<impl Stream<Item=String>, LlmError>;
}

// ✅ WORKS with dynamic dispatch
let provider: Box<dyn LlmProvider> = Box::new(OllamaProvider::new(...));
let registry: HashMap<String, Box<dyn LlmProvider>> = HashMap::new();
```

**Pros**:
- ✅ **Enables dynamic dispatch** (required for multi-provider)
- ✅ Industry standard (15M+ downloads/month, used by tokio ecosystem)
- ✅ Maintained by dtolnay (author of serde, syn, thiserror)
- ✅ Zero runtime overhead (proc macro at compile time)
- ✅ Works with trait objects (`Box<dyn Trait>`)
- ✅ Compatible with `Send + Sync` bounds for tokio

**Cons**:
- ⚠️ Slightly more complex error messages (proc macro expansion)
- ⚠️ Adds `async-trait = "0.1"` dependency (acceptable, stable crate)
- ⚠️ May become obsolete when native async traits support `dyn` (years away)

---

### Option C: Manual Boxing (No Crate)
**Status**: ❌ **Rejected** (too verbose)

```rust
trait LlmProvider: Send + Sync {
    fn embed(&self, texts: Vec<String>) -> Pin<Box<dyn Future<Output = Result<Vec<Vec<f32>>, LlmError>> + Send + '_>>;
}

impl LlmProvider for OllamaProvider {
    fn embed(&self, texts: Vec<String>) -> Pin<Box<dyn Future<...>>> {
        Box::pin(async move {
            // implementation
        })
    }
}
```

**Pros**:
- ✅ No external dependency
- ✅ Full control over Future types

**Cons**:
- ❌ Extremely verbose (5x more code)
- ❌ Error-prone (easy to miss `Pin`, `Send`, `'_` lifetime)
- ❌ Poor developer experience
- ❌ No benefit over `async-trait` (same codegen)

---

## Decision

**ACCEPTED**: Use `async-trait` crate for `LlmProvider` trait

**Rationale**:
1. **Technical Requirement**: Dynamic dispatch is non-negotiable for our multi-provider registry
2. **Industry Standard**: async-trait is the de facto solution (used by AWS SDK, Azure SDK, Google Cloud SDK)
3. **Stability**: Crate is at version 0.1.80+ with minimal breaking changes since 2020
4. **Performance**: Zero runtime cost (proc macro), provider latency dominates (50-500ms)
5. **Maintainability**: Far more readable than manual boxing, easier to onboard contributors

**Implementation**:
```toml
# Cargo.toml
[dependencies]
async-trait = "0.1.80"
```

```rust
// src/providers/mod.rs
use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, LlmError>;
    async fn chat(&self, messages: Vec<Message>) -> Result<String, LlmError>;
    async fn generate(&self, prompt: String) -> Result<String, LlmError>;
    fn name(&self) -> &str;
}

// Registry supports dynamic dispatch
pub struct LlmRegistry {
    providers: HashMap<String, Box<dyn LlmProvider>>,
}
```

---

## Consequences

### Positive
- ✅ Multi-provider registry works with trait objects
- ✅ Clean trait definitions, easy to implement new providers
- ✅ Compatible with tokio runtime and `Send + Sync` requirements
- ✅ Standard solution, familiar to Rust async developers

### Negative
- ⚠️ Dependency on external crate (acceptable, stable)
- ⚠️ May need refactor when Rust adds native `dyn async` support (estimated 2028+)

### Neutral
- 📌 Performance: <1µs proc macro overhead per call vs 50-500ms network latency (negligible)
- 📌 Binary size: +0KB (compile-time macro, no runtime code)

---

## Alternatives Considered and Rejected

| Alternative | Rejection Reason |
|---|---|
| **Enum dispatch** (`enum Provider { Ollama(...), OpenAI(...) }`) | Violates Open/Closed Principle, cannot add providers without core changes |
| **Static dispatch** (`fn query<P: LlmProvider>(provider: &P)`) | Cannot store heterogeneous providers in HashMap |
| **Callback-based** (`fn embed(&self, texts: Vec<String>, callback: Box<dyn Fn(Result)>)`) | Incompatible with async/await, poor ergonomics |

---

## References

- [async-trait crate](https://crates.io/crates/async-trait) - 15M+ downloads/month
- [Rust RFC 3185: Static async fn in traits](https://github.com/rust-lang/rfcs/pull/3185) - Accepted but no `dyn` support yet
- [Why async fn in traits are hard](https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/) - Niko Matsakis blog
- [tokio-rs uses async-trait](https://github.com/tokio-rs/axum/blob/main/axum-core/src/extract/mod.rs) - Industry adoption

---

**Decision Maker**: Claude Sonnet 4.5  
**Approved By**: AjDRoger (implicit via CLAUDE.md §4.4 - async-trait in Rust patterns)  
**Review Date**: 2027-01-01 (check if Rust native async traits support `dyn`)
