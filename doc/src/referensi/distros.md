# Distribusi yang Didukung

MELISA mendukung dua lapisan distribusi Linux: distribusi **host** (server yang menjalankan MELISA) dan distribusi **container** (sistem operasi di dalam container LXC).

---

## Distribusi Host yang Didukung

Distribusi host adalah sistem operasi yang diinstall di server fisik atau VM tempat MELISA berjalan.

| Distribusi | Package Manager | Firewall Default | Keterangan |
|------------|-----------------|-----------------|-----------|
| **Debian / Ubuntu / Linux Mint / Raspbian** | `apt-get` | UFW | Direkomendasikan untuk pemula |
| **Fedora / RHEL / CentOS / Rocky Linux / AlmaLinux** | `dnf` | Firewalld | Cocok untuk lingkungan enterprise |
| **Arch Linux / Manjaro** | `pacman` | iptables | Rolling release, selalu terbaru |
| **Alpine Linux** | `apk` | iptables | Sangat ringan, ideal untuk server minimal |
| **openSUSE** | `zypper` | Firewalld | Distro enterprise dari Jerman |
| **OrbStack** | `apt-get` | UFW + iptables | Lingkungan VM virtualisasi di macOS |

### Konfigurasi Per Distro

#### Debian / Ubuntu (dan turunannya)

```toml
pkg_manager  = "apt-get"
lxc_packages = ["lxc", "lxc-templates", "uidmap", "bridge-utils", "dnsmasq"]
firewall     = UFW
```

Perintah instalasi LXC yang dijalankan:
```bash
apt-get install -y lxc lxc-templates uidmap bridge-utils dnsmasq
```

#### Fedora / RHEL / Rocky Linux

```toml
pkg_manager  = "dnf"
lxc_packages = ["lxc", "lxc-templates", "lxc-extra", "dnsmasq"]
firewall     = Firewalld
```

#### Arch Linux

```toml
pkg_manager  = "pacman"
lxc_packages = ["lxc", "bridge-utils"]
firewall     = iptables
```

#### Alpine Linux

```toml
pkg_manager  = "apk"
lxc_packages = ["lxc", "lxc-templates"]
firewall     = iptables
```

#### openSUSE

```toml
pkg_manager  = "zypper"
lxc_packages = ["lxc", "lxc-templates", "bridge-utils"]
firewall     = Firewalld
```

#### OrbStack (Deteksi Otomatis)

OrbStack adalah environment virtualisasi ringan untuk macOS. MELISA mendeteksi OrbStack melalui string `orbstack` di `/etc/os-release` dan menerapkan konfigurasi kompatibilitas khusus:

- Override `lxc-net.service` dengan menghapus `ConditionVirtualization` agar lxc-net bisa berjalan di VM
- Konfigurasi bridge `lxcbr0` secara manual
- Instalasi tambahan: `ufw`, `iptables`

---

## Distribusi Container yang Didukung

Distribusi container adalah sistem operasi yang berjalan di **dalam** container LXC. Daftar lengkap tersedia dari repositori LXC resmi dan dapat dicari menggunakan:

```bash
melisa --search [kata_kunci]
```

### Format Kode Distribusi

```
distribusi/rilis/arsitektur
```

Contoh:
```
ubuntu/jammy/amd64
ubuntu/focal/arm64
debian/bookworm/amd64
alpine/3.18/amd64
fedora/38/amd64
archlinux/current/amd64
```

### Daftar Distribusi Populer

| Distribusi | Rilis Populer | Arsitektur | Kode MELISA |
|------------|---------------|------------|-------------|
| **Ubuntu** | 22.04 LTS (Jammy) | amd64, arm64 | `ubuntu/jammy/amd64` |
| **Ubuntu** | 20.04 LTS (Focal) | amd64, arm64 | `ubuntu/focal/amd64` |
| **Ubuntu** | 24.04 LTS (Noble) | amd64, arm64 | `ubuntu/noble/amd64` |
| **Debian** | 12 (Bookworm) | amd64, arm64 | `debian/bookworm/amd64` |
| **Debian** | 11 (Bullseye) | amd64, arm64 | `debian/bullseye/amd64` |
| **Alpine** | 3.18 | amd64 | `alpine/3.18/amd64` |
| **Alpine** | 3.17 | amd64 | `alpine/3.17/amd64` |
| **Fedora** | 38 | amd64 | `fedora/38/amd64` |
| **Fedora** | 39 | amd64 | `fedora/39/amd64` |
| **Arch Linux** | Current | amd64 | `archlinux/current/amd64` |
| **openSUSE** | Leap 15.5 | amd64 | `opensuse/15.5/amd64` |
| **Kali Linux** | Current | amd64 | `kali/current/amd64` |

> ℹ️ **Catatan:** Daftar distribusi di atas adalah contoh umum. Daftar aktual bergantung pada ketersediaan di server template LXC. Gunakan `melisa --search` untuk daftar terkini.

### Pemilihan Package Manager di Container

MELISA mendeteksi package manager yang sesuai secara otomatis berdasarkan nama distribusi container, menggunakan logika berikut:

```
ubuntu, debian, kali, mint, raspbian, linuxmint  →  apt
fedora, centos, rhel, rocky, alma               →  dnf
alpine                                           →  apk
arch, manjaro                                    →  pacman
suse, opensuse                                   →  zypper
lainnya                                          →  apt (default aman)
```

---

## Deteksi Lingkungan Virtual

MELISA secara otomatis mendeteksi apakah berjalan di dalam lingkungan virtual menggunakan tiga metode:

### 1. `systemd-detect-virt`

```bash
systemd-detect-virt
# Output: none / kvm / qemu / vmware / oracle / xen / ...
```

Jika output bukan `none` dan tidak kosong, dianggap sebagai lingkungan virtual.

### 2. OrbStack Detection

MELISA memeriksa `/etc/os-release` untuk string `orbstack`.

### 3. Indikator File

| File | Lingkungan |
|------|-----------|
| `/proc/vz` | OpenVZ |
| `/.dockerenv` | Docker container |

### Implikasi Deteksi Virtual

Jika berjalan di lingkungan virtual, MELISA menerapkan override konfigurasi `lxc-net.service` untuk memastikan bridge jaringan LXC dapat diinisialisasi dengan benar.

---

## Caching Daftar Distribusi

Daftar distribusi container diunduh dari repositori LXC resmi dan di-cache secara lokal untuk performa yang lebih baik:

| Setting | Nilai |
|---------|-------|
| Path cache | `/tmp/melisa_global_distros.cache` |
| TTL cache | 3600 detik (1 jam) |
| Lock file | `/tmp/melisa_distro.lock` |
| Max retry lock | 40 kali × 500ms |
| Stale lock timeout | 60 detik |

### Proses Caching

1. Cek apakah cache ada dan masih segar (< 1 jam)
2. Jika ya → sajikan dari cache (cepat)
3. Jika tidak → ambil lock file
4. Unduh daftar terbaru dari server LXC
5. Simpan ke cache
6. Lepas lock file

Lock file digunakan untuk mencegah race condition ketika beberapa proses MELISA mencoba memperbarui cache secara bersamaan.

### Paksa Update Cache

Untuk memaksa pembaruan daftar distribusi (hapus cache secara manual):

```bash
rm /tmp/melisa_global_distros.cache
melisa --search
```