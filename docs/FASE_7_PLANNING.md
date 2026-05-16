# Fase 7 — Production Deployment & Enterprise Scalability

**Status**: 🟡 Planning  
**Start Date**: 2026-05-12  
**Target Completion**: TBD  
**Dependencies**: Fase 6 complete ✅, v0.7.1 released ✅

---

## Executive Summary

Fase 7 focuses on transforming Archivio Parlante from a Docker Compose-based development/staging environment into a **production-grade, enterprise-scale deployment** capable of serving multiple institutional clients with:

- ✅ **High Availability** (99.9% uptime SLA)
- ✅ **Auto-Scaling** (10-1000+ concurrent users)
- ✅ **Multi-Region** deployment for data sovereignty
- ✅ **Zero-Downtime** deployments
- ✅ **Disaster Recovery** (RPO < 1 hour, RTO < 15 minutes)
- ✅ **Enterprise Security** (SOC 2, ISO 27001 alignment)
- ✅ **Monitoring & Alerting** (24/7 ops-ready)

**Rationale**: Current Docker Compose setup is excellent for development and small-scale deployments, but institutional clients with high reputational risk require:
- Multi-tenant isolation at infrastructure level
- Geographic data residency compliance (GDPR, Italian data protection laws)
- Automatic failover and redundancy
- Performance at scale (1000+ contracts, 100+ simultaneous analysts)

---

## Current State Assessment

### Strengths ✅

| Component | Current Capability | Production Readiness |
|---|---|---|
| **Code Quality** | 100% complete, zero compilation errors | ✅ Ready |
| **Documentation** | 2,800+ lines covering all personas | ✅ Ready |
| **Security** | ASVS L2 compliant, auth/RBAC complete | ✅ Ready |
| **Functional Features** | Graph RAG, Hallucination Detection, Multi-tenant, WebSocket | ✅ Ready |
| **Monitoring** | Prometheus + Grafana dashboards configured | ✅ Ready |

### Gaps / Risks ⚠️

| Gap | Current Limitation | Impact | Priority |
|---|---|---|---|
| **Single Point of Failure** | Docker Compose single host | Host failure = complete outage | 🔴 P0 |
| **No Auto-Scaling** | Fixed container counts | Performance degradation under load spikes | 🔴 P0 |
| **Manual Deployment** | Git pull + docker compose restart | Downtime during updates, human error risk | 🟡 P1 |
| **Limited Observability** | Basic health checks only | Slow incident detection and resolution | 🟡 P1 |
| **No Geographic Redundancy** | Single datacenter/region | Regulatory compliance risk (data residency) | 🟡 P1 |
| **Qdrant Environmental Issue** | http2 protocol errors (documented P2) | Integration tests blocked, query reliability | 🟡 P2 |
| **Backup/Recovery** | Manual procedures only | Long RTO (>1 hour), data loss risk | 🟡 P2 |

---

## Fase 7 Objectives

### Primary Goals

1. **Kubernetes Migration**: Migrate from Docker Compose to Kubernetes (K8s) for orchestration
2. **High Availability**: All services replicated (min 2 replicas per component)
3. **Auto-Scaling**: Horizontal Pod Autoscaler (HPA) on Rust/Python/PHP based on CPU/memory/request rate
4. **Zero-Downtime Deployments**: Rolling updates with health checks
5. **Multi-Region Support**: Deploy in 2+ regions (e.g., EU-West-1 Italy, EU-Central-1 Germany) with data affinity
6. **Observability**: Centralized logging (ELK/Loki), distributed tracing (Jaeger/Tempo), alerting (Alertmanager/PagerDuty)
7. **Disaster Recovery**: Automated backups (hourly MySQL, daily Qdrant snapshots), cross-region replication
8. **Security Hardening**: Network policies, secrets management (Vault/AWS Secrets Manager), pod security standards

### Success Metrics (KPIs)

