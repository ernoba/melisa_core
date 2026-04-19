# Client CLI (melisa-client)

`melisa-client` adalah aplikasi command-line lintas platform yang memungkinkan Anda mengelola server MELISA dari mesin lokal tanpa perlu SSH secara manual. Client ini ditulis sepenuhnya dalam Rust dan mendukung Linux, macOS, dan Windows.

---

## Instalasi

Lihat [Instalasi Client](../panduan-pengguna/instalasi-client.md) untuk panduan lengkap instalasi di setiap platform.

---

## Perintah Lengkap Client

### `melisa auth add`

Mendaftarkan profil koneksi server baru dan men-deploy SSH key.

```bash
melisa auth add <nama_profil> <user@server>
```

**Parameter:**
- `<nama_profil>` — Nama unik untuk profil ini
- `<user@server>` — SSH connection string ke server MELISA

**Contoh:**
```bash
melisa auth add produksi root@192.168.1.100
melisa auth add staging deploy@staging.example.com
melisa auth add dev admin@10.0.0.5
```

**Proses yang terjadi:**
1. Validasi nama profil dan format `user@server`
2. Generate SSH key pair jika belum ada di `~/.ssh/`
3. Deploy public key ke server via `ssh-copy-id`
4. Konfigurasi SSH multiplexing (jika tersedia)
5. Tanya username MELISA di server
6. Simpan profil ke `~/.config/melisa/profiles.conf`
7. Set profil ini sebagai profil aktif

**Validasi nama profil:**
- Panjang: 1–64 karakter
- Karakter: `a-z`, `A-Z`, `0-9`, `-`, `_`
- Tidak boleh dimulai dengan `-`

**Validasi format `user@server`:**
- Wajib mengandung karakter `@`
- Format: `username@hostname_atau_ip`

---

### `melisa auth remove`

Menghapus profil koneksi dari registry lokal.

```bash
melisa auth remove <nama_profil>
```

**Contoh:**
```bash
melisa auth remove staging
```

Sistem akan meminta konfirmasi:
```
[WARNING] Are you sure you want to permanently remove the profile 'staging'? (y/N): y
[SUCCESS] Profile 'staging' removed.
[INFO] The active profile was deleted. Use 'melisa auth switch' to select a new server.
```

> ℹ️ Menghapus profil **tidak** menghapus SSH key atau konfigurasi di server. Hanya entri di `profiles.conf` lokal yang dihapus.

---

### `melisa auth list`

Menampilkan semua profil koneksi yang terdaftar.

```bash
melisa auth list
```

**Output:**
```
Profil MELISA yang terdaftar:

  * produksi   →  root@192.168.1.100      [MELISA user: adminku]  [AKTIF]
    dev        →  deploy@10.0.0.5         [MELISA user: deploy]
    staging    →  admin@10.0.0.10        [MELISA user: adminku]

Gunakan 'melisa auth switch <nama>' untuk ganti profil aktif.
```

---

### `melisa auth switch`

Mengganti profil koneksi yang sedang aktif.

```bash
melisa auth switch <nama_profil>
```

**Contoh:**
```bash
melisa auth switch dev
melisa auth switch produksi
```

**Output:**
```
[INFO] Active server profile switched to 'dev'.
```

Profil aktif disimpan di file `~/.config/melisa/active` (Linux/macOS) atau `%APPDATA%\melisa\active` (Windows).

---

### `melisa exec`

Mengirim dan mengeksekusi perintah di server yang aktif melalui SSH.

```bash
melisa exec <perintah>
```

**Contoh:**
```bash
# Lihat daftar container
melisa exec "melisa --list"

# Buat container baru
melisa exec "melisa --create myapp ubuntu/jammy/amd64"

# Deploy aplikasi
melisa exec "melisa --up /home/admin/myapp/deploy.mel"

# Cek status server
melisa exec "df -h && free -h"
```

---

### `melisa status`

Menampilkan informasi tentang profil koneksi yang aktif.

```bash
melisa status
```

**Output:**
```
Profil Aktif  : produksi
Koneksi SSH   : root@192.168.1.100
MELISA User   : adminku
Status        : Terhubung ✓
```

---

## Konfigurasi SSH yang Dibuat Otomatis

