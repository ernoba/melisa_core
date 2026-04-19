# Koneksi Pertama dengan Client

Setelah server MELISA berjalan dan client terinstall, langkah terakhir adalah menghubungkan keduanya menggunakan profil koneksi SSH.

---

## Konsep Profil Koneksi

Client MELISA menggunakan konsep **profil** untuk menyimpan informasi koneksi ke berbagai server. Setiap profil memiliki:

- **Nama profil** — pengenal unik untuk profil (e.g., `produksi`, `staging`, `dev`)
- **SSH connection string** — `user@ip_atau_hostname` 
- **MELISA username** — username di sistem MELISA server (bisa berbeda dengan SSH user)

Profil disimpan di file `~/.config/melisa/profiles.conf` dengan format:
```
nama_profil=user@server|melisa_user
```

---

## Menambahkan Profil Server

```bash
melisa auth add <nama_profil> <user@server>
```

**Contoh:**
```bash
# Server produksi
melisa auth add produksi root@192.168.1.100

# Server development dengan user non-root
melisa auth add dev deploy@10.0.0.5

# Server dengan hostname
melisa auth add staging admin@melisa.internal.example.com
```

### Proses Interaktif

Saat menjalankan `auth add`, Anda akan diminta password server satu kali untuk menyalin SSH key:

```
$ melisa auth add produksi root@192.168.1.100

[INFO] Deploying public SSH key to root@192.168.1.100...
[INFO] Please prepare to enter the remote server password.

root@192.168.1.100's password: ●●●●●●●●
Number of key(s) added: 1

[SETUP] Enter your MELISA username on this server (leave blank to use SSH user 'root'):
> adminku
[SUCCESS] Server profile 'produksi' registered. Remote MELISA user: adminku
```

Setelah ini, koneksi berikutnya tidak memerlukan password karena menggunakan autentikasi kunci SSH.

---

## Mengelola Beberapa Profil

### Melihat Semua Profil

```bash
melisa auth list
```

Output:
```
Profil MELISA yang terdaftar:
  * produksi   → root@192.168.1.100  (MELISA user: adminku)  [AKTIF]
    dev        → deploy@10.0.0.5    (MELISA user: deploy)
    staging    → admin@10.0.0.10   (MELISA user: adminku)
```

Profil yang aktif ditandai dengan `*` dan `[AKTIF]`.

### Ganti Profil Aktif

```bash
melisa auth switch dev
```

```
[INFO] Active server profile switched to 'dev'.
```

### Hapus Profil

```bash
melisa auth remove staging
```

```
[WARNING] Are you sure you want to permanently remove the profile 'staging'? (y/N): y
[SUCCESS] Profile 'staging' removed.
```

---

## Validasi Nama Profil dan Username

MELISA menerapkan validasi ketat pada nama profil dan username:

### Aturan Nama Profil
- Panjang: 1–64 karakter
- Karakter yang diizinkan: huruf, angka, `-`, `_`
- Tidak boleh dimulai dengan `-`
- Tidak boleh mengandung karakter khusus, spasi, atau sekuens `..`

### Aturan Username MELISA
- Panjang: 1–32 karakter
- Karakter yang diizinkan: huruf, angka, `-`, `_`
- Tidak boleh dimulai dengan angka atau `-`
- Tidak boleh mengandung sekuens `..` (path traversal)

---

## SSH Multiplexing

Jika SSH di mesin Anda mendukung multiplexing (hampir semua versi modern), MELISA akan mengkonfigurasinya secara otomatis. SSH multiplexing memungkinkan:

- Koneksi SSH pertama dibuat sebagai *master connection*
- Koneksi berikutnya berbagi socket yang sama → jauh lebih cepat
- Tidak perlu autentikasi ulang untuk setiap perintah

Socket multiplexing disimpan di direktori `~/.ssh/melisa-sockets/` (Linux/macOS) atau lokasi yang sesuai di Windows.

---

## Contoh Skenario Penggunaan

### Skenario 1: Satu Server
```bash
melisa auth add server1 admin@192.168.1.10
# Langsung bisa digunakan
melisa exec "melisa --list"
```

### Skenario 2: Tim dengan Beberapa Server
```bash
# Setup semua server
melisa auth add prod    deploy@prod.example.com
melisa auth add staging deploy@staging.example.com
melisa auth add dev     deploy@dev.example.com

# Kerja di dev
melisa auth switch dev
melisa exec "melisa --create myapp ubuntu/jammy/amd64"

# Deploy ke staging
melisa auth switch staging
melisa exec "melisa --up ./myapp/deploy.mel"

# Deploy ke produksi
melisa auth switch prod
melisa exec "melisa --up ./myapp/deploy.mel"
```

### Skenario 3: Anggota Tim Non-Admin
Tim developer yang hanya perlu akses Standard User:
```bash
# Admin melakukan ini di server:
melisa --add developer1

# Developer melakukan ini di mesin lokal:
melisa auth add myserver developer1@192.168.1.100
melisa auth switch myserver
# Sekarang bisa membuat dan mengelola container mereka sendiri
```