# Manajemen Container

Container adalah unit isolasi utama di MELISA. Setiap container adalah lingkungan Linux yang terisolasi penuh menggunakan teknologi LXC (*Linux Containers*), dengan jaringan, filesystem, dan proses yang terpisah dari host.

---

## Konsep Jaringan Container

Semua container MELISA terhubung ke bridge jaringan `lxcbr0` yang dikonfigurasi otomatis saat setup. Setiap container mendapatkan:

- **IP Private** — diberikan oleh DHCP internal (`10.0.3.x`)
- **MAC Address** — dibangkitkan secara acak untuk setiap container
- **Akses Internet** — melalui NAT routing dari host
- **DNS** — dikonfigurasi otomatis (`/etc/resolv.conf`)

Arsitektur jaringan:
```
Internet
    │
   [Host NIC]
    │
  [NAT / iptables]
    │
  [lxcbr0 bridge] ── 10.0.3.1/24
    ├── container-1 (10.0.3.10)
    ├── container-2 (10.0.3.15)
    └── container-3 (10.0.3.22)
```

---

## Mencari Distribusi

Sebelum membuat container, cari kode distribusi yang tersedia:

```
melisa --search [kata_kunci]
```

**Contoh:**
```bash
# Cari semua distro
melisa --search

# Cari berdasarkan kata kunci
melisa --search ubuntu
melisa --search alpine
melisa --search debian
```

Daftar distro diambil dari repositori LXC resmi dan di-cache selama 1 jam (`/tmp/melisa_global_distros.cache`). Jika cache masih valid, daftar disajikan dari cache untuk kecepatan.

**Format kode distro:** `distro/rilis/arsitektur`

Contoh:
```
ubuntu/jammy/amd64
ubuntu/focal/arm64
debian/bookworm/amd64
alpine/3.18/amd64
fedora/38/amd64
archlinux/current/amd64
```

---

## Membuat Container

```
melisa --create <nama> <kode_distro>
```

**Contoh:**
```bash
melisa --create webserver ubuntu/jammy/amd64
melisa --create database alpine/3.18/amd64
melisa --create myapi debian/bookworm/amd64 --audit
```

### Proses Pembuatan Container (7 Tahap)

```
[STEP 1/7] Provisioning new container 'webserver'...
  → lxc-create -n webserver -t download -- -d ubuntu -r jammy -a amd64
  [OK] Container filesystem created.

[STEP 2/7] Configuring container network...
  [OK] MAC address generated: 02:a1:4f:3b:8c:21
  [OK] Network configuration injected into container config.

[STEP 3/7] Applying DNS configuration...
  [OK] DNS resolver configured.

[STEP 4/7] Starting container...
  [OK] Container started (lxc-start).

[STEP 5/7] Waiting for network...
  [INFO] DHCP lease obtained (IP: 10.0.3.15).

[STEP 6/7] Updating system packages...
  → apt-get update && apt-get upgrade -y
  [OK] System updated.

[STEP 7/7] Recording metadata...
  [OK] Container metadata saved.

[SUCCESS] Container 'webserver' is ready!
```

### Dukungan Distribusi di Container

MELISA secara otomatis mendeteksi package manager yang sesuai untuk setiap distribusi di dalam container:

| Distribusi | Package Manager |
|------------|-----------------|
| Ubuntu, Debian, Mint, Kali | `apt` |
| Fedora, CentOS, RHEL, Rocky | `dnf` |
| Alpine | `apk` |
| Arch, Manjaro | `pacman` |
| openSUSE | `zypper` |

---

## Menjalankan dan Menghentikan Container

```bash
# Jalankan container
melisa --run <nama>

# Hentikan container
melisa --stop <nama>
```

**Contoh:**
```bash
melisa --run webserver
melisa --stop database
```

### Troubleshooting: Container Tidak Mendapat IP

Jika bridge `lxcbr0` tidak ditemukan saat menjalankan container, MELISA akan otomatis mencoba memperbaiki konfigurasi jaringan host:

```
[WARNING] Network bridge 'lxcbr0' not found. Initiating host auto-repair...
```

Jika masalah berlanjut:
```bash
sudo systemctl restart lxc-net
sudo systemctl restart networking
```

---

## Masuk ke Container

```
melisa --use <nama>
```

