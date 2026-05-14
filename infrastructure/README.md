# 🎯 Archivio Parlante - Zero-Cost Kubernetes Infrastructure

**Costo totale: €0.00/mese** ✅

Questa guida ti permette di deployare l'intero stack Archivio Parlante su Kubernetes **completamente GRATIS** usando Oracle Cloud Free Tier.

---

## 📋 Prerequisiti

### Account Necessari (Tutti Gratuiti)

1. **Oracle Cloud Free Tier** (SEMPRE gratuito, no carta richiesta)
   - Registrati: https://www.oracle.com/cloud/free/
   - Verifica email e completa profilo
   - ⚠️ IMPORTANTE: Scegli regione **EU-Frankfurt** (data residency EU)

2. **Cloudflare** (Free tier)
   - Registrati: https://dash.cloudflare.com/sign-up
   - Aggiungi un dominio (anche gratuito da Freenom se necessario)

3. **GitHub Account** (già hai)
   - Per CI/CD Actions (2000 min/mese gratis)

---

## 🚀 Quick Start - Setup Automatico

### Opzione 1: Script Automatico Completo (Raccomandato)

```bash
# 1. Clona il repository
git clone <repo-url>
cd archivio-parlante/infrastructure

# 2. Esegui lo script master
chmod +x scripts/deploy-all.sh
./scripts/deploy-all.sh
```

Lo script ti guiderà attraverso:
- ✅ Setup Oracle Cloud VMs
- ✅ Installazione k3s cluster
- ✅ Deploy di tutti i servizi
- ✅ Configurazione Cloudflare Tunnel
- ✅ Setup monitoring

**Tempo totale: ~2 ore** (la maggior parte è attesa provisioning VM)

### Opzione 2: Setup Manuale Step-by-Step

Segui le guide nelle sottodirectory:
1. `oracle-cloud/README.md` - Creazione VMs
2. `k3s/README.md` - Installazione Kubernetes
3. `helm/README.md` - Deploy applicazioni
4. `cloudflare/README.md` - Setup tunnel HTTPS
5. `monitoring/README.md` - Prometheus + Grafana

---

## 💰 Risorse Oracle Cloud Free Tier

**Cosa ottieni GRATIS per sempre:**

| Risorsa | Quantità | Valore Equivalente |
|---------|----------|-------------------|
| ARM CPU (Ampere A1) | 4 cores | ~€40/mese |
| RAM | 24 GB | ~€50/mese |
| Block Storage | 200 GB | ~€20/mese |
| Outbound Transfer | 10 TB/mese | ~€900/mese |
| **TOTALE** | — | **~€1010/mese GRATIS** |

---

## 📊 Architettura Deploy

```
Internet
   ↓
Cloudflare Tunnel (HTTPS gratis)
   ↓
Oracle Cloud Free Tier
   ├── VM1 (2 vCPU, 12GB RAM) - k3s master + workers
   │   ├── PHP Gateway (512Mi RAM)
   │   ├── Rust Engine (1Gi RAM)
   │   ├── Python Worker (1Gi RAM)
   │   ├── MySQL (1Gi RAM)
   │   └── Redis (256Mi RAM)
   └── VM2 (2 vCPU, 12GB RAM) - k3s workers
       ├── Qdrant (2Gi RAM + 50GB PVC)
       ├── Ollama CPU (2Gi RAM + 30GB PVC)
       ├── Prometheus (512Mi RAM)
       └── Grafana (256Mi RAM)
```

**Totale RAM utilizzata: ~18GB / 24GB disponibili** ✅

---

## ⚡ Performance Attese

| Metrica | Valore | Note |
|---------|--------|------|
| **Query Latency** | 2-4s | CPU-only Ollama (con GPU: <1s) |
| **Concurrent Users** | 10-20 | Sufficiente per demo e beta |
| **Uptime** | 99.9% | Oracle Cloud SLA |
| **Storage** | 200GB | ~500+ documenti PDF |
| **Throughput** | 10 req/s | Limitato da CPU, scalabile |

**Per vendita/demo: perfettamente accettabile** ✅

---

## 🎯 Roadmap Post-Vendita

Una volta venduto il progetto e ottenuti i primi ricavi:

### Budget €50/mese
- Aggiungi Hetzner VPS con GPU (~€45/mese)
- Ollama su GPU → query <1s
- Mantieni Oracle per HA

### Budget €200/mese
- Upgrade a DigitalOcean Kubernetes
- Managed MySQL + Redis
- Multi-region (EU + US)

### Budget €500+/mese
- AWS EKS con auto-scaling
- RDS Multi-AZ
- CloudFront CDN
- Full production setup

---

## 📝 Checklist Pre-Deploy

- [ ] Account Oracle Cloud creato e verificato
- [ ] 2 VM ARM64 create (segui `oracle-cloud/README.md`)
- [ ] Dominio configurato su Cloudflare (anche gratuito)
- [ ] Git configurato con SSH keys
- [ ] File `.env` aggiornato con secrets

---

## 🆘 Troubleshooting

### Oracle Cloud non accetta registrazione?
- Prova con email diversa
- Usa VPN se in Italia (a volte filtrano)
- Alternative: Hetzner €4.51/mese (non gratis ma economico)

### VM ARM64 non disponibili?
- Prova regione diversa (UK South, EU Amsterdam)
- Riprova in orari notturni (meno carico)
- Oracle limita availability in alcune regioni

### k3s non parte?
- Verifica 2GB RAM liberi
- Controlla firewall: `sudo ufw allow 6443/tcp`
- Log: `sudo journalctl -u k3s`

---

## 📞 Supporto

- **Issues**: GitHub Issues di questo repo
- **Documentazione**: `docs/` folder
- **Community**: [Link Discord/Telegram quando disponibile]

---

**Creato con ❤️ per sviluppatori senza budget**
**Dalla povertà al successo, un deploy alla volta** 🚀