Saat `auth add` dijalankan, client MELISA mengkonfigurasi SSH untuk koneksi yang efisien.

### SSH Multiplexing

Jika SSH lokal mendukung multiplexing, konfigurasi berikut ditambahkan ke `~/.ssh/config`:

```
Host 192.168.1.100
    ControlMaster auto
    ControlPath ~/.ssh/melisa-sockets/%r@%h:%p
    ControlPersist 10m
```

Ini memungkinkan koneksi SSH pertama menjadi *master connection*, dan semua koneksi berikutnya berbagi koneksi yang sama — jauh lebih cepat dan efisien.

### SSH Key yang Digunakan

Client MELISA menggunakan SSH key di lokasi standar:

| Platform | Lokasi Key |
|----------|-----------|
| Linux/macOS | `~/.ssh/id_rsa` atau `~/.ssh/id_ed25519` |
| Windows | `%USERPROFILE%\.ssh\id_rsa` atau `%USERPROFILE%\.ssh\id_ed25519` |

Jika belum ada SSH key, client akan membuatnya secara otomatis.

---

## Lokasi File Konfigurasi

| Platform | Direktori Konfigurasi | Hak Akses |
|----------|-----------------------|-----------|
| Linux | `~/.config/melisa/` | 700 |
| macOS | `~/.config/melisa/` | 700 |
| Windows | `%APPDATA%\melisa\` | — |

**File di dalam direktori konfigurasi:**

| File | Hak Akses | Deskripsi |
|------|-----------|-----------|
| `profiles.conf` | 600 | Daftar semua profil koneksi |
| `active` | 600 | Nama profil yang sedang aktif |

**Format `profiles.conf`:**
```
# Format: nama_profil=user@server|melisa_user
produksi=root@192.168.1.100|adminku
dev=deploy@10.0.0.5|deploy
staging=admin@10.0.0.10|adminku
```

---

## Keamanan Client

### Perlindungan File Konfigurasi

- **Direktori konfigurasi** diset ke izin `700` (Unix) — hanya pemilik yang bisa membaca
- **File `profiles.conf`** diset ke izin `600` (Unix) — hanya pemilik yang bisa membaca/menulis
- File ditulis menggunakan pola *atomic write* (tulis ke `.tmp`, lalu rename) untuk mencegah korupsi data

### Validasi Keamanan

Semua nilai yang disimpan ke konfigurasi SSH divalidasi untuk mencegah injeksi:

```
# Karakter yang dilarang dalam nilai konfigurasi SSH:
\n  \r  \0  #
```

Ini mencegah injeksi konfigurasi SSH berbahaya yang bisa dieksploitasi untuk koneksi ke server yang tidak diinginkan.

### Autentikasi Berbasis Kunci

Setelah `auth add`, semua autentikasi dilakukan menggunakan kunci SSH publik/privat — tidak ada password yang disimpan di konfigurasi lokal.

---

## Contoh Skenario Penggunaan

### Skenario: Administrator Mengelola Beberapa Server

```bash
# Daftarkan semua server
melisa auth add prod-web     deploy@web.prod.com
melisa auth add prod-db      deploy@db.prod.com
melisa auth add staging      deploy@staging.com
melisa auth add dev-server   dev@192.168.1.50

# Kerja di staging
melisa auth switch staging
melisa exec "melisa --list"
melisa exec "melisa --up /home/deploy/app/staging.mel"

# Cek produksi
melisa auth switch prod-web
melisa exec "melisa --active"
melisa exec "melisa --info webapp-container"
```

### Skenario: CI/CD Pipeline

```bash
#!/bin/bash
# Script deployment otomatis

# Pastikan profil sudah terdaftar
melisa auth switch produksi

# Deploy versi terbaru
melisa exec "melisa --down /home/deploy/app/deploy.mel"
melisa exec "melisa --up /home/deploy/app/deploy.mel"

# Verifikasi
melisa exec "melisa --active"
echo "Deployment selesai!"
```

### Skenario: Developer Baru Bergabung

```bash
# Developer baru setup koneksi ke server
melisa auth add company-server devbaru@melisa.company.com

# Mulai bekerja
melisa exec "melisa --update myproject"
melisa exec "melisa --create mycontainer ubuntu/jammy/amd64"
```