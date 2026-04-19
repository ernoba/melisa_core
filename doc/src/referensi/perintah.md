# Daftar Perintah Lengkap

Halaman ini adalah referensi lengkap semua perintah yang tersedia di MELISA REPL. Semua perintah diawali dengan `melisa` diikuti subcommand.

---

## Format Umum

```
melisa <subcommand> [argumen] [--audit]
```

- **`--audit`** — Flag opsional yang dapat ditambahkan ke hampir semua perintah. Menampilkan perintah sistem yang dieksekusi di balik layar.
- Argumen dalam `<kurung sudut>` bersifat **wajib**.
- Argumen dalam `[kurung kotak]` bersifat **opsional**.

---

## Perintah Global

| Perintah | Deskripsi |
|----------|-----------|
| `melisa --help` atau `melisa -h` | Tampilkan bantuan dan daftar perintah |
| `melisa --about` | Tampilkan informasi versi dan metadata MELISA |
| `melisa --setup` | Jalankan setup host (install LXC, SSH, konfigurasi jaringan) |
| `melisa --setup --force-unsafe` | Paksa setup meskipun terdeteksi sebagai sesi SSH remote |
| `melisa --clear` | Hapus riwayat REPL (hanya Administrator) |

---

## Perintah Container

### Pencarian dan Informasi

| Perintah | Deskripsi |
|----------|-----------|
| `melisa --search` | Tampilkan semua distribusi LXC yang tersedia |
| `melisa --search <kata_kunci>` | Cari distribusi berdasarkan kata kunci |
| `melisa --list` | Tampilkan semua container beserta status |
| `melisa --active` | Tampilkan hanya container yang sedang berjalan |
| `melisa --info <nama>` | Tampilkan detail lengkap sebuah container |
| `melisa --ip <nama>` | Tampilkan IP address sebuah container |

### Siklus Hidup Container

| Perintah | Deskripsi |
|----------|-----------|
| `melisa --create <nama> <kode_distro>` | Buat container baru dengan distribusi yang ditentukan |
| `melisa --run <nama>` | Jalankan container yang sedang berhenti |
| `melisa --stop <nama>` | Hentikan container yang sedang berjalan |
| `melisa --delete <nama>` | Hapus container secara permanen (memerlukan konfirmasi) |

### Interaksi dengan Container

| Perintah | Deskripsi |
|----------|-----------|
| `melisa --use <nama>` | Masuk ke shell interaktif dalam container |
| `melisa --send <nama> <perintah>` | Kirim dan jalankan perintah di dalam container |
| `melisa --upload <nama> <path_tujuan>` | Upload file dari host ke container |

### Jaringan dan Volume

| Perintah | Deskripsi |
|----------|-----------|
| `melisa --share <nama> <path_host> <path_container>` | Mount folder host ke dalam container |
| `melisa --reshare <nama> <path_host> <path_container>` | Lepas mount folder dari container |

---

## Perintah Pengguna

> Semua perintah berikut memerlukan hak **Administrator**, kecuali `--passwd` yang bisa dijalankan untuk akun sendiri.

| Perintah | Deskripsi |
|----------|-----------|
| `melisa --add <username>` | Buat pengguna MELISA baru (interaktif: pilih peran + set password) |
| `melisa --remove <username>` | Hapus pengguna secara permanen (memerlukan konfirmasi) |
| `melisa --user` | Tampilkan semua pengguna MELISA yang terdaftar |
| `melisa --passwd <username>` | Ubah password pengguna |
| `melisa --upgrade <username>` | Upgrade Standard User menjadi Administrator |
| `melisa --clean` | Hapus file sudoers yatim (tidak memiliki akun pengguna yang sesuai) |

---

## Perintah Proyek

| Perintah | Hak Akses | Deskripsi |
|----------|-----------|-----------|
| `melisa --projects` | Semua | Tampilkan daftar proyek yang bisa diakses |
| `melisa --update <nama_proyek>` | Semua | Tarik pembaruan terbaru dari master ke workspace sendiri |
| `melisa --new_project <nama>` | Admin | Buat proyek master baru (Git bare repo) |
| `melisa --delete_project <nama>` | Admin | Hapus proyek master secara permanen |
| `melisa --invite <nama_proyek> <user1> [user2...]` | Admin | Undang pengguna ke proyek |
| `melisa --out <nama_proyek> <user1> [user2...]` | Admin | Keluarkan pengguna dari proyek |
| `melisa --pull <dari_user> <nama_proyek>` | Admin | Tarik workspace pengguna ke master |
| `melisa --update-all <nama_proyek>` | Admin | Distribusikan pembaruan master ke semua anggota |

---

## Perintah Deployment

| Perintah | Deskripsi |
|----------|-----------|
| `melisa --up <file.mel>` | Deploy aplikasi berdasarkan file manifest `.mel` |
| `melisa --down <file.mel>` | Hentikan deployment yang didefinisikan oleh manifest `.mel` |
| `melisa --mel-info <file.mel>` | Tampilkan ringkasan manifest tanpa melakukan deployment |

---

