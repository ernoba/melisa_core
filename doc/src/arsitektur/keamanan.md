# Keamanan & Input Guard

MELISA dirancang dengan prinsip *security-first*. Halaman ini menjelaskan semua mekanisme keamanan yang diimplementasikan, dengan fokus utama pada **Input Guard** — sistem validasi input berlapis yang melindungi REPL dari berbagai jenis serangan.

---

## Lapisan Keamanan MELISA

```
┌─────────────────────────────────────────────┐
│  Layer 1: Root Check                        │
│  Wajib root. Eskalasi otomatis via sudo     │
│  dengan whitelist variabel lingkungan       │
├─────────────────────────────────────────────┤
│  Layer 2: Input Guard                       │
│  Validasi setiap input sebelum eksekusi     │
│  (length, null-byte, injection, traversal)  │
├─────────────────────────────────────────────┤
│  Layer 3: Sudoers per-Pengguna              │
│  Hak sudo granular — hanya perintah LXC    │
│  yang diizinkan per pengguna                │
├─────────────────────────────────────────────┤
│  Layer 4: Shell Terkunci                    │
│  Login shell = /usr/local/bin/melisa        │
│  Pengguna tidak bisa akses bash langsung    │
├─────────────────────────────────────────────┤
│  Layer 5: Isolasi Home Directory            │
│  chmod 700 — hanya pemilik yang bisa akses  │
├─────────────────────────────────────────────┤
│  Layer 6: Isolasi Container LXC            │
│  Namespace, cgroup, jaringan terisolasi     │
└─────────────────────────────────────────────┘
```

---

## Layer 1: Root Check & Privilege Escalation

MELISA **wajib berjalan sebagai root** (`UID 0`). Ini diverifikasi di `main.rs` sebelum melakukan apapun:

```rust
fn is_running_as_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}
```

Jika tidak berjalan sebagai root, MELISA melakukan **re-eksekusi diri sendiri** dengan `sudo` menggunakan `process::Command::exec()` (Unix `execve` — menggantikan proses saat ini, bukan fork):

```rust
fn re_exec_as_root() {
    let exe_path = env::current_exe()?;
    let canonical_binary = fs::canonicalize(&exe_path)?;  // hindari symlink attack
    
    let mut sudo_cmd = process::Command::new("sudo");
    
    // Whitelist variabel lingkungan yang diizinkan
    for &var in ALLOWED_ENV_VARS {
        if env::var(var).is_ok() {
            sudo_cmd.arg(format!("--preserve-env={}", var));
        }
    }
    
    sudo_cmd.arg("--").arg(&canonical_binary).args(&args);
    sudo_cmd.exec();
}
```

### Whitelist Variabel Lingkungan

Hanya variabel berikut yang diizinkan melewati batas privilege:

```rust
const ALLOWED_ENV_VARS: &[&str] = &[
    "TERM",           // Tipe terminal (untuk warna)
    "LANG",           // Locale bahasa
    "LC_ALL",         // Override locale
    "LC_MESSAGES",    // Locale pesan
    "MELISA_DEBUG",   // Mode debug MELISA
];
```

Semua variabel lingkungan lain (termasuk `PATH`, `HOME`, `LD_PRELOAD`, dll.) **tidak** diwariskan ke proses root, mencegah serangan privilege escalation melalui environment manipulation.

### Kanonisasi Path Binary

```rust
let canonical_binary = fs::canonicalize(&exe_path)?;
```

Kanonisasi memastikan bahwa path yang dieksekusi adalah path absolut nyata — bukan symlink yang bisa dimanipulasi oleh attacker untuk menjalankan binary yang berbeda.

---

## Layer 2: Input Guard

Input Guard adalah sistem validasi yang memeriksa **setiap input pengguna** di REPL sebelum dieksekusi. Implementasi ada di `src/core/guard/filter.rs`.

### Posisi di REPL Loop

```rust
// src/cli/melisa_cli.rs — di dalam REPL loop
Ok(line) => {
    let input = line.trim();
    if input.is_empty() { continue; }
    
    // ── Input Security Gate ──────────────────────────────
    if let FilterResult::Block(reason) = filter_input(input) {
        eprintln!("{}[BLOCKED]{} {}", RED, RESET, reason);
        let _ = rl.add_history_entry(input);
        let _ = rl.append_history(&history_path);
        continue;  // TIDAK dieksekusi
    }
    // ────────────────────────────────────────────────────
    
    match execute_command(input, &p_info.user, &p_info.home).await {
        // ...
    }
}
```

