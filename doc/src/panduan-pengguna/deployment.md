# Deployment dengan File .mel

MELISA mendukung deployment aplikasi yang sepenuhnya deklaratif menggunakan file manifest berformat TOML dengan ekstensi `.mel`. File ini mendefinisikan semua aspek deployment — dari provisioning container hingga konfigurasi layanan, dependensi, port, volume, dan health check.

---

## Konsep Deployment

File `.mel` adalah "resep" deployment Anda. Dengan satu perintah, MELISA akan:

1. **Membuat** container jika belum ada
2. **Menginstall** semua dependensi (sistem dan bahasa pemrograman)
3. **Mengkonfigurasi** port forwarding dan volume mount
4. **Menjalankan** lifecycle hooks (setup, start, stop)
5. **Melakukan** health check untuk memastikan deployment berhasil

```
melisa --up ./myapp/deploy.mel
```

---

## Struktur File .mel

File `.mel` menggunakan format TOML dengan beberapa section utama:

```toml
# ═══════════════════════════════════════════════
# Section [project] — Informasi proyek (WAJIB)
# ═══════════════════════════════════════════════
[project]
name        = "mywebapp"          # nama unik proyek (wajib)
version     = "1.2.0"            # versi (opsional)
description = "Web app saya"     # deskripsi (opsional)
author      = "John Doe"          # nama author (opsional)

# ═══════════════════════════════════════════════
# Section [container] — Konfigurasi container (WAJIB)
# ═══════════════════════════════════════════════
[container]
distro     = "ubuntu/jammy/amd64"  # kode distro (wajib)
name       = "mywebapp-prod"        # nama container (opsional, default: nama proyek)
auto_start = true                   # start otomatis setelah dibuat (default: true)

# ═══════════════════════════════════════════════
# Section [env] — Variabel lingkungan (opsional)
# ═══════════════════════════════════════════════
[env]
DATABASE_URL = "postgres://localhost/mydb"
APP_PORT     = "8080"
NODE_ENV     = "production"
SECRET_KEY   = "rahasia-saya"

# ═══════════════════════════════════════════════
# Section [dependencies] — Dependensi (opsional)
# ═══════════════════════════════════════════════
[dependencies]
apt    = ["nginx", "curl", "git", "postgresql-client"]
pip    = ["django", "gunicorn", "psycopg2-binary"]
npm    = ["pm2"]
cargo  = []
gem    = []

# ═══════════════════════════════════════════════
# Section [ports] — Port forwarding (opsional)
# ═══════════════════════════════════════════════
[ports]
expose = [
    "8080:8080",   # host_port:container_port
    "443:443",
    "5432:5432",
]

# ═══════════════════════════════════════════════
# Section [volumes] — Mount folder (opsional)
# ═══════════════════════════════════════════════
[volumes]
mounts = [
    "/data/myapp:/app/data",          # host_path:container_path
    "/var/log/myapp:/app/logs",
]

# ═══════════════════════════════════════════════
# Section [lifecycle] — Perintah lifecycle (opsional)
# ═══════════════════════════════════════════════
[lifecycle]
setup = "bash /app/scripts/setup.sh"   # dijalankan sekali saat setup
start = "systemctl start gunicorn"     # dijalankan saat container start
stop  = "systemctl stop gunicorn"      # dijalankan saat container stop

# ═══════════════════════════════════════════════
# Section [health] — Health check (opsional)
# ═══════════════════════════════════════════════
[health]
command       = "curl -f http://localhost:8080/health"
retries       = 5
interval_secs = 10
timeout_secs  = 30

# ═══════════════════════════════════════════════
# Section [services.*] — Layanan tambahan (opsional)
# ═══════════════════════════════════════════════
[services.redis]
image = "redis/alpine"
ports = ["6379:6379"]

[services.postgres]
image = "postgres/alpine"
ports = ["5432:5432"]
```

---

## Validasi File .mel

MELISA memvalidasi file `.mel` secara ketat sebelum eksekusi:

| Field | Aturan Validasi |
|-------|-----------------|
| `[project].name` | Wajib ada, tidak boleh kosong |
| `[container].distro` | Wajib ada, format `distro/rilis/arsitektur` |
| `[ports].expose` | Setiap entry harus format `host:container` |
| `[volumes].mounts` | Setiap entry harus format `host_path:container_path` |

Jika validasi gagal, MELISA menampilkan pesan error yang jelas:
```
[ERROR] Invalid .mel file:
  TOML parse error at line 15, column 1: ...

Tip: Verify the path is correct. Example: melisa --up ./myapp/program.mel
```

---

## Perintah Deployment

### Deploy Aplikasi

```
melisa --up <path/ke/file.mel>
```

```bash
# Deploy dengan file di direktori saat ini
melisa --up ./deploy.mel

# Deploy dari path spesifik
melisa --up /home/user/myapp/production.mel

# Deploy dengan mode audit (tampilkan semua perintah)
melisa --up ./deploy.mel --audit
```

### Proses Deployment (7 Tahap)