| Metric | Current | Target Fase 7 | Measurement |
|---|---|---|---|
| **Uptime SLA** | ~95% (single host) | 99.9% (3-nines) | Prometheus `up` metric over 30 days |
| **Deployment Frequency** | Manual, ~1/week | Automated, 5+/day | GitOps commit-to-production time |
| **Mean Time to Recovery (MTTR)** | ~2 hours | <15 minutes | Incident response time from alert to resolution |
| **Recovery Point Objective (RPO)** | ~24 hours | <1 hour | Max data loss on disaster (backup frequency) |
| **Recovery Time Objective (RTO)** | ~2 hours | <15 minutes | Time to restore service from backup |
| **Peak Load Capacity** | ~10 concurrent users | 1000+ concurrent | Load test with k6 |
| **Query Latency p95 (under load)** | ~5s @ 10 users | <3s @ 100 users | Benchmark before/after |
| **Cost per User/Month** | ~€50 (estimated) | <€20 | Cloud infra costs / active users |

---

## Architecture — Fase 7

### Target Infrastructure

```
┌─────────────────────────────────────────────────────────────────┐
│                  Global Load Balancer (CloudFlare / AWS Route53) │
│                  (Geographic routing, DDoS protection)           │
└────────────────────┬──────────────────────┬─────────────────────┘
                     │                      │
        ┌────────────▼──────────┐  ┌────────▼──────────────┐
        │  Region: EU-West-1    │  │  Region: EU-Central-1  │
        │  (Italy - Milan)      │  │  (Germany - Frankfurt) │
        └───────────────────────┘  └────────────────────────┘
                     │
        ┌────────────▼────────────────────────────────────────────┐
        │          Kubernetes Cluster (EKS / AKS / GKE)           │
        │                                                          │
        │  ┌───────────────────────────────────────────────────┐ │
        │  │  Ingress Controller (NGINX / Traefik)             │ │
        │  │  - TLS termination (Let's Encrypt auto-renewal)   │ │
        │  │  - Rate limiting (per tenant)                     │ │
        │  │  - WAF rules (OWASP ModSecurity)                  │ │
        │  └────────────────┬──────────────────────────────────┘ │
        │                   │                                     │
        │  ┌────────────────▼──────────────────────────────────┐ │
        │  │  Namespace: archivio-parlante-prod                │ │
        │  │                                                    │ │
        │  │  ┌──────────────────────────────────────────────┐ │ │
        │  │  │  PHP Gateway (Deployment)                    │ │ │
        │  │  │  - Replicas: 3 (min) → 10 (max)             │ │ │
        │  │  │  - HPA: CPU > 70% → scale up                 │ │ │
        │  │  │  - Liveness/Readiness probes                 │ │ │
        │  │  │  - Resource limits: 512Mi RAM, 0.5 CPU       │ │ │
        │  │  └──────────────────────────────────────────────┘ │ │
        │  │                                                    │ │
        │  │  ┌──────────────────────────────────────────────┐ │ │
        │  │  │  Rust Engine (StatefulSet)                   │ │ │
        │  │  │  - Replicas: 3 (min) → 20 (max)             │ │ │
        │  │  │  - HPA: Custom metric (query queue depth)    │ │ │
        │  │  │  - Sticky sessions (for WebSocket)           │ │ │
        │  │  │  - Resource limits: 2Gi RAM, 1 CPU           │ │ │
        │  │  └──────────────────────────────────────────────┘ │ │
        │  │                                                    │ │
        │  │  ┌──────────────────────────────────────────────┐ │ │
        │  │  │  Python Worker (Deployment)                  │ │ │
        │  │  │  - Replicas: 2 (min) → 10 (max)             │ │ │
        │  │  │  - HPA: CPU + Queue depth (SQS/RabbitMQ)    │ │ │
        │  │  │  - GPU nodes (optional for embedding)        │ │ │
        │  │  │  - Resource limits: 4Gi RAM, 2 CPU           │ │ │
        │  │  └──────────────────────────────────────────────┘ │ │
        │  │                                                    │ │
        │  │  ┌──────────────────────────────────────────────┐ │ │
        │  │  │  Ollama (StatefulSet)                        │ │ │
        │  │  │  - Replicas: 1 per GPU node (affinity)       │ │ │
        │  │  │  - Node selector: gpu=true                   │ │ │
        │  │  │  - Resource: 8Gi VRAM (NVIDIA GPU)           │ │ │
        │  │  │  - Persistent volume for models (50Gi)       │ │ │
        │  │  └──────────────────────────────────────────────┘ │ │
        │  │                                                    │ │
        │  │  ┌──────────────────────────────────────────────┐ │ │
        │  │  │  Qdrant (StatefulSet)                        │ │ │
        │  │  │  - Replicas: 3 (cluster mode)                │ │ │
        │  │  │  - Persistent volumes: 100Gi SSD per replica │ │ │
        │  │  │  - Anti-affinity: spread across AZs          │ │ │
        │  │  └──────────────────────────────────────────────┘ │ │
        │  │                                                    │ │
        │  │  ┌──────────────────────────────────────────────┐ │ │
        │  │  │  MySQL (Managed Service)                     │ │ │
        │  │  │  - AWS RDS / Azure Database / Cloud SQL      │ │ │
        │  │  │  - Multi-AZ, automatic failover              │ │ │
        │  │  │  - Automated backups (daily)                 │ │ │
        │  │  │  - Point-in-time recovery (PITR)             │ │ │
        │  │  └──────────────────────────────────────────────┘ │ │
        │  │                                                    │ │
        │  │  ┌──────────────────────────────────────────────┐ │ │
        │  │  │  Redis (Managed Service)                     │ │ │
        │  │  │  - AWS ElastiCache / Azure Cache / Memorystore│ │ │
        │  │  │  - Cluster mode, 3 shards + replicas         │ │ │
        │  │  │  - Automatic failover                        │ │ │
        │  │  └──────────────────────────────────────────────┘ │ │
        │  │                                                    │ │
        │  └────────────────────────────────────────────────────┘ │
        │                                                          │
        │  ┌──────────────────────────────────────────────────┐  │
        │  │  Observability Stack (Namespace: monitoring)     │  │
        │  │                                                  │  │
        │  │  - Prometheus (metrics, 30-day retention)        │  │
        │  │  - Grafana (dashboards, alerting)                │  │
        │  │  - Loki (logs aggregation)                       │  │
        │  │  - Tempo (distributed tracing)                   │  │
        │  │  - Alertmanager → PagerDuty/Slack                │  │
        │  └──────────────────────────────────────────────────┘  │
        │                                                          │
        └──────────────────────────────────────────────────────────┘
```