Penting: Input yang diblokir **tetap dicatat ke riwayat** untuk keperluan audit, tetapi **tidak dieksekusi**.

### Empat Aturan Validasi (Urutan Prioritas)

#### Aturan 1: Length Guard

```
Jika len(input) > 1024 → BLOCK
```

Membatasi panjang input maksimum 1024 karakter. Ini mencegah:
- Buffer overflow pada alokasi string
- Denial of service via input sangat panjang
- Penyisipan payload tersembunyi dalam string panjang

**Pesan error:**
```
[BLOCKED] Input terlalu panjang (>1024 karakter).
```

#### Aturan 2: Null-Byte Guard

```
Jika input mengandung '\0' → BLOCK
```

Null byte tidak pernah valid dalam perintah MELISA. Serangan null-byte injection memanfaatkan fakta bahwa beberapa fungsi C memotong string pada `\0`, sehingga validasi di level tinggi bisa dibypass.

**Pesan error:**
```
[BLOCKED] Input mengandung null byte yang tidak diizinkan.
```

#### Aturan 3: Shell Injection Guard

```
Karakter yang diblokir: $( ` ${ && || ; \n \r > < |
Kecuali pada konteks: --send dan cd
```

Karakter-karakter ini adalah operator shell yang bisa digunakan untuk menyisipkan perintah tambahan:

| Karakter/Sekuens | Serangan yang Dicegah |
|------------------|----------------------|
| `$(...)` | Command substitution |
| `` `...` `` | Command substitution (legacy) |
| `${...}` | Variable expansion/injection |
| `&&` | Chaining perintah kondisional |
| `\|\|` | Chaining perintah kondisional |
| `;` | Pemisah perintah |
| `\n`, `\r` | Newline injection |
| `>`, `<` | Redirection I/O |
| `\|` | Piping ke perintah lain |

**Pengecualian Konteks:**

`--send <nama> <perintah>` — Perintah diteruskan verbatim ke container via LXC. Operator shell seperti `apt update && apt upgrade -y` sah digunakan di sini karena:
- LXC menerima argumen sebagai **array** (bukan string tunggal)
- Parsing dilakukan oleh shell **di dalam container**, bukan di host
- Layer isolasi LXC mencegah eksekusi di host

`cd <path>` — Shell injection tidak relevan karena `cd` hanya mengubah direktori kerja, bukan menjalankan perintah. Namun path traversal tetap diblokir.

**Pesan error:**
```
[BLOCKED] Shell injection terdeteksi: karakter '&&' tidak diizinkan dalam konteks ini.
```

#### Aturan 4: Path Traversal Guard

```
Selalu aktif — blokir: ../ ..\ ..%2f ..%5c
```

Path traversal memungkinkan akses ke direktori di luar yang diizinkan. MELISA memblokir semua bentuk path traversal, termasuk versi URL-encoded:

| Sekuens | Keterangan |
|---------|-----------|
| `../` | Path traversal Unix standar |
| `..\` | Path traversal Windows |
| `..%2f` | URL-encoded `/` |
| `..%5c` | URL-encoded `\` |

Aturan ini **selalu aktif** — tidak ada pengecualian konteks untuk path traversal.

**Pesan error:**
```
[BLOCKED] Path traversal terdeteksi: sekuens '..' tidak diizinkan.
```

---

## Layer 3: Sudoers Per-Pengguna

Setiap pengguna MELISA memiliki file sudoers terpisah di `/etc/sudoers.d/melisa-<username>` yang mendefinisikan perintah spesifik yang boleh dijalankan dengan sudo tanpa password.

### Pendekatan Whitelist

MELISA menggunakan pendekatan **whitelist** — hanya perintah yang secara eksplisit dicantumkan yang boleh dijalankan, bukan akses penuh ke sudo.

```sudoers
# /etc/sudoers.d/melisa-developer1
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-create
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-start
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-start -n *
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-stop
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-destroy
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-attach
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-info
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-ls
developer1 ALL=(root) NOPASSWD: /usr/bin/lxc-copy
# ... dan perintah lain yang diperlukan
```

### Validasi Sudoers

Sebelum file sudoers ditulis, MELISA memvalidasi sintaks menggunakan `visudo -c` untuk mencegah file sudoers yang rusak yang bisa mengunci akses sistem.

### Pembersihan Sudoers Yatim

Perintah `melisa --clean` memindai `/etc/sudoers.d/` dan menghapus file yang tidak memiliki pengguna sistem yang sesuai:

```rust
pub async fn clean_orphaned_sudoers() {
    // Scan /etc/sudoers.d/ untuk file dengan prefix "melisa-"
    // Cek apakah username ada di sistem
    // Hapus file yang tidak memiliki pengguna yang cocok
}
```

---

## Layer 4: Shell Terkunci

```bash
# Saat pengguna MELISA dibuat:
useradd -m -s /usr/local/bin/melisa <username>
```

Shell login pengguna dikunci ke binary MELISA. Ini berarti:

1. Login SSH langsung masuk ke MELISA REPL
2. Tidak bisa escape ke bash dengan `bash`, `sh`, atau shell lain
3. Tidak bisa mengeksekusi arbitrary code di luar MELISA REPL

Registrasi ke `/etc/shells` diperlukan agar shell dianggap valid oleh sistem:
```
/usr/local/bin/melisa
```

---

## Layer 5: Isolasi Home Directory

```bash
# Saat pengguna dibuat:
chmod 700 /home/<username>
```

Izin `700` berarti:
- Pemilik: baca, tulis, eksekusi ✅
- Grup: tidak ada akses ❌
- Lainnya: tidak ada akses ❌

Pengguna lain (kecuali root) tidak bisa membaca, menulis, atau melihat isi home directory pengguna lain.

---

## Layer 6: Isolasi Container LXC

Container LXC menggunakan fitur kernel Linux untuk isolasi penuh:

| Mekanisme | Fungsi |
|-----------|--------|
| **PID namespace** | Proses di container tidak bisa melihat/kill proses host |
| **Mount namespace** | Filesystem container terpisah dari host |
| **Network namespace** | Interface jaringan terisolasi (hanya lxcbr0) |
| **User namespace** | UID mapping antara container dan host |
| **cgroup** | Pembatasan resource (CPU, RAM, I/O) |

---

## Validasi Username & Nama Proyek

Semua identifier (username, nama proyek, nama profil) divalidasi untuk mencegah injeksi path dan karakter berbahaya:

```rust
// Validasi username
fn validate_server_username(username: &str) -> Result<(), String> {
    if username.is_empty() || username.len() > 32 { return Err(...); }
    if username.starts_with(|c: char| c.is_ascii_digit() || c == '-') { return Err(...); }
    if !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(...);
    }
    if username.contains("..") { return Err(...); }  // path traversal
    Ok(())
}
```

---

## Validasi Konfigurasi SSH Client

Di sisi client, nilai konfigurasi SSH (host, port, username) divalidasi sebelum ditulis ke file konfigurasi:

```rust
fn validate_ssh_config_value(value: &str, field: &str) -> io::Result<()> {
    let forbidden: &[char] = &['\n', '\r', '\0', '#'];
    for ch in forbidden {
        if value.contains(*ch) {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("SSH config {field} mengandung karakter terlarang: {ch:?}"),
            ));
        }
    }
    Ok(())
}
```

Karakter `\n`, `\r`, `\0`, dan `#` dilarang karena bisa digunakan untuk menyisipkan direktif SSH yang tidak diinginkan ke file `~/.ssh/config`.

---

## Pertimbangan Keamanan Tambahan

### Atomic File Write

File konfigurasi ditulis menggunakan pola atomic:
```rust
let tmp = path.with_extension("tmp");
fs::write(&tmp, content)?;
fs::rename(&tmp, &path)?;  // atomic di Unix
```

Ini mencegah file yang rusak/parsial jika proses terhenti saat penulisan.

### Deteksi Sesi Remote Saat Setup

MELISA mendeteksi dan memblokir setup dari sesi SSH remote secara default karena konfigurasi firewall bisa menyebabkan lockout:

```rust
pub async fn install_host_environment() {
    if is_risky_remote_session().await {
        // Blokir kecuali ada --force-unsafe
    }
}
```

Pengguna harus secara eksplisit menambahkan `--force-unsafe` untuk bypass proteksi ini.