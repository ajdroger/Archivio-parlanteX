# Oracle Cloud Free Tier - Setup Completo

**Tempo stimato: 45 minuti**

---

## 📝 Step 1: Registrazione Account (10 min)

1. Vai su https://www.oracle.com/cloud/free/
2. Click **"Start for free"**
3. Compila il form:
   - **Email**: La tua email principale
   - **Country**: Italy
   - **Cloud Account Name**: `archivio-parlante-<tuo-nome>` (unico globalmente)

4. **IMPORTANTE - Selezione Regione**:
   - Scegli **Germany Central (Frankfurt)** 
   - Oppure **UK South (London)**
   - ⚠️ NON scegliere regioni US (compliance GDPR)

5. Verifica email e completa il profilo
6. ⚠️ **NO CARTA DI CREDITO RICHIESTA** per Free Tier
   - Se chiede carta: clicca "Skip" o "Continue with Free Tier"

---

## 🖥️ Step 2: Creazione VM ARM64 (20 min)

### Via Web Console (Più Facile)

1. Login su https://cloud.oracle.com
2. Menu ☰ → **Compute** → **Instances**
3. Click **"Create Instance"**

**Configurazione VM1 (Master Node):**

```
Name: archivio-k3s-master
Image: Ubuntu 22.04 Minimal (ARM64)
Shape: VM.Standard.A1.Flex
  - OCPUs: 2
  - Memory: 12 GB
Networking:
  - Create new VCN: archivio-vcn
  - Public IP: Assign
  - Add SSH Keys: (genera o incolla la tua chiave pubblica)
Boot Volume: 50 GB (default OK)
```

4. Click **"Create"** e attendi ~3 minuti

5. Ripeti per **VM2** (Worker Node):

```
Name: archivio-k3s-worker
Image: Ubuntu 22.04 Minimal (ARM64)
Shape: VM.Standard.A1.Flex
  - OCPUs: 2
  - Memory: 12 GB
Networking:
  - Existing VCN: archivio-vcn
  - Public IP: Assign (temporaneo, useremo Cloudflare)
  - SSH Keys: (stessa di VM1)
Boot Volume: 50 GB
```

### Via Script Automatico (Avanzato)

```bash
# Richiede Oracle CLI installato
./setup-vms.sh
```

---

## 🔐 Step 3: Configurazione Security Rules (5 min)

Le VM Oracle hanno firewall di default MOLTO restrittivo. Dobbiamo aprire le porte necessarie.

1. Menu ☰ → **Networking** → **Virtual Cloud Networks**
2. Click su `archivio-vcn`
3. Click su **Security Lists** → **Default Security List**
4. Click **"Add Ingress Rules"**

**Aggiungi queste regole:**

| Source CIDR | Protocol | Port Range | Description |
|-------------|----------|------------|-------------|
| 0.0.0.0/0 | TCP | 22 | SSH |
| 0.0.0.0/0 | TCP | 6443 | Kubernetes API |
| 0.0.0.0/0 | TCP | 80 | HTTP (Cloudflare) |
| 0.0.0.0/0 | TCP | 443 | HTTPS (Cloudflare) |
| 10.0.0.0/16 | All | All | Internal VCN traffic |

⚠️ **Nota**: Cloudflare Tunnel rende sicuro anche con porte aperte (traffico passa solo tramite tunnel criptato)

---

## 🔑 Step 4: Accesso SSH alle VM (5 min)

### Da Windows (PowerShell)

```powershell
# Salva la chiave privata in ~/.ssh/oracle_key
# Assicurati che abbia permessi corretti
icacls $env:USERPROFILE\.ssh\oracle_key /inheritance:r
icacls $env:USERPROFILE\.ssh\oracle_key /grant:r "$($env:USERNAME):(R)"

# Connetti a VM1
ssh -i ~/.ssh/oracle_key ubuntu@<VM1_PUBLIC_IP>
```

### Da Linux/Mac

