# Inventario MCP Servers e Plugin — Archivio Parlante

**Data ricerca**: 2026-04-21  
**Scopo**: Identificare MCP servers e plugin esistenti per evitare di scrivere client HTTP custom

---

## 📊 Executive Summary

**Risultato ricerca**: Trovati MCP servers ufficiali e community per **tutti i servizi target**. Disponibili anche plugin Claude Code per linting/formattazione.

**Raccomandazione**: Adottare MCP servers ufficiali per Qdrant e MySQL, valutare MCP bridge per Ollama, usare plugin LSP per Rust/Python/PHP.

---

## 🔍 MCP Servers Ricercati

### 1. Qdrant Vector Database

**Status**: ✅ MCP server ufficiale disponibile

| Campo | Valore |
|---|---|
| **Nome** | `mcp-server-qdrant` |
| **Repo ufficiale** | [qdrant/mcp-server-qdrant](https://github.com/qdrant/mcp-server-qdrant) |
| **Tipo** | Official Qdrant MCP implementation |
| **Linguaggio** | Python |
| **Installazione** | `pip install mcp-server-qdrant` (PyPI) |
| **Funzionalità** | - `qdrant-store`: persist embeddings<br>- `qdrant-find`: semantic search<br>- Collection management<br>- Vector operations (dense + sparse) |
| **Compatibilità** | Claude vector hooks, REST API, MCP protocol |
| **Licenza** | Apache 2.0 (presumed, Qdrant standard) |
| **Community** | Multiple community forks (knowledge graph, code search) |

**Alternative valutate**:
- [delorenj/mcp-qdrant-memory](https://github.com/delorenj/mcp-qdrant-memory) — knowledge graph implementation
- [mhalder/qdrant-mcp-server](https://github.com/mhalder/qdrant-mcp-server) — semantic search + OpenAI embeddings

**Raccomandazione**: ✅ **Usare MCP ufficiale** `qdrant/mcp-server-qdrant` come base. Se serve knowledge graph legale, valutare fork `delorenj` in Fase 2.

---

### 2. Ollama (LLM locale)

**Status**: ⚠️ MCP bridge richiesto (Ollama nativo non supporta MCP)

| Campo | Valore |
|---|---|
| **Nome** | `ollama-mcp` + MCPHost bridge |
| **Repo** | [rawveg/ollama-mcp](https://github.com/rawveg/ollama-mcp) |
| **Bridge consigliato** | [MCPHost](https://github.com/mark3labs/mcphost) (Go) |
| **Tipo** | Community MCP wrapper per Ollama SDK |
| **Linguaggio** | Node.js (server), Go (bridge) |
| **Funzionalità** | - Tool calling<br>- Model switching<br>- Streaming responses<br>- Embedding generation<br>- Complete Ollama SDK via MCP |
| **Setup** | 1. `go install github.com/mark3labs/mcphost@latest`<br>2. Config JSON con Ollama URL<br>3. `mcphost -m ollama:qwen3 --config servers.json` |
| **Compatibilità modelli** | Qwen 3, Llama 3.1/3.3, Mistral, Hermes 3, Gemma 4 (14B+ raccomandato) |
| **Licenza** | MIT (community) |

**Alternative valutate**:
- [jonigl/mcp-client-for-ollama](https://github.com/jonigl/mcp-client-for-ollama) — TUI client con agent mode
- MCPHost (raccomandato, setup < 5 min)

**Raccomandazione**: 🤔 **Valutare se necessario**. Per il nostro caso, client HTTP diretto `reqwest` verso Ollama REST API (`/api/embeddings`, `/api/chat`) può essere più semplice che aggiungere un bridge MCP. **Decisione**: scrivere client Rust nativo in Fase 1.1, opzionalmente wrappare con MCP in Fase 5 se serve interoperabilità con altri tool.

---

### 3. Docker Compose

**Status**: ✅ MCP server disponibile per orchestrazione

| Campo | Valore |
|---|---|
| **Nome** | `mcp-server-docker` + `mcp-compose` |
| **Repo** | [ckreiling/mcp-server-docker](https://github.com/ckreiling/mcp-server-docker)<br>[phildougherty/mcp-compose](https://github.com/phildougherty/mcp-compose) |
| **Tipo** | Community MCP per Docker management |
| **Linguaggio** | Node.js / TypeScript |
| **Funzionalità** | - Container lifecycle (start/stop/restart)<br>- Compose orchestration<br>- Log streaming<br>- Unified API gateway<br>- Multi-server management |
| **Transport** | SSE / HTTP (Stdio limitato in prod) |
| **Prestazioni** | Docker MCP Gateway v2: p95 < 50ms, >10k req/s (benchmark 2026) |
| **Licenza** | MIT |

**Raccomandazione**: ⚠️ **Non necessario per il nostro progetto**. Docker Compose è usato solo per orchestrare i nostri 7 servizi localmente. Non abbiamo bisogno di controllare Docker via MCP. Utile se in futuro servisse gestione dinamica container da UI/agent.

---

### 4. MySQL Database

**Status**: ✅ Multipli MCP server disponibili

| Campo | Valore |
|---|---|
| **Nome** | `mysql_mcp_server` (official-like) |
| **Repo** | [designcomputer/mysql_mcp_server](https://github.com/designcomputer/mysql_mcp_server)<br>[benborla/mcp-server-mysql](https://github.com/benborla/mcp-server-mysql) (~980 stars) |
| **Tipo** | Community MCP con SSH tunnel support |
| **Linguaggio** | Node.js |
| **Funzionalità** | - Schema discovery (auto-inspect)<br>- Read-only queries (sicurezza)<br>- Granular permissions (INSERT/UPDATE/DELETE)<br>- Natural language → SQL<br>- SSH tunnel per connessioni remote |
| **Cloud** | Azure MySQL MCP (Microsoft), Aurora MySQL MCP (AWS) |
| **Licenza** | MIT |

**Raccomandazione**: 🤔 **Valutare in Fase 3 (PHP Gateway)**. Il PHP gateway già accede MySQL via PDO. MCP utile se servisse query NL→SQL da admin UI o debugging assistito da Claude. Non prioritario per MVP.

---

## 🔌 Plugin Claude Code per Linting/Formattazione

### 5. Rust — clippy + rustfmt

**Status**: ✅ Plugin LSP ufficiale disponibile

| Campo | Valore |
|---|---|
| **Nome** | Rust Analyzer LSP Plugin |
| **Repo** | [zircote/rust-lsp](https://github.com/zircote/rust-lsp) |
| **Registry** | [Claude Plugin Directory](https://claude.com/plugins/rust-analyzer-lsp) |
| **Funzionalità** | - rust-analyzer LSP integration<br>- Clippy lints in real-time<br>- rustfmt formatting on save<br>- cargo check diagnostics<br>- 16 automated hooks (check, clippy, expand, audit)<br>- Refactoring operations |
| **Hooks supportati** | `rust-check`, `rust-clippy`, `rust-expand`, `rust-audit` |
| **Compatibilità** | Claude Code native, VS Code, JetBrains |

**Raccomandazione**: ✅ **Adottare** in `.claude/settings.json` come hook pre-commit. Automatizza `cargo fmt` + `cargo clippy` senza script custom.

**Setup minimo**:
```json
{
  "hooks": {
    "before-commit": "rust-check && rust-clippy"
  }
}
```

---

### 6. Python — ruff + mypy + ty

**Status**: ✅ Plugin LSP disponibili (Pyright + ty)

| Campo | Valore |
|---|---|
| **Nome** | Pyright LSP + ty LSP (Astral) |
| **Repo** | [Claude Pyright Plugin](https://claude.com/plugins/pyright-lsp)<br>[ilepn/ty-lsp-claude-code](https://github.com/ilepn/ty-lsp-claude-code) |
| **ty** | Extremely fast type checker (Rust-based, Astral/Ruff team) |
| **Funzionalità Pyright** | - Real-time type checking<br>- Error detection<br>- Code intelligence<br>- .py + .pyi support |
| **Funzionalità ty** | - Blazing-fast type check (10-100× mypy)<br>- LSP integration<br>- Compatible con Ruff |
| **Hook tools** | `uv run ruff format`, `uv run ruff check`, `uv run mypy`, `uv run ty` |
| **Astral ecosystem** | `/astral:uv`, `/astral:ruff`, `/astral:ty` skills disponibili |

**Raccomandazione**: ✅ **Adottare Pyright LSP + hook ruff**. Per velocità massima, valutare `ty` al posto di `mypy` in Fase 2 (Rust-based, stesso team di Ruff).

**Setup minimo**:
```json
{
  "hooks": {
    "before-commit": "uv run ruff format . && uv run ruff check . && uv run mypy app --strict"
  }
}
```

---

### 7. PHP — PHPStan + php-cs-fixer

**Status**: ✅ Plugin LSP disponibile

| Campo | Valore |
|---|---|
| **Nome** | PHP LSP Plugin |
| **Repo** | [zircote/php-lsp](https://github.com/zircote/php-lsp) |
| **LSP** | Phpactor / Intelephense |
| **Funzionalità** | - Phpactor LSP integration<br>- php-cs-fixer formatting<br>- PHPStan static analysis (level 9+)<br>- Automated hooks (lint, format, test)<br>- WordPress optimization (WP PHPStan skill) |
| **Skills specializzati** | [WP PHPStan](https://mcpmarket.com/tools/skills/wp-phpstan-static-analysis)<br>[PHP Modernization](https://github.com/netresearch/php-modernization-skill) (PHP 8.x patterns) |
| **Hook tools** | `php-cs-fixer fix`, `phpstan analyse --level=8`, `pest` / `phpunit` |

**Raccomandazione**: ✅ **Adottare PHP LSP Plugin** in `.claude/settings.json`. Hook pre-commit per PHPStan L8 + php-cs-fixer.

**Setup minimo**:
```json
{
  "hooks": {
    "before-commit": "composer run phpstan && composer run cs-fix"
  }
}
```

---

## 📋 Riepilogo Adozioni Raccomandate

| Servizio/Tool | MCP/Plugin | Adottare? | Priorità | Nota |
|---|---|---|---|---|
| **Qdrant** | `mcp-server-qdrant` (official) | ✅ Sì | 🔴 Alta | Usare in Fase 1.1 per operations vettoriali |
| **Ollama** | `ollama-mcp` + MCPHost bridge | ⚠️ Opzionale | 🟡 Bassa | Client HTTP nativo più semplice, MCP se serve interop |
| **Docker Compose** | `mcp-compose` | ❌ No | 🟢 N/A | Non necessario, orchestrazione statica |
| **MySQL** | `mysql_mcp_server` | ⚠️ Opzionale | 🟡 Bassa | Utile per admin UI NL→SQL, non per MVP |
| **Rust LSP** | `rust-analyzer` + hooks | ✅ Sì | 🔴 Alta | Hook pre-commit clippy + fmt |
| **Python LSP** | Pyright + Ruff hooks | ✅ Sì | 🔴 Alta | Hook pre-commit ruff + mypy/ty |
| **PHP LSP** | Phpactor + PHPStan hooks | ✅ Sì | 🔴 Alta | Hook pre-commit phpstan L8 + cs-fixer |

---

## 🎯 Azioni Successive

### Fase 0 (Setup Docker Compose)
- [ ] Nessun MCP richiesto (orchestrazione statica)

### Fase 1.1 (Rust Engine — Qdrant Client)
- [ ] **Decidere**: MCP ufficiale `qdrant/mcp-server-qdrant` vs client `qdrant-client` crate nativo
- [ ] **Raccomandazione**: Iniziare con **crate nativo `qdrant-client`** (0 dipendenze esterne, gRPC performante). Opzionalmente wrappare con MCP in Fase 5 se serve interoperabilità.

### Fase 1.1 (Rust Engine — Ollama Client)
- [ ] Scrivere client HTTP nativo `reqwest` verso `/api/embeddings` e `/api/chat`
- [ ] Skip MCP bridge per ora (complessità non giustificata)

### Fase 3 (PHP Gateway)
- [ ] Valutare `mysql_mcp_server` se serve admin panel con query NL→SQL
- [ ] Altrimenti PDO nativo sufficiente

### Setup `.claude/settings.json` (post Fase 0)
- [ ] Configurare hook pre-commit Rust: `cargo fmt && cargo clippy -- -D warnings`
- [ ] Configurare hook pre-commit Python: `uv run ruff format . && uv run ruff check . && uv run mypy app --strict`
- [ ] Configurare hook pre-commit PHP: `composer run phpstan && composer run cs-fix`

---

## 📚 Sources

### MCP Servers
- [GitHub - qdrant/mcp-server-qdrant](https://github.com/qdrant/mcp-server-qdrant)
- [Qdrant MCP Server: Vector Database Storage & Search](https://mcpmarket.com/server/qdrant-1)
- [mcp-server-qdrant · PyPI](https://pypi.org/project/mcp-server-qdrant/0.6.0/)
- [Qdrant MCP server - Portkey Docs](https://portkey.ai/docs/integrations/mcp-servers/qdrant-mcp-server)
- [Ollama MCP: How to Connect Local LLMs to Any MCP Server (2026)](https://www.morphllm.com/ollama-mcp)
- [GitHub - rawveg/ollama-mcp](https://github.com/rawveg/ollama-mcp)
- [GitHub - jonigl/mcp-client-for-ollama](https://github.com/jonigl/mcp-client-for-ollama)
- [Deploy MCP Server with Docker: Complete Guide for 2026](https://mcpize.com/blog/deploy-mcp-docker)
- [GitHub - phildougherty/mcp-compose](https://github.com/phildougherty/mcp-compose)
- [GitHub - ckreiling/mcp-server-docker](https://github.com/ckreiling/mcp-server-docker)
- [GitHub - designcomputer/mysql_mcp_server](https://github.com/designcomputer/mysql_mcp_server)
- [GitHub - benborla/mcp-server-mysql](https://github.com/benborla/mcp-server-mysql)
- [MySQL MCP Server: Secure AI Database Access](https://mcpmarket.com/server/mysql-2)

### Claude Code Plugins
- [GitHub - zircote/rust-lsp](https://github.com/zircote/rust-lsp)
- [Rust Analyzer LSP – Claude Plugin](https://claude.com/plugins/rust-analyzer-lsp)
- [GitHub - ilepn/ty-lsp-claude-code](https://github.com/ilepn/ty-lsp-claude-code)
- [Pyright LSP – Claude Plugin](https://claude.com/plugins/pyright-lsp)
- [GitHub - astral-sh/claude-code-plugins](https://github.com/astral-sh/claude-code-plugins)
- [GitHub - zircote/php-lsp](https://github.com/zircote/php-lsp)
- [WP PHPStan: WordPress Static Analysis Claude Code Skill](https://mcpmarket.com/tools/skills/wp-phpstan-static-analysis)
- [GitHub - netresearch/php-modernization-skill](https://github.com/netresearch/php-modernization-skill)

---

**Ultimo aggiornamento**: 2026-04-21 — Fase -1, Step 3
