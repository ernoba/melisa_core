# Setup Awal

Halaman ini memandu Anda melalui konfigurasi MELISA setelah instalasi selesai — mulai dari setup host hingga membuat pengguna dan container pertama.

---

## Menjalankan MELISA REPL

MELISA menggunakan pendekatan *shell-as-service*: binary `melisa` didaftarkan sebagai shell login. Untuk masuk ke MELISA REPL sebagai root:

```bash
sudo melisa
```

Anda akan disambut dengan dashboard sistem seperti ini:

```
>> INITIALIZING CORE ENGINE...
0x3A7F9C12 [ OK ] Initializing core subsystem...
0x1B2E4D98 [ OK ] Verifying LXC bridge connectivity...
0x9F3C1A77 [ OK ] Loading security namespaces...

[ DONE ] KERNEL INITIALIZED: M.E.L.I.S.A // SYSTEM_STABLE_ENVIRONMENT

╔══════════════════════════════════════════════════════════╗
║  MELISA SECURE ENVIRONMENT                               ║
║  Host: myserver │ OS: Ubuntu 22.04                      ║
║  CPU: Intel Core i7  │ RAM: 4.2 GB / 16.0 GB            ║
╚══════════════════════════════════════════════════════════╝

Authenticated as root. Secure session granted.
melisa ❯
```

---

## Eskalasi Hak Akses Otomatis

MELISA **wajib berjalan sebagai root**. Jika Anda menjalankan `melisa` tanpa sudo, sistem secara otomatis melakukan re-eksekusi dengan `sudo`:

```
MELISA: Insufficient privileges detected. Elevating via sudo...
```

Variabel lingkungan yang diizinkan melewati sudo hanya:
- `TERM`, `LANG`, `LC_ALL`, `LC_MESSAGES`, `MELISA_DEBUG`

Semua variabel lingkungan lainnya tidak diwariskan ke proses root untuk alasan keamanan.

---

## Mode Audit (`--audit`)

Hampir semua perintah MELISA mendukung flag `--audit` yang menampilkan perintah sistem yang dieksekusi di balik layar:

```bash
melisa --create webserver ubuntu/jammy/amd64 --audit
```

Output mode audit:
```
[AUDIT] Running: lxc-create -n webserver -t download -- -d ubuntu -r jammy -a amd64
[AUDIT] Running: lxc-start -n webserver
...
```

Mode audit berguna untuk:
- Memahami apa yang dilakukan MELISA
- Debugging ketika ada error
- Audit keamanan dan compliance

---

## Setup Host Pertama Kali

Jalankan perintah setup untuk mengkonfigurasi server host secara otomatis:

```bash
# Di dalam MELISA REPL
melisa --setup
```

Atau langsung dari bash:
```bash
sudo melisa --setup
```

### Proses Setup Otomatis

```
════ MELISA HOST SETUP ════
[INFO] Detected host distribution: Debian / Ubuntu

[1/8] Installing LXC packages...
  → apt-get install -y lxc lxc-templates uidmap bridge-utils dnsmasq
  [OK] LXC packages installed.

[2/8] Installing SSH server...
  [OK] OpenSSH server already installed and running.

[3/8] Copying binary to system path...
  [OK] Binary installed to /usr/local/bin/melisa.

[4/8] Configuring SSH firewall rules...
  [OK] UFW rule added for SSH (port 22).

[5/8] Setting up LXC network...
  [OK] lxcbr0 bridge configured.
  [OK] NAT routing enabled.

[6/8] Registering MELISA as valid shell...
  [OK] /usr/local/bin/melisa added to /etc/shells.

[7/8] Configuring system sudoers...
  [OK] Sudoers configured for LXC operations.

[8/8] Verifying installation...
  [OK] Setup complete!
```

---

## Membuat Pengguna MELISA Pertama

Setelah setup host selesai, buat pengguna administrator pertama:

```
melisa ❯ melisa --add adminpertama
```

```
--- Provisioning New MELISA User: adminpertama ---
Select Access Level for adminpertama:
  1) Administrator (Full Management: Users, Projects & LXC)
  2) Standard User (Project & LXC Management Only)
Enter choice (1/2): 1

[SUCCESS] User account 'adminpertama' successfully created.
[INFO] Please set a password for 'adminpertama':
New password: ●●●●●●●●
Retype new password: ●●●●●●●●
[SUCCESS] Password configured successfully.
[INFO] Administrator privileges granted.
```

### Perbedaan Peran Pengguna

| Fitur | Standard User | Administrator |
|-------|:---:|:---:|
| Buat/hapus container | ✅ | ✅ |
| Jalankan/hentikan container | ✅ | ✅ |
| Masuk ke container (`--use`) | ✅ | ✅ |
| Kirim perintah ke container | ✅ | ✅ |
| Kelola proyek (personal) | ✅ | ✅ |
| Buat pengguna baru | ❌ | ✅ |
| Hapus pengguna | ❌ | ✅ |
| Upgrade peran pengguna | ❌ | ✅ |
| Buat/hapus proyek master | ❌ | ✅ |
| Undang/keluarkan pengguna dari proyek | ❌ | ✅ |
| Distribusi pembaruan proyek | ❌ | ✅ |
| Hapus riwayat REPL (`--clear`) | ❌ | ✅ |

---

## Mencari dan Membuat Container Pertama

### Cari Distribusi yang Tersedia

```
melisa ❯ melisa --search ubuntu
```

```
[INFO] Fetching available LXC distributions...
[INFO] Serving distro list from cache (TTL: 3600s)

Distribusi yang cocok dengan 'ubuntu':
  ubuntu/focal/amd64
  ubuntu/focal/arm64
  ubuntu/jammy/amd64
  ubuntu/jammy/arm64
  ubuntu/noble/amd64
```

### Buat Container

```
melisa ❯ melisa --create webserver ubuntu/jammy/amd64
```

```
[STEP 1/5] Creating container 'webserver'...
[OK] Container created.

[STEP 2/5] Configuring network...
[OK] MAC address assigned: 02:a1:b2:c3:d4:e5
[OK] Network config injected.

[STEP 3/5] Starting container...
[OK] Container started.

[STEP 4/5] Waiting for DHCP...
[INFO] Network connection established (IP: 10.0.3.15). Allowing DNS resolver to settle...
[OK] DNS configured.

[STEP 5/5] Updating system...
[OK] System updated.

[SUCCESS] Container 'webserver' is ready!
```

### Masuk ke Container

```
melisa ❯ melisa --use webserver
```

Anda akan masuk ke shell dalam container. Untuk kembali ke MELISA REPL, ketik `exit`.

---

## Langkah Selanjutnya

Setelah setup awal selesai, eksplorasi fitur-fitur lanjutan:

- 📦 [Manajemen Container](./container.md) — Kelola siklus hidup container
- 👥 [Manajemen Pengguna](./pengguna.md) — Kelola tim dan hak akses
- 📁 [Manajemen Proyek](./proyek.md) — Kolaborasi berbasis Git
- 🚀 [Deployment dengan .mel](./deployment.md) — Deploy aplikasi secara deklaratif