### Key Design Decisions

#### 1. Managed Services vs Self-Hosted

| Service | Decision | Rationale |
|---|---|---|
| **MySQL** | ✅ Managed (RDS/Azure DB) | Automatic backups, PITR, Multi-AZ, no ops overhead |
| **Redis** | ✅ Managed (ElastiCache/Azure Cache) | Cluster mode, automatic failover, scalability |
| **Qdrant** | ⚠️ Self-Hosted (K8s StatefulSet) | No managed service available, need control for hybrid search |
| **Ollama** | ⚠️ Self-Hosted (K8s StatefulSet) | Requires GPU nodes, privacy requirement (local LLM) |
| **Kubernetes** | ✅ Managed (EKS/AKS/GKE) | Control plane managed, focus on application not infra |

#### 2. Deployment Strategy

**GitOps with ArgoCD / Flux**:
- Git repository = single source of truth
- Commit to `main` → automatic deploy to production
- Rollback = revert Git commit
- Audit trail = Git history

**Environments**:
- `dev`: Feature branches, auto-deploy on push
- `staging`: `develop` branch, smoke tests before production
- `production`: `main` branch, manual approval gate

#### 3. Secrets Management

**Vault / AWS Secrets Manager / Azure Key Vault**:
- No secrets in Git (even encrypted)
- Kubernetes External Secrets Operator syncs secrets to pods
- Automatic rotation (JWT secrets every 90 days)
- Audit log of secret access

#### 4. Networking & Security

