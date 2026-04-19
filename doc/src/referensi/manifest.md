# Format File Manifest (.mel)

File `.mel` adalah file konfigurasi deployment berformat TOML (*Tom's Obvious Minimal Language*) yang mendefinisikan seluruh lingkungan deployment aplikasi Anda di MELISA.

---

## Spesifikasi Lengkap

### Section `[project]` — Wajib

Mendefinisikan metadata proyek.

```toml
[project]
name        = "nama-proyek"    # (Wajib) Nama unik proyek
version     = "1.0.0"         # (Opsional) Versi semver
description = "Deskripsi"     # (Opsional) Deskripsi singkat
author      = "Nama Author"    # (Opsional) Nama pembuat
```

| Field | Tipe | Wajib | Deskripsi |
|-------|------|:---:|-----------|
| `name` | String | ✅ | Nama unik proyek, tidak boleh kosong |
| `version` | String | ❌ | Versi aplikasi (bebas format, direkomendasikan semver) |
| `description` | String | ❌ | Deskripsi proyek |
| `author` | String | ❌ | Nama atau email pembuat |

---

### Section `[container]` — Wajib

Mendefinisikan konfigurasi container LXC yang akan dibuat.

```toml
[container]
distro     = "ubuntu/jammy/amd64"  # (Wajib) Kode distribusi LXC
name       = "nama-container"       # (Opsional) Nama container, default: nama proyek
auto_start = true                   # (Opsional) Auto start, default: true
```

| Field | Tipe | Default | Deskripsi |
|-------|------|---------|-----------|
| `distro` | String | — (wajib) | Kode distribusi LXC. Format: `distro/rilis/arsitektur`. Lihat `melisa --search` |
| `name` | String | Nama proyek (spasi diganti `-`, lowercase) | Nama container yang akan dibuat |
| `auto_start` | Boolean | `true` | Apakah container otomatis dijalankan setelah dibuat |

**Logika penamaan otomatis:**
```
project.name = "My Web App"  →  container name = "my-web-app"
project.name = "myapp"       →  container name = "myapp"
```

---

### Section `[env]` — Opsional

Mendefinisikan variabel lingkungan yang akan diset di dalam container.

```toml
[env]
DATABASE_URL = "postgres://user:pass@localhost/db"
APP_PORT     = "8080"
NODE_ENV     = "production"
```

| Field | Tipe | Deskripsi |
|-------|------|-----------|
| Kunci apa saja | String | Variabel lingkungan yang akan diinjeksikan ke container |

---

### Section `[dependencies]` — Opsional

Mendefinisikan paket yang akan diinstall di dalam container dari berbagai package manager.

```toml
[dependencies]
apt      = ["nginx", "curl", "git"]
pacman   = []
dnf      = []
zypper   = []
apk      = []
pip      = ["django", "gunicorn"]
npm      = ["pm2", "express"]
cargo    = ["mdbook"]
gem      = ["rails"]
composer = ["laravel/laravel"]
```

| Field | Package Manager | Distribusi |
|-------|-----------------|------------|
| `apt` | APT | Debian, Ubuntu, Mint |
| `pacman` | Pacman | Arch, Manjaro |
| `dnf` | DNF | Fedora, RHEL, Rocky |
| `zypper` | Zypper | openSUSE |
| `apk` | APK | Alpine |
| `pip` | pip3 | Python (semua distro) |
| `npm` | npm | Node.js (semua distro) |
| `cargo` | cargo | Rust (semua distro) |
| `gem` | gem | Ruby (semua distro) |
| `composer` | composer | PHP (semua distro) |

**Catatan:** MELISA hanya akan menginstall dependensi untuk package manager yang relevan dengan distribusi container. Dependensi untuk package manager lain diabaikan.

---

### Section `[ports]` — Opsional

Mendefinisikan port forwarding dari host ke container.

```toml
[ports]
expose = [
    "80:80",        # host_port:container_port
    "443:443",
    "8080:3000",    # port host 8080 diteruskan ke port 3000 di container
    "5432:5432",
]
```

| Field | Format | Deskripsi |
|-------|--------|-----------|
| `expose` | `"host_port:container_port"` | Daftar port yang diteruskan dari host ke container |

**Aturan validasi:**
- Setiap entry harus mengandung tepat satu karakter `:`
- Format wajib: `host_port:container_port`
- Port harus berupa angka valid (1–65535)

**Contoh yang valid:**
```toml
expose = ["80:80", "8443:443", "27017:27017"]
```

**Contoh yang tidak valid:**
```toml
expose = ["8080"]          # ❌ Tidak ada pemisah ':'
expose = ["80:8080:443"]   # ❌ Terlalu banyak ':'
```

---

### Section `[volumes]` — Opsional

Mendefinisikan folder yang di-mount dari host ke container.

```toml
[volumes]
mounts = [
    "/data/app:/app/data",
    "/var/log/myapp:/app/logs",
    "/etc/myapp/config:/app/config:ro",  # :ro untuk read-only (jika didukung)
]
```

| Field | Format | Deskripsi |
|-------|--------|-----------|
| `mounts` | `"host_path:container_path"` | Daftar mount point |

**Aturan validasi:**
- Setiap entry harus mengandung tepat satu karakter `:`
- Format wajib: `host_path:container_path`
- Path harus menggunakan path absolut

---

### Section `[lifecycle]` — Opsional

Mendefinisikan perintah yang dijalankan pada titik-titik tertentu dalam siklus hidup container.

```toml
[lifecycle]
setup = "bash /app/scripts/setup.sh"
start = "systemctl start myapp"
stop  = "systemctl stop myapp"
```

| Field | Kapan Dijalankan | Deskripsi |
|-------|-----------------|-----------|
| `setup` | Sekali saat deployment pertama | Inisialisasi aplikasi (migrasi database, build assets) |
| `start` | Setiap kali container distart | Menjalankan layanan utama |
| `stop` | Setiap kali container dihentikan | Menghentikan layanan dengan bersih |

**Perintah multi-baris:**
```toml
[lifecycle]
setup = """
cd /app &&
python manage.py migrate &&
python manage.py collectstatic --noinput &&
echo "Setup selesai!"
"""
```

---

### Section `[health]` — Opsional

Mendefinisikan health check untuk memverifikasi bahwa deployment berhasil.

```toml
[health]
command       = "curl -f http://localhost:8080/health"
retries       = 5
interval_secs = 10
timeout_secs  = 30
```

| Field | Tipe | Default | Deskripsi |
|-------|------|---------|-----------|
| `command` | String | — | Perintah yang dijalankan untuk cek kesehatan |
| `retries` | Integer | 3 | Jumlah percobaan sebelum dianggap gagal |
| `interval_secs` | Integer | 5 | Jeda antar percobaan (detik) |
| `timeout_secs` | Integer | 30 | Batas waktu total health check (detik) |

Health check dianggap berhasil jika perintah mengembalikan exit code `0`. Jika semua percobaan gagal, deployment dianggap gagal namun container tetap berjalan (untuk keperluan debugging).

---

### Section `[services.*]` — Opsional

Mendefinisikan layanan tambahan yang diperlukan oleh aplikasi.

```toml
[services.redis]
image = "redis/alpine"
ports = ["6379:6379"]

[services.postgres]
image = "postgres/alpine"
ports = ["5432:5432"]

[services.rabbitmq]
image = "rabbitmq/alpine"
ports = ["5672:5672", "15672:15672"]
```

| Field | Tipe | Deskripsi |
|-------|------|-----------|
| `image` | String | Identifikasi image/distro layanan |
| `ports` | Array String | Port yang diekspos layanan |

---

## Template File .mel

Salin template ini sebagai titik awal untuk proyek Anda:

```toml
# ================================
# MELISA Deployment Manifest
# File: deploy.mel
# ================================

[project]
name        = "nama-proyek-saya"
version     = "1.0.0"
description = "Deskripsi singkat proyek"
author      = "Nama Anda"

[container]
distro     = "ubuntu/jammy/amd64"
# name     = "nama-container-kustom"  # hapus komentar jika ingin nama kustom
auto_start = true

[env]
# KUNCI = "nilai"
APP_ENV = "production"

[dependencies]
apt = []    # Paket sistem
pip = []    # Library Python
npm = []    # Paket Node.js
# cargo = []
# gem = []

[ports]
expose = [
    # "80:80",
    # "443:443",
]

[volumes]
mounts = [
    # "/path/host:/path/container",
]

[lifecycle]
# setup = "perintah inisialisasi"
# start = "perintah start layanan"
# stop  = "perintah stop layanan"

# [health]
# command       = "curl -f http://localhost/"
# retries       = 5
# interval_secs = 10
# timeout_secs  = 60
```

---

## Penanganan Error Manifest

### Error: `NotFound`
```
[ERROR] File 'myapp.mel' not found.
Tip: Verify the path is correct. Example: melisa --up ./myapp/program.mel
```
**Solusi:** Periksa apakah path ke file `.mel` sudah benar.

### Error: `TomlParse`
```
[ERROR] Invalid .mel file:
  TOML parse error at line 12, column 1:
  expected key, found `.`
```
**Solusi:** Buka file `.mel` dan perbaiki sintaks TOML di baris yang disebutkan.

### Error: `Invalid` (Validasi)
```
[ERROR] Manifest validation error:
  Invalid port format '8080': expected 'host_port:container_port'
```
**Solusi:** Perbaiki format port menjadi `host_port:container_port`.

### Error: `Io`
```
[ERROR] IO error: Permission denied (os error 13)
```
**Solusi:** Pastikan Anda memiliki izin baca pada file `.mel`.