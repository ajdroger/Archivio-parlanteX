# 🚀 Guida Rapida - Setup Completo in Italiano

**Tempo totale: 2-3 ore** (la maggior parte è attesa)  
**Costo: €0.00** (sempre gratis)

---

## 🎯 I Tuoi Dati Oracle Cloud

```
Account Cloud: ajmeer03
Email: ajmeer03@gmail.com
Console: https://cloud.oracle.com
```

✅ Account già creato! Ora creiamo le macchine virtuali.

---

## 📋 Percorso Completo - 3 Fasi Principali

### FASE 1: Crea le VM su Oracle Cloud (Web Console) - 30 min

**Opzione A: Automatico (consigliato se hai Linux/Mac)**

```bash
# Se hai Linux/Mac localmente
cd infrastructure/oracle-cloud
chmod +x setup-account.sh
./setup-account.sh
```

Lo script crea automaticamente tutto. **Salti al FASE 2**.

---

**Opzione B: Manuale (se sei su Windows) - SEGUI QUESTI STEP:**

#### Step 1.1: Accedi alla Console Oracle (2 min)

1. Vai su: https://cloud.oracle.com
2. Login con:
   - Cloud Account Name: `ajmeer03`
   - Email: `ajmeer03@gmail.com`  
   - Password: `@4441Amotiamo`

3. ⚠️ **Selezione Regione IMPORTANTE**:
   - In alto a destra, clicca sul nome della regione
   - Seleziona: **Germany Central (Frankfurt)** 
   - (Compliance GDPR europea)

#### Step 1.2: Crea Prima VM - Master Node (15 min)

1. Menu ☰ (hamburger in alto a sinistra)
2. **Compute** → **Instances**
3. Click **"Create Instance"** (bottone blu)

**Configurazione VM1:**

| Campo | Valore |
|-------|--------|
| **Name** | `archivio-k3s-master` |
| **Compartment** | (root) - lascia default |
| **Placement** | Availability Domain: AD-1 (primo disponibile) |
| **Image** | Click "Change Image" → Canonical Ubuntu → **22.04** (ARM64) |
| **Shape** | Click "Change Shape" → **VM.Standard.A1.Flex** |
| ↳ OCPUs | **2** |
| ↳ Memory (GB) | **12** |
| **Primary VNIC** | |
| ↳ VCN | Click "Create new virtual cloud network" |
| ↳ VCN Name | `archivio-vcn` |
| ↳ Subnet | Create new subnet (default) |
| ↳ Public IP | ✅ Assign public IPv4 address |
| **Add SSH Keys** | |
| ↳ Generate | ✅ Generate a key pair for me |
| ↳ Click | **Save Private Key** (salva come `oracle_key.pem`) |
| ↳ Click | **Save Public Key** |
| **Boot Volume** | |
| ↳ Size | 50 GB (default OK) |

4. Click **"Create"** (bottone blu in basso)
5. ⏳ Aspetta 3-5 minuti (stato diventa "Running" - verde)

6. **IMPORTANTE - Salva l'IP**:
   - Copia **Public IP Address** (es: 140.238.x.x)
   - Salvalo in un file: `VM1_IP: 140.238.x.x`

#### Step 1.3: Configura Firewall (5 min)

Dobbiamo aprire le porte necessarie.

1. Nella pagina della VM appena creata, sezione **Primary VNIC**
2. Click sul nome del **Subnet** (link blu: subnet-XXXXXXXX)
3. Click sulla **Security List** (link blu: Default Security List for archivio-vcn)
4. Click **"Add Ingress Rules"** (bottone blu)

**Aggiungi 4 regole (una alla volta):**

**Regola 1 - SSH:**
- Source CIDR: `0.0.0.0/0`
- IP Protocol: `TCP`
- Destination Port Range: `22`
- Description: `SSH access`
- Click **Add Ingress Rules**

**Regola 2 - HTTP:**
- Source CIDR: `0.0.0.0/0`
- IP Protocol: `TCP`
- Destination Port Range: `80`
- Click **Add Ingress Rules**

**Regola 3 - HTTPS:**
- Source CIDR: `0.0.0.0/0`
- IP Protocol: `TCP`
- Destination Port Range: `443`
- Click **Add Ingress Rules**