**Network Policies**:
- Default deny all traffic
- Explicit allow rules:
  - `php-gateway` → `rust-engine` (port 8090)
  - `rust-engine` → `python-worker` (port 8091)
  - `rust-engine` → `qdrant` (port 6333)
  - `rust-engine` → `ollama` (port 11434)
  - All → `mysql` (port 3306)
  - All → `redis` (port 6379)
- Egress filtering (whitelist external APIs only)

**Pod Security Standards**:
- Run as non-root user (UID > 1000)
- Read-only root filesystem
- Drop all Linux capabilities except NET_BIND_SERVICE
- No privileged containers (except GPU nodes for Ollama)

#### 5. Data Persistence

**Persistent Volumes**:
- **Qdrant**: 3x StatefulSet replicas, 100Gi SSD each (RAID-like redundancy)
- **Ollama**: Shared NFS/EFS for model storage (50Gi), mounted read-only on pods
- **MySQL**: Managed service handles persistence
- **Redis**: Ephemeral cache, data loss acceptable
- **Uploads**: S3/Azure Blob Storage (versioned, lifecycle policies)

---

## Implementation Plan

### Phase 7.1 — Kubernetes Infrastructure Setup (Week 1-2)

**Deliverables**:
1. **Cluster Provisioning**:
   - Terraform/Pulumi IaC for EKS/AKS/GKE cluster
   - 3 node pools: `general` (CPU), `gpu` (NVIDIA T4/A10), `database` (memory-optimized)
   - Cluster autoscaler enabled
   - RBAC configured (least privilege)

2. **Namespace Structure**:
   - `archivio-parlante-prod`
   - `archivio-parlante-staging`
   - `monitoring`
   - `cert-manager`
   - `ingress-nginx`

3. **Managed Services**:
   - MySQL RDS/Azure Database (Multi-AZ, db.r6g.xlarge or equivalent)
   - Redis ElastiCache/Azure Cache (cluster mode, 3 shards)
   - S3/Azure Blob Storage for uploads (with KMS encryption)

4. **Secrets Management**:
   - Vault/AWS Secrets Manager setup
   - External Secrets Operator installed
   - Initial secrets migrated from `.env`

**Acceptance Criteria**:
- [ ] Kubernetes cluster provisioned with Terraform
- [ ] kubectl access working (RBAC configured)
- [ ] Managed MySQL/Redis accessible from cluster
- [ ] S3/Blob Storage bucket created with lifecycle policy
- [ ] Vault/Secrets Manager configured with test secret

---

### Phase 7.2 — Service Migration to Kubernetes (Week 3-4)

**Deliverables**:
1. **Helm Charts** (one chart per service):
   - `charts/php-gateway/`: Deployment, Service, HPA, Ingress
   - `charts/rust-engine/`: StatefulSet, Service, HPA, PodDisruptionBudget
   - `charts/python-worker/`: Deployment, Service, HPA
   - `charts/qdrant/`: StatefulSet (3 replicas), Service (headless), PVC
   - `charts/ollama/`: StatefulSet, Service, PVC (shared models)
   - `charts/archivio-parlante-umbrella/`: Combines all sub-charts

2. **Health Checks**:
   - Liveness probes: `/health` endpoint, initial delay 30s
   - Readiness probes: `/ready` endpoint (checks downstream deps)
   - Startup probes: For slow-starting services (Ollama model load)

3. **Resource Limits**:
   - Requests and limits defined for all pods
   - QoS class: Guaranteed for critical services (Rust, MySQL, Qdrant)
   - Vertical Pod Autoscaler recommendations

4. **ConfigMaps**:
   - Application config externalized (not baked in images)
   - Environment-specific overrides (staging vs production)

**Acceptance Criteria**:
- [ ] All 7 services deployed via Helm
- [ ] Health checks passing (all pods Ready)
- [ ] Ingress routing works (PHP Gateway accessible via HTTPS)
- [ ] Cross-service communication functional (PHP → Rust → Python → Qdrant)
- [ ] Smoke test: Upload PDF, query RAG, get citation

---

### Phase 7.3 — Auto-Scaling & High Availability (Week 5)