```
━━━ MELISA DEPLOYMENT ENGINE ━━━
[UP] Reading manifest: ./deploy.mel

[MANIFEST SUMMARY]
  Project  : mywebapp v1.2.0
  Container: mywebapp-prod (ubuntu/jammy/amd64)
  Deps     : nginx, curl, git (apt) | django, gunicorn (pip)
  Ports    : 8080→8080, 443→443
  Volumes  : /data/myapp → /app/data

[STEP 1/7] Provisioning new container 'mywebapp-prod'...
  [OK] Container created.

[STEP 2/7] Starting container...
  [OK] Container started. IP: 10.0.3.18

[STEP 3/7] Installing system dependencies (apt)...
  → apt-get install -y nginx curl git postgresql-client
  [OK] System packages installed.

[STEP 4/7] Installing language dependencies (pip)...
  → pip3 install django gunicorn psycopg2-binary
  [OK] Python packages installed.

[STEP 5/7] Configuring port forwarding...
  → iptables -t nat -A PREROUTING -p tcp --dport 8080 -j DNAT --to 10.0.3.18:8080
  [OK] Port 8080 forwarded.

[STEP 6/7] Mounting volumes...
  [OK] /data/myapp mounted to /app/data.

[STEP 7/7] Running health check...
  [Attempt 1/5] curl -f http://localhost:8080/health... [OK]

[SUCCESS] Deployment complete! 'mywebapp-prod' is live.
```

### Menghentikan Deployment

```
melisa --down <path/ke/file.mel>
```

```bash
melisa --down ./deploy.mel
```

Perintah ini akan menghentikan container yang terkait dengan manifest tersebut dan menjalankan lifecycle hook `stop` jika dikonfigurasi.

### Inspeksi Manifest

```
melisa --mel-info <path/ke/file.mel>
```

```bash
melisa --mel-info ./deploy.mel
```

Menampilkan ringkasan manifest tanpa melakukan deployment:
```
[INFO] Manifest: ./deploy.mel

  Project   : mywebapp (v1.2.0)
  Author    : John Doe
  Container : mywebapp-prod → ubuntu/jammy/amd64
  Auto Start: yes

  Dependencies:
    apt    : nginx, curl, git, postgresql-client
    pip    : django, gunicorn, psycopg2-binary
    npm    : pm2

  Ports     : 8080:8080, 443:443, 5432:5432
  Volumes   : /data/myapp:/app/data, /var/log/myapp:/app/logs

  Lifecycle :
    setup  : bash /app/scripts/setup.sh
    start  : systemctl start gunicorn
    stop   : systemctl stop gunicorn

  Health    : curl -f http://localhost:8080/health
              Retries: 5, Interval: 10s, Timeout: 30s
```

---

## Dependensi yang Didukung

MELISA mendeteksi package manager secara otomatis dan mendukung instalasi dari berbagai ekosistem:

| Section | Package Manager | Contoh |
|---------|-----------------|--------|
| `apt` | APT (Debian/Ubuntu) | `nginx`, `postgresql-client` |
| `pacman` | Pacman (Arch) | `nginx`, `python` |
| `dnf` | DNF (Fedora/RHEL) | `nginx`, `python3-pip` |
| `zypper` | Zypper (openSUSE) | `nginx`, `python3-pip` |
| `apk` | APK (Alpine) | `nginx`, `py3-pip` |
| `pip` | pip (Python) | `django`, `flask`, `gunicorn` |
| `npm` | npm (Node.js) | `pm2`, `express` |
| `cargo` | Cargo (Rust) | `mdbook`, `tokei` |
| `gem` | Gem (Ruby) | `rails`, `sinatra` |
| `composer` | Composer (PHP) | `laravel/laravel` |

---

## Contoh File .mel Lengkap

### Contoh: Aplikasi Python/Django

```toml
[project]
name        = "django-app"
version     = "2.0.0"
description = "Aplikasi Django dengan PostgreSQL"
author      = "Tim Backend"

[container]
distro = "ubuntu/jammy/amd64"
name   = "django-prod"

[env]
DATABASE_URL = "postgres://appuser:secret@localhost/appdb"
DJANGO_SECRET_KEY = "super-secret-production-key"
DEBUG = "False"
ALLOWED_HOSTS = "yourdomain.com"

[dependencies]
apt = ["python3-pip", "python3-venv", "postgresql", "nginx"]
pip = ["django>=4.2", "gunicorn", "psycopg2-binary", "whitenoise"]

[ports]
expose = ["80:80", "443:443"]

[volumes]
mounts = [
    "/data/django/media:/app/media",
    "/data/django/static:/app/static",
]

[lifecycle]
setup = """
cd /app && 
python manage.py migrate &&
python manage.py collectstatic --noinput
"""
start = "systemctl start gunicorn nginx"
stop  = "systemctl stop gunicorn"

[health]
command       = "curl -sf http://localhost/health/"
retries       = 10
interval_secs = 5
timeout_secs  = 60
```

### Contoh: Aplikasi Node.js

```toml
[project]
name    = "node-api"
version = "1.0.0"

[container]
distro = "ubuntu/jammy/amd64"
name   = "nodeapi-prod"

[env]
PORT      = "3000"
NODE_ENV  = "production"
JWT_SECRET = "my-jwt-secret"

[dependencies]
apt = ["curl"]
npm = ["pm2"]

[ports]
expose = ["3000:3000"]

[volumes]
mounts = ["/var/log/nodeapi:/app/logs"]

[lifecycle]
setup = "npm install --production"
start = "pm2 start ecosystem.config.js"
stop  = "pm2 stop all"

[health]
command       = "curl -f http://localhost:3000/api/health"
retries       = 5
interval_secs = 8
timeout_secs  = 30
```

### Contoh: Static Site dengan Nginx

```toml
[project]
name    = "static-site"
version = "1.0.0"

[container]
distro = "alpine/3.18/amd64"

[dependencies]
apk = ["nginx"]

[ports]
expose = ["80:80"]

[volumes]
mounts = ["/var/www/mysite:/usr/share/nginx/html:ro"]

[lifecycle]
start = "nginx -g 'daemon off;'"
stop  = "nginx -s quit"

[health]
command       = "wget -qO- http://localhost/index.html"
retries       = 3
interval_secs = 5
timeout_secs  = 15
```