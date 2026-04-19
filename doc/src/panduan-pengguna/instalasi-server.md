# Instalasi Server MELISA

Server MELISA adalah binary Rust yang harus berjalan di mesin Linux dengan akses root. Binary ini akan menjadi shell login pengguna MELISA, sehingga setiap pengguna yang login via SSH langsung masuk ke lingkungan MELISA yang terisolasi.

---

## Persyaratan Server

| Komponen | Persyaratan Minimum |
|----------|---------------------|
| Sistem Operasi | Linux (lihat daftar distribusi yang didukung) |
| Arsitektur | x86_64 (amd64) |
| Hak Akses | Root atau pengguna dengan sudo penuh |
| RAM | 512 MB (minimum), 2 GB (direkomendasikan) |
| Disk | 10 GB (untuk OS dan container) |
| Rust | Edisi 2024 (`rustup` versi terbaru) |
| LXC | Versi 3.x+ (diinstall otomatis saat setup) |
| SSH Server | OpenSSH (diinstall otomatis saat setup) |

### Distribusi Linux yang Didukung

| Distribusi | Package Manager | Firewall |
|------------|-----------------|----------|
| Debian / Ubuntu / Mint | `apt-get` | UFW |
| Fedora / RHEL / Rocky / AlmaLinux | `dnf` | Firewalld |
| Arch Linux / Manjaro | `pacman` | iptables |
| Alpine Linux | `apk` | iptables |
| openSUSE | `zypper` | Firewalld |
| OrbStack (VM) | `apt-get` | UFW + iptables |

---

## Langkah 1: Install Rust Toolchain

Jika Rust belum terinstall di server:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default stable
```

Verifikasi instalasi:
```bash
rustc --version
cargo --version
```

---

## Langkah 2: Clone dan Kompilasi

```bash
# Clone repositori
git clone https://github.com/ernoba/melisa_core.git
cd melisa_core

# Kompilasi dalam mode release (dioptimasi, ukuran kecil)
cargo build --release
```

> ⏳ Proses kompilasi pertama membutuhkan waktu beberapa menit karena harus mengunduh dan mengkompilasi semua dependensi.

Binary hasil kompilasi akan berada di `target/release/melisa`.

---

## Langkah 3: Install Binary ke Sistem

```bash
# Copy binary ke lokasi sistem
sudo cp target/release/melisa /usr/local/bin/melisa

# Verifikasi
melisa --about
```

> ⚠️ **Penting:** Binary harus berada di `/usr/local/bin/melisa` karena path ini digunakan sebagai shell login pengguna MELISA.

---

## Langkah 4: Jalankan Setup Host

Setelah binary terinstall, jalankan perintah setup untuk mengkonfigurasi server secara otomatis:

```bash
sudo melisa --setup
```

Perintah ini akan melakukan hal-hal berikut secara otomatis:

1. **Deteksi distribusi Linux** — MELISA mendeteksi distro host dan memilih konfigurasi yang sesuai
2. **Install paket LXC** — menginstall LXC, template, bridge-utils, dan dnsmasq
3. **Install SSH Server** — memastikan OpenSSH server terinstall dan berjalan
4. **Copy binary ke sistem** — memastikan binary melisa ada di `/usr/local/bin/`
5. **Konfigurasi firewall** — membuka port SSH di UFW/Firewalld/iptables
6. **Setup jaringan LXC** — mengkonfigurasi bridge `lxcbr0` dan NAT routing
7. **Registrasi shell** — mendaftarkan `/usr/local/bin/melisa` ke `/etc/shells`
8. **Konfigurasi sudoers** — mengatur izin sudo untuk operasi LXC

### Peringatan: Sesi SSH Remote

Jika Anda menjalankan setup melalui SSH, MELISA akan menampilkan peringatan keamanan karena konfigurasi firewall bisa menyebabkan lockout:

```
[BLOCKED] Sesi SSH Remote terdeteksi.
[SAFETY] Setup dihentikan untuk mencegah lockout firewall.
[INFO] Jika Anda yakin, jalankan kembali dengan: melisa --setup --force-unsafe
```

Untuk melanjutkan setup melalui SSH (gunakan dengan hati-hati):

```bash
sudo melisa --setup --force-unsafe
```

---

## Langkah 5: Verifikasi Instalasi

Setelah setup selesai, verifikasi bahwa semua komponen berjalan dengan benar:

```bash
# Cek LXC bridge
ip link show lxcbr0

# Cek SSH server
systemctl status ssh   # atau sshd di beberapa distro

# Cek melisa terdaftar sebagai shell valid
grep melisa /etc/shells

# Buat pengguna MELISA pertama (admin)
sudo melisa
melisa --add adminpertama
```

---

## Langkah 6: Buat Pengguna Admin Pertama

Setelah masuk ke MELISA REPL sebagai root:

```
melisa --add namaadmin
```

Sistem akan meminta:
1. Pilihan peran: `1` untuk Administrator, `2` untuk Standard User
2. Password untuk pengguna baru

---

## Troubleshooting Instalasi Server

### Error: `lxcbr0` tidak ditemukan setelah setup

```bash
# Restart layanan lxc-net
sudo systemctl restart lxc-net

# Jika gagal, cek log
sudo journalctl -u lxc-net -n 50
```

### Error: Kompilasi gagal karena versi Rust terlalu lama

```bash
rustup update stable
cargo build --release
```

### Setup gagal di lingkungan virtual (VPS/VM)

MELISA mendeteksi lingkungan virtual secara otomatis menggunakan `systemd-detect-virt`. Jika berjalan di OrbStack, Docker, atau container lain, MELISA akan menerapkan konfigurasi kompatibilitas khusus secara otomatis.

### Binary tidak bisa dieksekusi (permission denied)

```bash
sudo chmod +x /usr/local/bin/melisa
```