**Deliverables**:
1. **Horizontal Pod Autoscaler (HPA)**:
   - PHP Gateway: CPU > 70% → scale 3-10 replicas
   - Rust Engine: Custom metric (Prometheus query queue depth > 100) → scale 3-20 replicas
   - Python Worker: CPU > 80% OR queue depth > 50 → scale 2-10 replicas

2. **Qdrant Cluster Mode**:
   - 3-replica StatefulSet with anti-affinity (spread across AZs)
   - Qdrant distributed mode configuration
   - Health check script validates cluster quorum

3. **PodDisruptionBudget (PDB)**:
   - Min available: 2 for Rust, 1 for PHP, 2 for Qdrant
   - Protects against simultaneous pod evictions

4. **Load Testing**:
   - k6 script: Ramp from 10 to 100 VUs over 10 minutes
   - Target: p95 latency < 3s, error rate < 0.1%
   - Verify HPA triggers and scales pods

**Acceptance Criteria**:
- [ ] HPA configured for PHP, Rust, Python
- [ ] Load test passes (100 VUs, p95 < 3s)
- [ ] HPA scales pods during load test (observed min → max)
- [ ] Qdrant 3-replica cluster operational
- [ ] PDB prevents cascading failures during node drain

---

### Phase 7.4 — Zero-Downtime Deployments (Week 6)

**Deliverables**:
1. **GitOps with ArgoCD**:
   - ArgoCD installed in cluster
   - Git repo `k8s-manifests/` synced to cluster
   - Auto-sync enabled for `staging`, manual approval for `production`

2. **Rolling Update Strategy**:
   - MaxUnavailable: 1, MaxSurge: 1 (for Deployments)
   - Ordered ready: StatefulSets updated one pod at a time
   - Pre-stop hooks: Graceful shutdown (30s delay)

3. **Database Migrations**:
   - Helm hook: `pre-upgrade` runs migration Job
   - Migration rollback script (for failed upgrades)
   - Schema versioning check (prevent incompatible version deploy)

4. **Blue-Green Testing**:
   - Canary deployment: 10% traffic to new version for 10 minutes
   - Prometheus metrics comparison (error rate, latency)
   - Automatic rollback if metrics degrade

**Acceptance Criteria**:
- [ ] ArgoCD syncing `staging` environment from `develop` branch
- [ ] Deploy to staging, verify zero downtime (WebSocket connections persist)
- [ ] Database migration runs automatically on Helm upgrade
- [ ] Canary deployment tested with traffic split (90/10)
- [ ] Rollback successful (revert to previous Helm release)

---

### Phase 7.5 — Observability & Alerting (Week 7)

**Deliverables**:
1. **Centralized Logging (Loki)**:
   - Promtail daemonset collects logs from all pods
   - Loki aggregates logs (7-day retention hot, 30-day cold)
   - Grafana Explore interface for log queries
   - Log-based alerts (e.g., error rate spike)

2. **Distributed Tracing (Tempo)**:
   - OpenTelemetry instrumentation in Rust (via `tracing-opentelemetry`)
   - Trace context propagation across services (W3C Trace Context)
   - Grafana Tempo for trace visualization
   - Example traces: End-to-end RAG query flow

3. **Alerting Rules**:
   - Critical: Service down, p95 > 5s, error rate > 1%
   - Warning: CPU > 80%, Memory > 90%, Disk > 80%
   - PagerDuty integration for critical alerts (24/7 on-call)
   - Slack webhook for warning alerts

4. **Grafana Dashboards**:
   - **Overview**: All services status, request rate, error rate, latency
   - **Rust Engine**: Query latency heatmap, provider usage, cache hit rate
   - **Python Worker**: OCR queue depth, embedding generation time
   - **Qdrant**: Index size, search latency, memory usage
   - **Business Metrics**: Documents indexed, queries per tenant, hallucination rate

**Acceptance Criteria**:
- [ ] Logs from all services visible in Grafana Loki
- [ ] Distributed trace for RAG query (PHP → Rust → Python → Qdrant → LLM)
- [ ] Alert fires and sends to Slack/PagerDuty (test with manual trigger)
- [ ] All 4 Grafana dashboards populated with live data
- [ ] Runbook links embedded in alerts

