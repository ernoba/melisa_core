# Manajemen Pengguna

MELISA mengimplementasikan sistem manajemen pengguna Linux yang terintegrasi dengan mekanisme kontrol akses berbasis peran. Setiap pengguna MELISA adalah pengguna sistem Linux yang sebenarnya, dengan shell login diset ke binary MELISA.

---

## Konsep Pengguna MELISA

### Bagaimana Pengguna MELISA Bekerja

Ketika pengguna MELISA dibuat, sistem melakukan hal berikut:

1. **Buat akun sistem Linux** — `useradd -m -s /usr/local/bin/melisa <username>`
2. **Set izin home directory** — `chmod 700 /home/<username>` (hanya pemilik yang bisa akses)
3. **Set password** — interaktif via `passwd`
4. **Konfigurasi sudoers** — file `/etc/sudoers.d/melisa-<username>` dibuat
5. **Tambah ke grup admin** (jika Administrator) — tambah ke grup `sudo` atau `wheel`

Karena shell pengguna diset ke `/usr/local/bin/melisa`, setiap kali pengguna login via SSH, mereka langsung masuk ke MELISA REPL — bukan bash shell biasa.

### Peran Pengguna

| Peran | Kode | Deskripsi |
|-------|------|-----------|
| **Administrator** | `Admin` | Akses penuh ke semua fitur MELISA |
| **Standard User** | `Regular` | Akses ke manajemen container dan proyek personal |

---

## Perintah Manajemen Pengguna

> ⚠️ Semua perintah manajemen pengguna memerlukan hak **Administrator**.

### Tambah Pengguna Baru

```
melisa --add <username>
```

```bash
melisa --add developer1
melisa --add john_doe
melisa --add qa-engineer
```

Proses interaktif:
```
--- Provisioning New MELISA User: developer1 ---
Select Access Level for developer1:
  1) Administrator (Full Management: Users, Projects & LXC)
  2) Standard User (Project & LXC Management Only)
Enter choice (1/2): 2

[SUCCESS] User account 'developer1' successfully created.
[INFO] Please set a password for 'developer1':
New password: ●●●●●●●●
Retype new password: ●●●●●●●●
[SUCCESS] Password configured successfully.
[INFO] Standard User access granted.
```

### Validasi Username

MELISA menerapkan aturan validasi ketat:
- Panjang: 1–32 karakter
- Karakter yang diizinkan: huruf (`a-z`, `A-Z`), angka (`0-9`), tanda hubung (`-`), dan garis bawah (`_`)
- Tidak boleh dimulai dengan angka atau tanda hubung
- Tidak boleh mengandung sekuens `..` (path traversal)

✅ Valid: `developer1`, `john_doe`, `qa-engineer`, `admin2`
❌ Tidak valid: `1developer`, `-user`, `user name`, `../../etc/passwd`

### Lihat Daftar Pengguna

```
melisa --user
```

Output:
```
Pengguna MELISA yang terdaftar:
  USERNAME        ROLE           HOME
  root            (system)       /root
  adminku         Administrator  /home/adminku
  developer1      Standard User  /home/developer1
  john_doe        Standard User  /home/john_doe
```

### Ubah Password Pengguna

```
melisa --passwd <username>
```

```bash
melisa --passwd developer1
```

```
[INFO] Changing password for 'developer1'...
New password: ●●●●●●●●
Retype new password: ●●●●●●●●
[SUCCESS] Password changed successfully.
```

### Upgrade Peran Pengguna

Upgrade Standard User menjadi Administrator:

```
melisa --upgrade <username>
```

```bash
melisa --upgrade developer1
```

```
[INFO] Upgrading 'developer1' to Administrator role...
[OK] Added to sudo group.
[OK] Sudoers file updated.
[SUCCESS] 'developer1' is now an Administrator.
```

> 📝 **Catatan:** Saat ini tidak ada perintah downgrade (Administrator ke Standard User). Jika diperlukan, hapus pengguna dan buat ulang dengan peran yang diinginkan.

### Hapus Pengguna

```
melisa --remove <username>
```

```bash
melisa --remove developer1
```

```
[WARNING] Are you sure you want to permanently delete user 'developer1'? (y/N): y
[INFO] Removing user account 'developer1'...
[OK] Home directory /home/developer1 removed.
[OK] Sudoers file removed.
[SUCCESS] User 'developer1' has been permanently deleted.
```

> ⚠️ **Perhatian:** Penghapusan pengguna bersifat permanen. Home directory dan semua data pengguna akan dihapus.

---

## Konfigurasi Sudoers

MELISA menggunakan file sudoers terpisah per pengguna di direktori `/etc/sudoers.d/`. Format file: `melisa-<username>`.

### Sudoers untuk Standard User

```sudoers
# MELISA Sudoers - developer1 (Standard User)
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-create
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-start
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-stop
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-destroy
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-attach
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-info
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-ls
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-copy
```

### Sudoers untuk Administrator

Administrator mendapatkan izin penuh tambahan untuk manajemen sistem, jaringan, dan pengguna lain.

### Membersihkan Sudoers Yatim

Jika ada file sudoers yang tidak sesuai dengan pengguna yang ada:

```
melisa --clean
```

```
[INFO] Scanning for orphaned sudoers files...
[OK] Found 2 orphaned files.
[OK] Removed: /etc/sudoers.d/melisa-olduser
[OK] Removed: /etc/sudoers.d/melisa-testuser
[INFO] Sudoers cleanup complete.
```

---

## Skenario Penggunaan

### Skenario: Setup Tim Pengembang

```bash
# Buat pengguna untuk tim
melisa --add senior_dev    # pilih 1 (Administrator)
melisa --add backend_dev   # pilih 2 (Standard User)
melisa --add frontend_dev  # pilih 2 (Standard User)
melisa --add qa_engineer   # pilih 2 (Standard User)

# Senior dev dibuat admin
# Backend dev hanya bisa kelola container sendiri
```

### Skenario: Promosi Pengguna

```bash
# Backend dev dipromosikan jadi admin
melisa --upgrade backend_dev
```

### Skenario: Pengguna Meninggalkan Tim

```bash
# Hapus akses
melisa --remove eks_developer

# Bersihkan sudoers yatim (jika ada)
melisa --clean
```

---

## Keamanan Pengguna

### Isolasi Home Directory

Setiap pengguna memiliki home directory dengan izin `700` — tidak ada pengguna lain (kecuali root) yang bisa membaca konten home directory pengguna lain.

### Shell Terkunci

Shell pengguna dikunci ke `/usr/local/bin/melisa`. Pengguna tidak bisa menjalankan bash, sh, atau shell lainnya secara langsung melalui SSH — mereka hanya bisa berinteraksi melalui MELISA REPL.

### Kontrol Perintah via Input Guard

Semua input di MELISA REPL difilter oleh Input Guard sebelum dieksekusi. Ini mencegah pengguna dari:
- Shell injection (`$(`, `` ` ``, `${`, `&&`, `||`, `;`)
- Path traversal (`../`, `..\`, `..%2f`)
- Null-byte injection (`\0`)

Lihat [Keamanan & Input Guard](../arsitektur/keamanan.md) untuk detail teknis.