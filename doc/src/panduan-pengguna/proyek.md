# Manajemen Proyek

MELISA menyediakan sistem manajemen proyek kolaboratif berbasis **Git bare repository**. Sistem ini memungkinkan tim untuk berbagi kode, mendistribusikan pembaruan, dan mengelola workspace per anggota tim secara terpusat di server.

---

## Konsep Proyek MELISA

### Arsitektur Proyek

Setiap proyek MELISA terdiri dari:

```
/var/melisa/projects/
└── <nama_proyek>/              ← Master repository (Git bare)
    ├── HEAD
    ├── config
    ├── objects/
    └── refs/

/home/<username>/
└── <nama_proyek>/              ← Workspace pengguna (Git clone)
    ├── .git/
    └── ... (file proyek)
```

- **Master Repository** — repository Git bare di `/var/melisa/projects/<nama>/`, dikelola oleh Administrator
- **Workspace Pengguna** — clone dari master di home directory setiap anggota tim

### Alur Kerja Kolaborasi

```
[Admin]                          [Developer]
   │                                 │
   ├─ --new_project myapp            │
   ├─ --invite myapp dev1 dev2       │
   │                                 │
   │                        ← --update myapp (pull dari master)
   │                                 │
   │                        ← Edit file, git commit, git push
   │                                 │
   ├─ --pull dev1 myapp              │
   ├─ --update-all myapp ────────────┤
   │                        ← --update myapp (terima pembaruan)
```

---

## Perintah Proyek

### Membuat Proyek Baru (Admin)

```
melisa --new_project <nama_proyek>
```

```bash
melisa --new_project mywebapp
melisa --new_project backend_api
melisa --new_project data-pipeline
```

> ⚠️ Hanya **Administrator** yang bisa membuat proyek master.

**Validasi nama proyek:**
- Panjang: 1–64 karakter
- Karakter yang diizinkan: huruf, angka, `-`, `_`
- Tidak boleh dimulai dengan `-`
- Tidak boleh mengandung spasi, karakter khusus, atau `..`

Proses pembuatan:
```
--- Initializing New Project: mywebapp ---
[OK] Created master directory: /var/melisa/projects/mywebapp
[OK] Permissions set to 2770 (setgid enabled)
[OK] Git bare repository initialized.
[SUCCESS] Project 'mywebapp' is ready for collaboration!
```

### Lihat Daftar Proyek

```
melisa --projects
```

Output menampilkan proyek yang tersedia bagi pengguna saat ini:
```
Proyek yang tersedia untuk 'developer1':
  NAMA              MASTER PATH                        STATUS
  mywebapp          /var/melisa/projects/mywebapp      Tersedia
  backend_api       /var/melisa/projects/backend_api   Tersedia
```

### Mengundang Pengguna ke Proyek (Admin)

```
melisa --invite <nama_proyek> <user1> [user2 ...]
```

```bash
# Undang satu pengguna
melisa --invite mywebapp developer1

# Undang beberapa pengguna sekaligus
melisa --invite mywebapp developer1 developer2 qa_engineer
```

Proses undangan per pengguna:
1. Membuat direktori workspace di home pengguna
2. Mengatur izin yang sesuai
3. Menyalin konten master ke workspace pengguna
4. Mengkonfigurasi remote Git

```
[INFO] Inviting 'developer1' to project 'mywebapp'...
[OK] Workspace created at /home/developer1/mywebapp
[OK] Git repository initialized.
[OK] Remote 'origin' configured to /var/melisa/projects/mywebapp
[SUCCESS] 'developer1' has been granted access to 'mywebapp'.
```

### Memperbarui Workspace (User)

Pengguna dapat menarik pembaruan terbaru dari master ke workspace mereka:

```
melisa --update <nama_proyek>
```

```bash
melisa --update mywebapp
```

```
[INFO] Pulling latest changes for 'mywebapp'...
[OK] Changes pulled from master.
[SUCCESS] Workspace 'mywebapp' is up to date.
```

### Menarik Workspace Pengguna (Admin)

Administrator dapat menarik/menggabungkan perubahan dari workspace pengguna ke master:

```
melisa --pull <dari_user> <nama_proyek>
```

```bash
melisa --pull developer1 mywebapp
```

```
[INFO] Pulling workspace from 'developer1' for project 'mywebapp'...
[OK] Changes from /home/developer1/mywebapp merged to master.
[SUCCESS] Master repository updated.
```

### Distribusi Pembaruan ke Semua Anggota (Admin)

Setelah master diperbarui, distribusikan ke semua anggota proyek:

```
melisa --update-all <nama_proyek>
```

```bash
melisa --update-all mywebapp
```

```
[INFO] Distributing master updates for 'mywebapp'...
[INFO] Updating workspace for: developer1... [OK]
[INFO] Updating workspace for: developer2... [OK]
[INFO] Updating workspace for: qa_engineer... [OK]
[SUCCESS] All workspaces updated.
```

### Mengeluarkan Pengguna dari Proyek (Admin)

```
melisa --out <nama_proyek> <user1> [user2 ...]
```

```bash
# Keluarkan satu pengguna
melisa --out mywebapp eks_developer

# Keluarkan beberapa pengguna
melisa --out mywebapp eks_dev1 eks_dev2
```

```
[INFO] Revoking 'eks_developer' access to 'mywebapp'...
[OK] Workspace /home/eks_developer/mywebapp removed.
[SUCCESS] 'eks_developer' access to 'mywebapp' has been revoked.
```

### Menghapus Proyek (Admin)

```
melisa --delete_project <nama_proyek>
```

```bash
melisa --delete_project mywebapp
```

> ⚠️ **Perhatian:** Menghapus proyek master akan menghapus repository master. Workspace pengguna yang ada mungkin tidak otomatis terhapus.

---

## Skenario Penggunaan

### Skenario: Setup Proyek Tim Baru

```bash
# 1. Admin membuat proyek
melisa --new_project ecommerce-platform

# 2. Undang semua anggota tim
melisa --invite ecommerce-platform backend1 backend2 frontend1 qa1

# 3. Setiap developer menarik workspace mereka
# (dilakukan oleh masing-masing developer)
melisa --update ecommerce-platform
```

### Skenario: Alur Kerja Harian

```bash
# Developer mulai hari kerja - tarik pembaruan terbaru
melisa --update myproject

# (Developer mengedit file, commit dengan git biasa)
cd ~/myproject
git add .
git commit -m "Tambah fitur login"
git push origin main

# Admin review dan pull ke master
melisa --pull developer1 myproject

# Admin distribusikan ke semua anggota
melisa --update-all myproject
```

### Skenario: Developer Meninggalkan Tim

```bash
# Sebelum hapus pengguna, tarik work terakhir mereka
melisa --pull eks_dev ecommerce-platform

# Keluarkan dari proyek
melisa --out ecommerce-platform eks_dev

# Hapus akun pengguna
melisa --remove eks_dev
```

---

## Izin dan Keamanan Proyek

### Izin Master Repository

Master repository dibuat dengan izin `2770` (setgid bit aktif):
- `7` — pemilik (root) bisa baca, tulis, eksekusi
- `7` — grup bisa baca, tulis, eksekusi
- `0` — pengguna lain tidak punya akses
- `setgid bit` — file baru di dalam direktori mewarisi grup yang sama

### Izin Workspace Pengguna

Workspace pengguna berada di home directory mereka yang dilindungi dengan izin `700` — hanya pemilik yang bisa mengaksesnya.