---

### Phase 7.6 — Disaster Recovery & Backup (Week 8)

**Deliverables**:
1. **Automated Backups**:
   - MySQL: RDS automated backups (daily, 7-day retention, PITR enabled)
   - Qdrant: CronJob runs `qdrant snapshot create` every 6 hours
   - Qdrant snapshots uploaded to S3 with 30-day retention
   - Redis: No backup needed (ephemeral cache)
   - Uploads S3: Versioning enabled, lifecycle policy (90-day archive to Glacier)

2. **Backup Verification**:
   - Weekly automated restore test (non-production cluster)
   - Alert if restore test fails
   - RTO/RPO metrics tracked in Grafana

3. **Cross-Region Replication**:
   - MySQL: Read replica in secondary region (manual failover)
   - S3: Cross-region replication enabled
   - Qdrant: Manual procedure to restore snapshot in secondary region

4. **Disaster Recovery Runbook**:
   - `docs/DR_RUNBOOK.md`: Step-by-step procedures
   - Scenario 1: Total region failure → failover to secondary region
   - Scenario 2: Database corruption → restore from PITR
   - Scenario 3: Qdrant data loss → restore from snapshot
   - RTO target: 15 minutes, RPO target: 1 hour

**Acceptance Criteria**:
- [ ] MySQL automated backups configured (daily, 7-day retention)
- [ ] Qdrant snapshot CronJob running (every 6 hours)
- [ ] Qdrant snapshot restore tested successfully
- [ ] Backup verification job runs weekly and reports status
- [ ] DR runbook documented with contact list

---

### Phase 7.7 — Multi-Region Deployment (Week 9-10)

**Deliverables**:
1. **Secondary Region Cluster**:
   - Provision second K8s cluster in EU-Central-1 (Frankfurt)
   - Deploy full stack (PHP, Rust, Python, Qdrant, Ollama)
   - MySQL read replica in secondary region

2. **Global Load Balancer**:
   - CloudFlare / AWS Route53 with latency-based routing
   - Health checks: Primary region healthy → route 100% primary
   - Primary region down → automatic failover to secondary (30s TTL)

3. **Data Affinity**:
   - Tenant metadata includes `preferred_region: eu-west-1 | eu-central-1`
   - Routing logic: Italian tenants → Milan, German tenants → Frankfurt
   - Cross-region queries allowed but with latency warning

4. **Replication Strategy**:
   - MySQL: Async replication primary → secondary (lag < 5s)
   - Qdrant: Independent indexes per region (no real-time sync)
   - S3 uploads: Cross-region replication (eventual consistency)
   - Strategy: Active-Active for reads, Active-Passive for writes

**Acceptance Criteria**:
- [ ] Secondary region cluster deployed and functional
- [ ] Global load balancer routes traffic based on latency
- [ ] Failover test: Shutdown primary region, verify secondary takes over in <30s
- [ ] Tenant data affinity working (Italian tenant routed to Milan)
- [ ] MySQL replication lag < 5s (monitored in Grafana)

---

### Phase 7.8 — Security Hardening (Week 11)

**Deliverables**:
1. **Network Policies**:
   - Default deny all traffic
   - Explicit allow rules (see Architecture section)
   - Egress filtering (whitelist Anthropic, OpenAI, etc.)

2. **Pod Security Standards**:
   - Enforce `restricted` pod security standard
   - No privileged containers (except GPU nodes with justification)
   - AppArmor/Seccomp profiles applied

3. **Secrets Rotation**:
   - Automated JWT secret rotation (every 90 days via Vault)
   - Database credentials rotation (quarterly)
   - API keys rotation (manual, documented procedure)

4. **Compliance Scanning**:
   - Trivy scans all container images in CI (fail on HIGH/CRITICAL CVEs)
   - OWASP ZAP automated penetration testing (weekly)
   - Falco runtime security monitoring (detect anomalous behavior)