## Perintah REPL Built-in

Perintah berikut tersedia langsung di REPL tanpa prefix `melisa`:

| Perintah | Deskripsi |
|----------|-----------|
| `exit` atau `quit` | Keluar dari sesi MELISA |
| `cd <path>` | Pindah direktori kerja |
| `cd ~` | Kembali ke home directory |
| `<perintah_bash>` | Jalankan perintah bash biasa (ls, cat, git, dll.) |

> ℹ️ MELISA REPL mendukung eksekusi perintah bash biasa melalui shell. Perintah seperti `ls`, `cat`, `git`, `python3`, `cargo`, dan lainnya dapat dijalankan langsung. PATH secara otomatis mencakup `~/.cargo/bin`.

---

## Referensi Cepat dengan Contoh

### Container — Siklus Lengkap

```bash
# Cari distribusi
melisa --search ubuntu

# Buat container
melisa --create myapp ubuntu/jammy/amd64

# Cek status
melisa --list

# Masuk ke container
melisa --use myapp

# Kirim perintah tanpa masuk
melisa --send myapp apt update && apt upgrade -y

# Cek IP
melisa --ip myapp

# Hentikan
melisa --stop myapp

# Jalankan kembali
melisa --run myapp

# Hapus
melisa --delete myapp
```

### Pengguna — Siklus Lengkap (Admin)

```bash
# Buat pengguna
melisa --add developer1

# Lihat daftar
melisa --user

# Ubah password
melisa --passwd developer1

# Upgrade ke admin
melisa --upgrade developer1

# Hapus pengguna
melisa --remove developer1

# Bersihkan sudoers yatim
melisa --clean
```

### Proyek — Siklus Lengkap

```bash
# (Admin) Buat proyek
melisa --new_project myproject

# (Admin) Undang anggota
melisa --invite myproject dev1 dev2

# (Dev) Tarik workspace
melisa --update myproject

# (Admin) Tarik perubahan dari dev ke master
melisa --pull dev1 myproject

# (Admin) Distribusi ke semua anggota
melisa --update-all myproject

# (Admin) Keluarkan pengguna
melisa --out myproject dev1

# (Admin) Hapus proyek
melisa --delete_project myproject
```

### Deployment — Siklus Lengkap

```bash
# Lihat info manifest
melisa --mel-info ./app/deploy.mel

# Deploy
melisa --up ./app/deploy.mel

# Deploy dengan audit log
melisa --up ./app/deploy.mel --audit

# Hentikan deployment
melisa --down ./app/deploy.mel
```

---

## Tabel Hak Akses Perintah

| Kategori | Perintah | Standard User | Administrator |
|----------|----------|:---:|:---:|
| **Info** | `--help`, `--about`, `--mel-info` | ✅ | ✅ |
| **Container** | `--search`, `--list`, `--active`, `--info`, `--ip` | ✅ | ✅ |
| **Container** | `--create`, `--run`, `--stop`, `--delete` | ✅ | ✅ |
| **Container** | `--use`, `--send`, `--upload` | ✅ | ✅ |
| **Container** | `--share`, `--reshare` | ✅ | ✅ |
| **Deployment** | `--up`, `--down` | ✅ | ✅ |
| **Proyek** | `--projects`, `--update` | ✅ | ✅ |
| **Pengguna** | `--add`, `--remove`, `--user`, `--upgrade`, `--clean` | ❌ | ✅ |
| **Pengguna** | `--passwd` | ✅ (sendiri) | ✅ |
| **Proyek** | `--new_project`, `--delete_project` | ❌ | ✅ |
| **Proyek** | `--invite`, `--out`, `--pull`, `--update-all` | ❌ | ✅ |
| **Sistem** | `--setup`, `--clear` | ❌ | ✅ |

---

## Kode Exit dan Error

MELISA mengembalikan pesan error yang deskriptif ke terminal. Format umum pesan:

```
[ERROR] Pesan error yang menjelaskan masalah
[WARNING] Peringatan yang tidak menghentikan eksekusi
[INFO] Informasi status proses
[SUCCESS] Konfirmasi operasi berhasil
[BLOCKED] Input ditolak oleh Input Guard
[AUDIT] Log perintah sistem (hanya di mode --audit)
```

### Error Umum dan Solusinya

| Error | Penyebab | Solusi |
|-------|----------|--------|
| `Container 'X' does not exist` | Nama container salah | Gunakan `melisa --list` untuk cek nama |
| `Container 'X' is not running` | Container belum distart | Jalankan `melisa --run X` terlebih dahulu |
| `Only administrators can...` | Tidak punya hak admin | Minta admin untuk menjalankan perintah |
| `[BLOCKED] Shell injection detected` | Input mengandung karakter berbahaya | Periksa karakter dalam perintah |
| `[BLOCKED] Path traversal detected` | Path mengandung `../` | Gunakan path absolut |
| `File 'X' not found` | Path file `.mel` salah | Verifikasi path ke file manifest |
| `lxcbr0 not found` | Bridge jaringan LXC tidak aktif | Jalankan `sudo systemctl restart lxc-net` |