Perintah ini menjalankan `lxc-attach` ke container sehingga Anda mendapat shell interaktif di dalam container. Untuk kembali ke MELISA REPL, ketik `exit`.

```bash
melisa --use webserver
# Sekarang Anda berada di dalam container webserver
root@webserver:~# apt install nginx
root@webserver:~# exit
# Kembali ke MELISA REPL
```

---

## Mengirim Perintah ke Container

Tanpa masuk ke container, Anda bisa mengirim perintah langsung:

```
melisa --send <nama> <perintah>
```

**Contoh:**
```bash
# Update packages
melisa --send webserver apt update && apt upgrade -y

# Cek status service
melisa --send webserver systemctl status nginx

# Jalankan script
melisa --send webserver bash /tmp/deploy.sh
```

> ℹ️ **Catatan:** Untuk perintah `--send`, operator seperti `&&`, `||`, `;`, dan pipe `|` diizinkan karena diteruskan langsung ke LXC tanpa melalui shell parsing. Ini adalah pengecualian dari aturan Input Guard yang melarang karakter tersebut pada konteks lain.

---

## Melihat Informasi Container

### Daftar Semua Container

```bash
# Semua container
melisa --list

# Hanya container yang sedang berjalan
melisa --active
```

Output `--list`:
```
Daftar Container MELISA:
  NAME         STATUS    IP           DISTRO              CREATED
  webserver    RUNNING   10.0.3.15    ubuntu/jammy        2024-01-15 09:30:21
  database     STOPPED   -            alpine/3.18         2024-01-14 14:22:05
  myapi        RUNNING   10.0.3.22    debian/bookworm     2024-01-15 11:45:33
```

### Detail Container

```bash
melisa --info <nama>
```

Menampilkan:
- Status container (running/stopped)
- IP address
- Distro dan rilis
- Tanggal pembuatan
- Konfigurasi jaringan
- Volume yang di-mount

### Mendapatkan IP Container

```bash
melisa --ip <nama>
```

```
10.0.3.15
```

---

## Upload File ke Container

```
melisa --upload <nama> <path_di_container>
```

Perintah ini memungkinkan Anda memilih file dari filesystem lokal dan mengunggahnya ke path yang ditentukan di dalam container.

**Contoh:**
```bash
# Upload file konfigurasi
melisa --upload webserver /etc/nginx/nginx.conf

# Upload script
melisa --upload myapi /tmp/deploy.sh
```

---

## Berbagi Folder (Shared Volume)

Anda dapat me-mount folder dari host ke container:

### Tambah Shared Folder

```
melisa --share <nama> <path_host> <path_container>
```

```bash
# Mount folder proyek ke dalam container
melisa --share webserver /home/user/myproject /var/www/html

# Mount direktori data
melisa --share database /data/mysql /var/lib/mysql
```

### Hapus Shared Folder

```
melisa --reshare <nama> <path_host> <path_container>
```

```bash
melisa --reshare webserver /home/user/myproject /var/www/html
```

> ⚠️ **Catatan:** Perubahan shared folder biasanya memerlukan restart container agar berlaku.

---

## Menghapus Container

```
melisa --delete <nama>
```

MELISA akan meminta konfirmasi sebelum menghapus:

```
[WARNING] Are you sure you want to permanently delete container 'webserver'? (y/N): y
Destroying container 'webserver'... ████████████████████ 100%
[SUCCESS] Container 'webserver' has been permanently deleted.
```

> ⚠️ **Perhatian:** Penghapusan container bersifat **permanen dan tidak dapat dibatalkan**. Semua data di dalam container akan hilang kecuali disimpan di shared folder.

---

## Lingkungan Virtual (VPS/VM/OrbStack)

MELISA secara otomatis mendeteksi lingkungan virtual menggunakan `systemd-detect-virt` dan pemeriksaan OS release. Jika terdeteksi, MELISA menerapkan konfigurasi kompatibilitas khusus:

- **OrbStack** — override systemd `lxc-net.service` untuk menonaktifkan `ConditionVirtualization`
- **Docker/Container** — deteksi via `/.dockerenv`
- **OpenVZ** — deteksi via `/proc/vz`

Konfigurasi ini diterapkan otomatis — tidak ada tindakan manual yang diperlukan.