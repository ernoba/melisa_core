# Instalasi Client MELISA

Client MELISA (`melisa-client`) adalah aplikasi command-line lintas platform yang memungkinkan Anda terhubung ke server MELISA dari mesin lokal. Client ditulis ulang dari Bash ke Rust untuk mendukung Linux, macOS, dan Windows secara native.

---

## Cara Kerja Client

Client MELISA bertindak sebagai jembatan antara mesin lokal Anda dan server MELISA:

```
[Mesin Lokal]                    [Server Linux]
melisa auth add myserver user@ip ──▶ Deploy SSH key
melisa exec <command>            ──SSH▶ Jalankan di server
```

Client menyimpan profil koneksi (nama server, alamat SSH, username MELISA) di direktori konfigurasi lokal dan menggunakan SSH multiplexing untuk koneksi yang efisien.

---

## Instalasi di Linux / macOS

### Dari Source (Disarankan)

```bash
# Clone repositori client
git clone https://github.com/ernoba/melisa_core.git
cd melisa_core/client

# Kompilasi
cargo build --release

# Install ke PATH
cp target/release/melisa ~/.local/bin/melisa
# atau
sudo cp target/release/melisa /usr/local/bin/melisa
```

Setelah instalasi, muat ulang shell:
```bash
source ~/.bashrc
# atau
source ~/.zshrc
```

Verifikasi:
```bash
melisa --version
```

### Menggunakan Script Instalasi

Script `install.sh` tersedia untuk instalasi otomatis:

```bash
cd client/
bash install.sh
```

Script ini akan:
1. Memeriksa ketersediaan Rust toolchain
2. Mengkompilasi binary client
3. Menginstall ke `~/.local/bin/`
4. Menambahkan direktori ke PATH jika belum ada

---

## Instalasi di Windows

### Persyaratan Windows

- Windows 10 versi 1809 atau lebih baru / Windows 11
- PowerShell 5.1+ (sudah tersedia secara default)
- OpenSSH Client (fitur opsional Windows)

### Menggunakan Script PowerShell

```powershell
# Buka PowerShell (tidak perlu Administrator)
cd client\
.\install.ps1
```

Script PowerShell ini akan:

1. **Cek versi Windows** — memastikan kompatibilitas
2. **Install OpenSSH Client** — jika belum tersedia sebagai Windows Capability
3. **Install Rust toolchain** — jika belum ada via `winget` atau `rustup-init.exe`
4. **Kompilasi binary** — `cargo build --release`
5. **Install ke direktori user** — `%LOCALAPPDATA%\Programs\melisa\`
6. **Tambah ke PATH** — memperbarui PATH pengguna secara permanen

#### Opsi PowerShell

```powershell
# Instalasi tanpa kompilasi (menggunakan binary pre-built)
.\install.ps1 -NoBuild

# Lewati pemeriksaan OpenSSH
.\install.ps1 -SkipOpenSsh
```

Setelah instalasi selesai, buka terminal baru agar perubahan PATH berlaku:
```powershell
melisa --help
```

---

## Konfigurasi Awal Client

### Struktur Direktori Konfigurasi

Client MELISA menyimpan konfigurasi di:

| Platform | Lokasi |
|----------|--------|
| Linux/macOS | `~/.config/melisa/` |
| Windows | `%APPDATA%\melisa\` |

File yang disimpan:
- `profiles.conf` — daftar profil koneksi server (hak akses 600)
- `active` — nama profil yang sedang aktif

### Keamanan File Konfigurasi

File konfigurasi dilindungi dengan izin ketat:
- Direktori konfigurasi: **700** (hanya pemilik yang bisa akses)
- File `profiles.conf`: **600** (hanya pemilik yang bisa baca/tulis)

---

## Perintah Client MELISA

Setelah terinstall, berikut perintah-perintah utama client:

### Manajemen Profil Koneksi

```bash
# Tambah profil server baru
melisa auth add <nama_profil> <user@ip_server>
# Contoh:
melisa auth add produksi root@192.168.1.100
melisa auth add development admin@10.0.0.5

# Tampilkan semua profil
melisa auth list

# Ganti profil aktif
melisa auth switch <nama_profil>
melisa auth switch development

# Hapus profil
melisa auth remove <nama_profil>
```

### Eksekusi Perintah di Server

```bash
# Eksekusi satu perintah di server aktif
melisa exec <perintah>

# Contoh
melisa exec "melisa --list"
melisa exec "melisa --create myapp ubuntu/jammy/amd64"
```

### Informasi Koneksi

```bash
# Tampilkan profil yang aktif dan informasi koneksi
melisa status
```

---

## Proses `auth add` Secara Detail

Saat Anda menjalankan `melisa auth add`, proses berikut terjadi secara otomatis:

1. **Validasi input** — memvalidasi nama profil dan format `user@host`
2. **Generate SSH key** — membuat pasangan kunci SSH jika belum ada
3. **Deploy public key** — mengirim public key ke server via `ssh-copy-id`
4. **Konfigurasi SSH Multiplexing** — mengoptimalkan koneksi berulang (jika tersedia)
5. **Input MELISA username** — meminta username MELISA di server (boleh berbeda dari SSH user)
6. **Simpan profil** — menyimpan konfigurasi ke `profiles.conf`
7. **Set sebagai aktif** — profil baru langsung diset sebagai profil aktif

```
$ melisa auth add produksi deploy@192.168.1.100
[INFO] Deploying public SSH key to deploy@192.168.1.100...
[INFO] Please prepare to enter the remote server password.
deploy@192.168.1.100's password: ●●●●●●●●
[SETUP] Enter your MELISA username on this server (leave blank to use SSH user 'deploy'):
> adminku
[SUCCESS] Server profile 'produksi' registered. Remote MELISA user: adminku
```

---

## Troubleshooting Client

### Error: `ssh-copy-id` tidak ditemukan (macOS lama)

Beberapa versi macOS tidak menyertakan `ssh-copy-id`. Install via Homebrew:
```bash
brew install ssh-copy-id
```

### Error: Koneksi ditolak setelah `auth add`

Pastikan:
1. Server SSH berjalan: `sudo systemctl status ssh`
2. Port 22 tidak diblokir firewall server
3. Alamat IP dan username benar

### Error di Windows: OpenSSH tidak ditemukan

Aktifkan OpenSSH Client melalui Settings:
```
Settings → Apps → Optional Features → Add a feature → OpenSSH Client
```

Atau via PowerShell sebagai Administrator:
```powershell
Add-WindowsCapability -Online -Name OpenSSH.Client~~~~0.0.1.0
```