**Regola 4 - Kubernetes API:**
- Source CIDR: `0.0.0.0/0`
- IP Protocol: `TCP`
- Destination Port Range: `6443`
- Click **Add Ingress Rules**

✅ Firewall configurato!

#### Step 1.4: Crea Seconda VM - Worker Node (10 min)

Ripeti ESATTAMENTE come Step 1.2, ma con questi cambi:

| Campo | Valore Diverso |
|-------|----------------|
| **Name** | `archivio-k3s-worker` |
| **VCN** | Seleziona **archivio-vcn** (già creata) |
| **Subnet** | Seleziona subnet esistente |
| **SSH Keys** | Carica **la stessa chiave pubblica** di VM1 |

⚠️ **IMPORTANTE**: Usa la STESSA coppia di chiavi SSH (quella salvata al Step 1.2)!

Salva anche questo IP: `VM2_IP: 140.238.y.y`

✅ **Fase 1 COMPLETATA!** Hai 2 VM ARM64 con 24GB RAM totali, GRATIS per sempre!

---

### FASE 2: Installa Kubernetes (k3s) - 30 min

#### Step 2.1: Prepara la Chiave SSH su Windows (2 min)

1. Apri PowerShell
2. Vai nella cartella dove hai salvato `oracle_key.pem`
3. Esegui:

```powershell
# Imposta permessi corretti (Windows)
icacls oracle_key.pem /inheritance:r
icacls oracle_key.pem /grant:r "$($env:USERNAME):(R)"

# Sposta in .ssh
Move-Item oracle_key.pem $env:USERPROFILE\.ssh\oracle_key.pem
```

#### Step 2.2: Connetti a VM1 (Master) (2 min)

```powershell
# Sostituisci <VM1_IP> con l'IP salvato
ssh -i $env:USERPROFILE\.ssh\oracle_key.pem ubuntu@<VM1_IP>
```

**Esempio:**
```powershell
ssh -i $env:USERPROFILE\.ssh\oracle_key.pem ubuntu@140.238.50.100
```

Alla domanda "Are you sure you want to continue?", rispondi `yes`.

✅ Sei dentro VM1!

#### Step 2.3: Clone Repository su VM1 (3 min)

```bash
# Su VM1
git clone https://github.com/<your-username>/archivio-parlante.git
cd archivio-parlante
```

#### Step 2.4: Installa k3s Master (10 min)

```bash
# Sempre su VM1
cd infrastructure/k3s
chmod +x install-k3s.sh
./install-k3s.sh
```

Lo script:
- ✅ Installa k3s
- ✅ Configura firewall
- ✅ Salva token per worker
- ⏳ Tempo: ~8 minuti

Alla fine vedrai:

```
✅ k3s Master Node Installation Complete!
================================================

📋 Next Steps:
1. Copy the join command to worker nodes:
   cat ~/k3s-join-command.txt
```

#### Step 2.5: Copia Token per Worker (2 min)

```bash
# Su VM1
cat ~/k3s-join-command.txt
```

Output sarà tipo:

```
K3S_TOKEN=K10abc123def456...
K3S_URL=https://10.0.0.10:6443
```

**Copia TUTTO questo output** (lo usi al prossimo step).

#### Step 2.6: Connetti Worker Node (10 min)

**Apri una NUOVA finestra PowerShell** (lascia la prima aperta):

```powershell
# Connetti a VM2 (worker)
ssh -i $env:USERPROFILE\.ssh\oracle_key.pem ubuntu@<VM2_IP>
```

Una volta dentro VM2:

```bash
# Su VM2
# Clone repository anche qui
git clone https://github.com/<your-username>/archivio-parlante.git
cd archivio-parlante/infrastructure/k3s

# Join al cluster
chmod +x join-worker.sh
./join-worker.sh <K3S_URL> <K3S_TOKEN>
```

**Sostituisci `<K3S_URL>` e `<K3S_TOKEN>`** con i valori copiati al Step 2.5!

**Esempio:**
```bash
./join-worker.sh https://10.0.0.10:6443 K10abc123def456...
```

⏳ Tempo: ~5 minuti

#### Step 2.7: Verifica Cluster (1 min)

**Torna alla finestra PowerShell di VM1**:

```bash
# Su VM1
kubectl get nodes
```

