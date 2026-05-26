# Full Stack Deployment Checklist

## ✅ Phase 1: Code Fixes (COMPLETED)
- [x] Fixed 16 compilation errors across 9 files
- [x] Added chrono serde support
- [x] Added utoipa ToSchema derives (9 structs)
- [x] Fixed Qdrant 1.17 API compatibility
- [x] Fixed ownership/borrowing issues
- [x] Added LlmProvider.generate() method
- [x] Reduced build optimization to avoid SIGILL

## ⏳ Phase 2: Docker Build (IN PROGRESS)
- [ ] rust-engine build completes successfully
- [ ] python-worker build completes successfully
- [ ] php-gateway build completes successfully

**Commands**:
```bash
cd C:\Users\aj_93\OneDrive\Documenti\GitHub\Archivio-parlanteX
docker compose build --no-cache rust-engine
docker compose build --no-cache python-worker
docker compose build --no-cache php-gateway
```

## ⏳ Phase 3: Service Startup
- [ ] Start all services
- [ ] Verify 7 services running (MySQL, Redis, Rust, Python, PHP, Qdrant, Ollama)

**Commands**:
```bash
docker compose up -d
docker ps  # Should show 7 containers
```

**Expected Services**:
1. archivio-mysql (MySQL 8.0)
2. archivio-redis (Redis 7)
3. archivio-rust-engine (Port 8090)
4. archivio-python-worker (Port 8091)
5. archivio-php-gateway (host port 9080)
6. archivio-qdrant (host REST 6335)
7. archivio-ollama (Port 11434)

## ⏳ Phase 4: Health Checks
- [ ] Rust engine health check
- [ ] Python worker health check
- [ ] PHP gateway health check
- [ ] Qdrant health check
- [ ] Ollama health check

**Commands**:
```bash
curl http://localhost:8090/health  # Rust
curl http://localhost:8091/health  # Python
curl http://localhost:9080/health  # PHP Gateway
curl http://localhost:6335/  # Qdrant (host REST)
curl http://localhost:11434/api/tags  # Ollama
```

**Expected Response**: All should return 200 OK

## ⏳ Phase 5: Database Setup
- [ ] Verify MySQL is accessible
- [ ] Run database migrations (auto-run on startup)
- [ ] Seed test user

**Commands**:
```bash
# Verify MySQL
docker exec -it archivio-mysql mysql -u root -e "SHOW DATABASES;"

# Seed test user
docker exec -i archivio-mysql mysql -u root < db/seeds/test-user.sql
```

**Test User Credentials**:
- Email: test@example.com
- Password: password123

## ⏳ Phase 6: E2E Tests
- [ ] Frontend dev server running
- [ ] Run Playwright E2E tests
- [ ] Verify 8/8 login tests pass

**Commands**:
```bash
cd frontend
npm run dev  # In separate terminal
npm run test:e2e
```

**Expected**: 8 passing tests (login/logout flows)

## ⏳ Phase 7: Manual Testing
- [ ] Login flow
- [ ] Document upload
- [ ] RAG query with citations
- [ ] Multi-contract comparison
- [ ] LLM model switching

**Test Scenarios**:
1. **Login**: Navigate to http://localhost:5174/login, use test@example.com / password123
2. **Upload**: Go to Documents page, upload a PDF
3. **Query**: Ask a question in Chat RAG
4. **Compare**: Select 2+ documents, run comparison
5. **Model Switch**: Change LLM provider in settings

## ⏳ Phase 8: Integration Verification
- [ ] Frontend → PHP → Rust → Qdrant flow works
- [ ] Frontend → PHP → Python worker flow works
- [ ] No console errors in browser
- [ ] No service crashes in docker logs

**Commands**:
```bash
docker compose logs -f  # Monitor all services
```

## 📊 Current Status

**Completed**: Phase 1 (Code Fixes)
**In Progress**: Phase 2 (Docker Build)
**Remaining**: Phases 3-8 (Deployment & Testing)

**Overall Progress**: ~20% complete

## 🚨 Known Issues

### Fixed ✅
1. ~~32 Rust compilation errors~~ → Fixed with 16 code changes
2. ~~SIGILL illegal instruction~~ → Fixed by reducing optimization level
3. ~~Missing ToSchema derives~~ → Added to 9 structs
4. ~~Qdrant API incompatibility~~ → Migrated to 1.17 API

### Remaining ⚠️
1. **Sparse vector config**: Temporarily removed from Qdrant collection creation (needs re-adding)
2. **Build optimization**: Using opt-level 2 instead of 3 (slightly slower but compatible)

## 📝 Notes

- **Docker Desktop**: Must be running on Windows
- **WSL2**: Required for Docker Desktop
- **RAM**: Full stack requires ~4GB RAM
- **Ports (ParlanteX host)**: 5173, **9080**, 8090, 8091, **3307**, **6380**, **6335**, 11434 free; 8080/3306/6379/6333 reserved for starter
- **Ollama Models**: Will auto-download on first use (~5GB for qwen2.5:7b)

## 🎯 Success Criteria

**Deployment is successful when**:
- ✅ All 7 services running
- ✅ All health checks return 200
- ✅ Test user can log in
- ✅ E2E tests pass 8/8
- ✅ Manual test scenarios work
- ✅ No errors in service logs

---

**Last Updated**: 2026-04-29 10:30
**Session**: Rust Backend Fixes & Deployment
**Engineer**: Claude Sonnet 4.5