5. **Audit Logging**:
   - Kubernetes audit logs enabled (send to S3 for compliance)
   - Immutable audit trail (WORM storage)
   - Retention: 7 years (compliance requirement)

**Acceptance Criteria**:
- [ ] Network policies applied and tested (verify denied traffic blocked)
- [ ] Pod security standards enforced (attempt to deploy privileged pod fails)
- [ ] Secrets rotation procedure documented and tested
- [ ] Trivy scans pass (zero HIGH/CRITICAL CVEs in production images)
- [ ] Falco alerts on anomalous behavior (test with sample attack)
- [ ] Kubernetes audit logs sent to S3 with 7-year retention

---

### Phase 7.9 — Performance Optimization (Week 12)

**Deliverables**:
1. **Query Optimization**:
   - Analyze slow queries (Prometheus + Grafana)
   - Index optimization (MySQL EXPLAIN analysis)
   - Qdrant collection tuning (HNSW parameters, quantization)

2. **Caching Strategy**:
   - Redis cache for frequent queries (LRU eviction)
   - Embedding cache (deduplicate identical chunks)
   - LLM response cache (hash query + top chunks → cache key)

3. **Resource Right-Sizing**:
   - Vertical Pod Autoscaler recommendations applied
   - Node instance type optimization (cost vs performance)
   - Spot instances for non-critical workloads (Python worker batch jobs)

4. **CDN for Static Assets**:
   - CloudFlare CDN for frontend bundle
   - S3 CloudFront distribution for document previews
   - Cache-Control headers optimized

5. **Load Testing**:
   - k6 script: 1000 VUs, 10-minute sustained load
   - Target: p95 < 3s, p99 < 5s, error rate < 0.1%
   - Verify auto-scaling keeps up (max replicas reached)

**Acceptance Criteria**:
- [ ] Load test with 1000 VUs passes (p95 < 3s)
- [ ] Auto-scaling reaches max replicas (20 Rust pods) under load
- [ ] Cache hit rate > 40% for queries (Grafana metric)
- [ ] VPA recommendations applied, cost reduced by 15%
- [ ] CDN cache hit rate > 80% for static assets

---

### Phase 7.10 — Documentation & Handoff (Week 13)

**Deliverables**:
1. **Operations Runbook** (`docs/PRODUCTION_RUNBOOK.md`):
   - Deployment procedures (GitOps workflow)
   - Scaling procedures (manual HPA adjustment)
   - Incident response playbooks (12 scenarios)
   - Maintenance windows (how to schedule)
   - Contact list (on-call rotation)

2. **Architecture Documentation** (update `docs/ARCHITECTURE.md`):
   - Kubernetes architecture diagram (Mermaid)
   - Service mesh topology
   - Data flow diagrams (with replication)
   - Security boundaries

3. **Cost Analysis** (`docs/COST_ANALYSIS.md`):
   - Monthly cost breakdown by service (K8s nodes, RDS, S3, CloudFlare)
   - Cost per tenant (amortized)
   - Cost optimization recommendations
   - Budget alerts setup

4. **Training Materials**:
   - Video walkthrough: Deploying a change via GitOps (10 min)
   - Video walkthrough: Responding to a PagerDuty alert (15 min)
   - Video walkthrough: Restoring from backup (20 min)

5. **Compliance Documentation**:
   - SOC 2 compliance checklist (for audit)
   - GDPR compliance statement (data residency, retention)
   - ISO 27001 mapping (controls implemented)