Output DEVE mostrare:

```
NAME                    STATUS   ROLES                  AGE
archivio-k3s-master     Ready    control-plane,master   10m
archivio-k3s-worker     Ready    <none>                 2m
```

✅ **Fase 2 COMPLETATA!** Hai un cluster Kubernetes funzionante!

---

### FASE 3: Deploy Archivio Parlante - 60 min

#### Step 3.1: Deploy Automatico (1 comando!)

**Su VM1 (master)**:

```bash
cd ~/archivio-parlante/infrastructure/scripts
chmod +x deploy-all.sh
./deploy-all.sh
```

Lo script esegue:
1. Build immagini Docker (15 min)
2. Deploy MySQL + Redis (5 min)
3. Deploy Qdrant + Ollama (5 min)
4. Download modelli Ollama (10 min) 
5. Deploy app (Rust + Python + PHP) (5 min)
6. Deploy monitoring (Prometheus + Grafana) (5 min)

⏳ **Tempo totale: ~45 minuti** (la maggior parte è download modelli)

Vedrai output tipo:

```
🚀 Starting deployment...
✅ kubectl installed
✅ Helm installed
✅ Cluster connectivity OK
📦 Creating namespaces...
...
✅ Deployment complete!
```

#### Step 3.2: Verifica Deploy (5 min)

```bash
# Su VM1
kubectl get pods -n archivio-parlante
```

Aspetta che TUTTI i pod siano `Running` (può richiedere 5-10 min):

```
NAME                              READY   STATUS    RESTARTS
php-gateway-xxxxxxxxx-xxxxx       1/1     Running   0
php-gateway-xxxxxxxxx-xxxxx       1/1     Running   0
rust-engine-xxxxxxxxx-xxxxx       1/1     Running   0
rust-engine-xxxxxxxxx-xxxxx       1/1     Running   0
python-worker-xxxxxxxxx-xxxxx     1/1     Running   0
python-worker-xxxxxxxxx-xxxxx     1/1     Running   0
mysql-0                           1/1     Running   0
redis-xxxxxxxxx-xxxxx             1/1     Running   0
qdrant-0                          1/1     Running   0
ollama-0                          1/1     Running   0
```

✅ Tutti `Running`? **DEPLOYMENT COMPLETATO!**

#### Step 3.3: Test Sistema (5 min)

```bash
# Test health check
kubectl exec -n archivio-parlante deploy/rust-engine -- curl -s http://localhost:8090/health | jq .
```

Output:

```json
{
  "status": "ok",
  "service": "rust-engine",
  "version": "0.8.0",
  "providers": ["ollama"],
  "cloud_enabled": false
}
```

✅ **Sistema FUNZIONANTE!**

---

## 🎉 COMPLIMENTI! Sistema Pronto!

Hai deployato con successo:
- ✅ 2 VM Oracle Cloud (GRATIS per sempre)
- ✅ Kubernetes cluster (k3s)
- ✅ Archivio Parlante completo (7 servizi)
- ✅ Monitoraggio (Prometheus + Grafana)
- ✅ **Costo totale: €0.00/mese**

---

## 🌐 Prossimi Step: Esposizione Pubblica

Per rendere il sistema accessibile da internet con HTTPS:

```bash
cd ~/archivio-parlante/infrastructure/cloudflare
./setup-tunnel.sh
```

(Guida separata in `cloudflare/README.md`)

---

## 🆘 Problemi Comuni

### "Shape VM.Standard.A1.Flex not available"

Oracle a volte limita ARM64. **Soluzioni:**
1. Riprova di notte (2-6 AM)
2. Cambia regione (UK South, Amsterdam)
3. Usa script automatico che fa retry

### "Permission denied (publickey)"

SSH key non corretta:
```powershell
# Verifica permessi
icacls $env:USERPROFILE\.ssh\oracle_key.pem
```

### "Pod in CrashLoopBackOff"

Uno dei servizi non parte. Vedi log:
```bash
kubectl logs -n archivio-parlante <pod-name>
```

---

## 📞 Hai Bisogno di Aiuto?

Posta lo screenshot dell'errore e ti aiuto!

---

**Creato con ❤️ per chi parte da zero budget**  
**Dalla necessità all'opportunità** 🚀