```bash
chmod 600 ~/.ssh/oracle_key
ssh -i ~/.ssh/oracle_key ubuntu@<VM1_PUBLIC_IP>
```

### Test Connessione

```bash
# Su VM1
ubuntu@archivio-k3s-master:~$ sudo apt update
ubuntu@archivio-k3s-master:~$ free -h
              total        used        free      shared  buff/cache   available
Mem:           11Gi       180Mi        11Gi       0.0Ki       140Mi        11Gi
Swap:            0B          0B          0B
```

✅ Se vedi ~11GB disponibili, sei pronto!

---

## 💾 Step 5: Creazione Block Storage (5 min)

Per dati persistenti (Qdrant, MySQL, Ollama models):

1. Menu ☰ → **Storage** → **Block Volumes**
2. Click **"Create Block Volume"**

**Configurazione:**

```
Name: archivio-data
Size: 100 GB (dentro i 200GB free tier)
Availability Domain: (stessa delle VM)
Performance: Balanced (default OK)
```

3. Una volta creato, **Attach** alla VM2:
   - Click sul volume → **Attached Instances**
   - Select `archivio-k3s-worker`
   - Attachment Type: Paravirtualized
   - Access: Read/Write

4. Su VM2, monta il volume:

```bash
# SSH su VM2
ssh -i ~/.ssh/oracle_key ubuntu@<VM2_PUBLIC_IP>

# Identifica il device
sudo lsblk
# Output: vedrai /dev/sdb (100GB)

# Formatta e monta
sudo mkfs.ext4 /dev/sdb
sudo mkdir -p /mnt/data
sudo mount /dev/sdb /mnt/data
sudo chown ubuntu:ubuntu /mnt/data

# Auto-mount al boot
echo '/dev/sdb /mnt/data ext4 defaults 0 0' | sudo tee -a /etc/fstab
```

---

## ✅ Verifica Setup

Checklist finale prima di procedere con k3s:

- [ ] VM1 (master) raggiungibile via SSH
- [ ] VM2 (worker) raggiungibile via SSH
- [ ] Entrambe le VM hanno ~12GB RAM disponibili
- [ ] VM2 ha volume 100GB montato su `/mnt/data`
- [ ] Security rules configurate (porta 6443, 80, 443)
- [ ] Ping tra VM1 e VM2 funziona (`ping <IP_INTERNO_VM2>`)

**Se tutto ✅, procedi con:** `../k3s/README.md`

---

## 🔍 Troubleshooting

### "Shape VM.Standard.A1.Flex not available"

Oracle limita la disponibilità ARM64 in alcune regioni/orari.

**Soluzioni:**
1. Prova regione diversa (UK South, Netherlands, ...)
2. Riprova di notte (2-6 AM CET, meno carico)
3. Script automatico di retry:

```bash
#!/bin/bash
while true; do
  oci compute instance launch --shape VM.Standard.A1.Flex ... && break
  echo "Retry in 30s..."
  sleep 30
done
```

### "Out of capacity for this shape"

Purtroppo Oracle a volte esaurisce capacity anche nel Free Tier.

**Alternative (in ordine di preferenza):**
1. **Hetzner Cloud** - CPX11 (2 vCPU, 2GB RAM) = €4.51/mese
   - Ancora molto economico
   - Setup identico (cambia solo provider)
2. **DigitalOcean** - Basic Droplet = $6/mese ($200 credito iniziale)
3. **Linode** - Nanode 1GB = $5/mese ($100 credito)

### VM creata ma SSH non funziona

```bash
# Verifica security list
# Verifica SSH key corretta
# Prova accesso via Serial Console (Oracle Web Console)
```

---

## 📞 Prossimo Step

Una volta completato questo setup, procedi con:

**→ `../k3s/README.md`** per installare Kubernetes

---

**Tip**: Salva gli IP pubblici delle VM in un file per riferimento rapido:

```bash
echo "VM1_MASTER_IP=<ip>" > ../vm-ips.env
echo "VM2_WORKER_IP=<ip>" >> ../vm-ips.env
```