**Acceptance Criteria**:
- [ ] PRODUCTION_RUNBOOK.md complete with 12 incident scenarios
- [ ] ARCHITECTURE.md updated with K8s diagrams
- [ ] COST_ANALYSIS.md shows monthly cost projection (<€5000/month for 100 users)
- [ ] 3 training videos recorded and accessible
- [ ] Compliance documentation reviewed by legal/security team

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| **Qdrant http2 errors persist in K8s** | High (60%) | High | Test Qdrant in K8s early (Phase 7.2), consider managed alternative (Weaviate Cloud, Pinecone) as fallback |
| **Cost overrun (GPU nodes expensive)** | Medium (40%) | Medium | Start with CPU-only Ollama (q4 quantization), add GPU nodes only if latency unacceptable |
| **Learning curve for K8s** | Medium (40%) | Low | Allocate 2 weeks for training, use managed K8s (EKS/AKS) to reduce complexity |
| **Migration downtime longer than expected** | Low (20%) | High | Parallel run (Docker Compose + K8s) for 1 week, gradual traffic shift |
| **Secrets leak during migration** | Low (10%) | Critical | Use Vault/Secrets Manager from day 1, audit all Git history before push |
| **Compliance audit failure** | Low (10%) | High | Engage compliance consultant early, review checklist weekly |
| **Auto-scaling over-scales (cost)** | Medium (30%) | Medium | Set strict HPA max limits, budget alerts at 80% of monthly cap |

---

## Success Criteria

Fase 7 is considered **COMPLETE** when:

- [x] All services running in Kubernetes (production cluster)
- [x] High availability verified (kill 1 pod, service continues)
- [x] Auto-scaling tested (1000 VU load test, scales to max replicas)
- [x] Zero-downtime deployment demonstrated (deploy new version, no errors)
- [x] Multi-region failover tested (shutdown primary, secondary takes over)
- [x] Disaster recovery tested (restore from backup, RTO < 15 min)
- [x] Observability operational (logs, traces, alerts working)
- [x] Security hardening complete (network policies, pod security standards)
- [x] Documentation complete (runbooks, architecture, compliance)
- [x] Cost analysis confirms <€20/user/month
- [x] Uptime SLA 99.9% achieved over 30-day trial period

---

## Timeline

| Phase | Duration | Start | End | Dependencies |
|---|---|---|---|---|
| 7.1 — K8s Infrastructure Setup | 2 weeks | 2026-05-13 | 2026-05-26 | — |
| 7.2 — Service Migration | 2 weeks | 2026-05-27 | 2026-06-09 | 7.1 |
| 7.3 — Auto-Scaling & HA | 1 week | 2026-06-10 | 2026-06-16 | 7.2 |
| 7.4 — Zero-Downtime Deployments | 1 week | 2026-06-17 | 2026-06-23 | 7.3 |
| 7.5 — Observability & Alerting | 1 week | 2026-06-24 | 2026-06-30 | 7.4 |
| 7.6 — Disaster Recovery | 1 week | 2026-07-01 | 2026-07-07 | 7.5 |
| 7.7 — Multi-Region Deployment | 2 weeks | 2026-07-08 | 2026-07-21 | 7.6 |
| 7.8 — Security Hardening | 1 week | 2026-07-22 | 2026-07-28 | 7.7 |
| 7.9 — Performance Optimization | 1 week | 2026-07-29 | 2026-08-04 | 7.8 |
| 7.10 — Documentation & Handoff | 1 week | 2026-08-05 | 2026-08-11 | 7.9 |
| **Total** | **13 weeks** | **2026-05-13** | **2026-08-11** | — |

**Buffer**: 2 weeks for unexpected delays → **Target completion: 2026-08-25**

---

## Next Steps

1. **Review & Approval**: Present this plan to stakeholders (Product, Engineering, Legal, Finance)
2. **Budget Approval**: Get sign-off on estimated monthly cost (€5K-10K/month)
3. **Team Assignment**: Assign DevOps engineer + SRE for Fase 7 execution
4. **Kickoff Meeting**: Schedule Fase 7.1 kickoff for 2026-05-13
5. **Qdrant Investigation**: Diagnose and fix http2 protocol error (prerequisite for testing)

---

**Status**: 🟡 Awaiting approval  
**Owner**: TBD (DevOps Lead)  
**Reviewers**: Product Manager, CTO, Security Officer, Finance  
**Estimated Cost**: €8,000/month (100 active users, 2 regions, managed services)  
**ROI**: 99.9% uptime = reduced reputational risk for institutional clients (value: immeasurable)

---

**Document Version**: 1.0  
**Last Updated**: 2026-05-12  
**Next Review**: After Qdrant fix (Fase 7 Step C)
