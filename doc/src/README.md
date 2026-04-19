# MELISA — Management Environment Linux Sandbox

<div style="text-align:center; padding: 2rem 0;">
  <pre style="display:inline-block; text-align:left; color:#00b4d8; font-size:0.85em;">
 ███╗   ███╗███████╗██╗     ██╗███████╗ █████╗
 ████╗ ████║██╔════╝██║     ██║██╔════╝██╔══██╗
 ██╔████╔██║█████╗  ██║     ██║███████╗███████║
 ██║╚██╔╝██║██╔══╝  ██║     ██║╚════██║██╔══██║
 ██║ ╚═╝ ██║███████╗███████╗██║███████║██║  ██║
 ╚═╝     ╚═╝╚══════╝╚══════╝╚═╝╚══════╝╚═╝  ╚═╝
  </pre>
  <p><strong>Management Environment Linux Sandbox</strong></p>
  <p>Versi <code>0.1.3</code> · Ditulis dalam Rust · Lisensi MIT</p>
</div>

---

## Apa itu MELISA?

**MELISA** adalah platform manajemen lingkungan sandbox berbasis **Linux Container (LXC)** yang ditulis sepenuhnya dalam bahasa Rust. MELISA dirancang untuk administrator sistem dan tim pengembang yang ingin mengelola container, pengguna, dan deployment aplikasi secara terpusat melalui antarmuka REPL (*Read-Eval-Print Loop*) yang aman dan intuitif.

MELISA terdiri dari dua komponen utama:

| Komponen | Deskripsi | Bahasa |
|----------|-----------|--------|
| **melisa** (server) | Daemon utama yang berjalan di server Linux dengan hak root, mengelola container LXC, pengguna sistem, dan proyek kolaboratif | Rust |
| **melisa-client** | Klien lintas platform (Linux, macOS, Windows) untuk terhubung ke server MELISA dari jarak jauh via SSH | Rust |

---

## Fitur Utama

### 🐧 Manajemen Container LXC
Buat, jalankan, hentikan, dan hapus container Linux dengan berbagai distribusi (Ubuntu, Debian, Fedora, Arch, Alpine, dan lainnya). Setiap container mendapat jaringan terisolasi melalui bridge `lxcbr0`.

### 👥 Manajemen Pengguna Multi-Level
MELISA mendukung dua peran pengguna:
- **Administrator** — akses penuh ke semua fitur termasuk manajemen pengguna dan proyek
- **Standard User** — akses ke manajemen container dan proyek personal

### 📁 Manajemen Proyek Kolaboratif
Proyek dikelola sebagai *bare Git repository* di `/var/melisa/projects`. Administrator dapat membuat proyek, mengundang anggota, mendistribusikan pembaruan, dan menarik workspace pengguna.

### 🚀 Deployment Deklaratif via `.mel`
File manifest berformat TOML (`.mel`) memungkinkan deployment aplikasi yang sepenuhnya deklaratif — mulai dari provisioning container, instalasi dependensi, konfigurasi port, mount volume, hingga health check.

### 🔒 Keamanan Berlapis
- Wajib berjalan sebagai root dengan escalasi via `sudo`
- Input Guard mencegah shell injection, path traversal, dan null-byte attack
- Konfigurasi SSH dengan izin ketat (600/700)
- Sudoers per-pengguna dengan konfigurasi granular

### 🌐 Klien Lintas Platform
Client MELISA berjalan native di Linux, macOS, dan Windows — tidak memerlukan dependensi tambahan selain SSH.

---

## Cara Kerja (Singkat)

```
[Pengguna/Admin]
      │
      ▼
[melisa-client] ──SSH──▶ [Server Linux]
                                │
                          [melisa REPL]
                                │
               ┌────────────────┼────────────────┐
               ▼                ▼                ▼
         [Container]      [Pengguna]        [Proyek]
           (LXC)          (Sistem)          (Git)
```

1. **Client** terhubung ke server melalui SSH menggunakan profil yang tersimpan
2. Shell pengguna di server diset ke binary `melisa`, sehingga setiap login langsung masuk ke MELISA REPL
3. Di dalam REPL, pengguna menjalankan perintah `melisa --<subcommand>` untuk mengelola sistem

---

## Persyaratan Sistem

### Server
| Komponen | Persyaratan |
|----------|-------------|
| OS | Linux (Debian/Ubuntu, Fedora/RHEL, Arch, Alpine, openSUSE) |
| Hak Akses | Root / sudo |
| LXC | Versi 3.x atau lebih baru |
| Rust | Edisi 2024 (untuk kompilasi) |
| Jaringan | Bridge `lxcbr0` (dikonfigurasi otomatis saat setup) |

### Client
| Platform | Persyaratan |
|----------|-------------|
| Linux | SSH client |
| macOS | SSH client (bawaan) |
| Windows | OpenSSH Client (Windows 10/11) |
| Rust | Edisi 2024 (untuk kompilasi dari source) |

---

## Memulai dengan Cepat

```bash
# 1. Kompilasi dan install binary di server
cargo build --release
sudo cp target/release/melisa /usr/local/bin/melisa

# 2. Jalankan setup host (perlu root)
sudo melisa --setup

# 3. Di mesin lokal, install client dan tambah profil server
melisa auth add production root@192.168.1.100

# 4. Buat container pertama
melisa --search ubuntu
melisa --create webserver ubuntu/jammy/amd64
melisa --run webserver
melisa --use webserver
```

---

## Struktur Dokumentasi

Dokumentasi ini dibagi menjadi beberapa bagian:

- **Memulai** — Instalasi, setup awal, dan koneksi pertama
- **Panduan Penggunaan** — Tutorial lengkap untuk setiap fitur
- **Referensi** — Daftar perintah lengkap dan format file
- **Arsitektur** — Penjelasan teknis internal sistem

> 💡 **Tip:** Gunakan sidebar di sebelah kiri untuk navigasi, atau gunakan fitur pencarian (ikon 🔍) untuk menemukan topik spesifik.