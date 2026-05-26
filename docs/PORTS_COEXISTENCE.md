# Porte e coexistence con archivio-parlante-starter

**Archivio ParlanteX** (Docker) e **archivio-parlante-starter** (PHP locale / AMPPS) possono girare insieme sulla stessa macchina. Le porte **esterne** di ParlanteX sono state scelte per non collidere con lo starter.

## Porte host (da Windows / browser / curl)

| Servizio | Porta host ParlanteX | Riservata allo starter | URL esempio |
|---|---|---|---|
| PHP Gateway | **9080** → container 80 | **8080** | `http://localhost:9080/health` |
| MySQL | **3307** → 3306 | **3306** | `mysql -h 127.0.0.1 -P 3307 -u root` |
| Redis | **6380** → 6379 | **6379** | `redis-cli -p 6380` |
| Qdrant REST | **6335** → 6333 | **6333** (se usata) | `http://localhost:6335/` |
| Qdrant gRPC | **6336** → 6334 | — | — |
| Rust Engine | 8090 | — | `http://localhost:8090/health` |
| Python Worker | 8091 | — | `http://localhost:8091/health` |
| Ollama | 11434 | 11434 (condivisa OK) | `http://localhost:11434` |
| Frontend Vite (dev) | 5173 | — | `http://localhost:5173` |

**Mai** mappare ParlanteX su 8080, 3306, 6379 o 6333 **sul host**.

## URL interni (`.env` / `docker-compose` su `archivio_net`)

Tra container usare sempre hostname del servizio e porta **interna**:

| Variabile | Valore corretto |
|---|---|
| `MYSQL_HOST` | `mysql` |
| `MYSQL_PORT` | `3306` (non 3307) |
| `REDIS_URL` | `redis://redis:6379` (non 6380) |
| `QDRANT_URL` | `http://qdrant:6333` (non 6335) |
| `OLLAMA_URL` | `http://ollama:11434` |
| `RUST_ENGINE_URL` | `http://rust-engine:8090` |
| `PYTHON_WORKER_URL` | `http://python-worker:8091` (o `host.docker.internal:8091` se worker nativo) |

## Frontend in sviluppo

| Modalità | API verso PHP |
|---|---|
| Vite dev (`npm run dev`) | `VITE_API_BASE_URL=/api` + proxy in `frontend/vite.config.ts` → `localhost:9080` |
| Chiamata diretta | `VITE_API_BASE_URL=http://localhost:9080/api` |
| UI servita dal container PHP | Aprire `http://localhost:9080` (build statica in gateway, se configurata) |

## Verifica rapida

```bash
make health
curl -s http://localhost:9080/health
curl -s http://localhost:6335/
```

Riferimento operativo: [RUNBOOK.md](./RUNBOOK.md).
