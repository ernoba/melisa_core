# Modul Inti MELISA

Halaman ini menjelaskan setiap modul dalam kodebase MELISA secara teknis — tanggung jawab, interface publik, dan detail implementasi penting.

---

## `src/cli` — Lapisan Antarmuka

### `melisa_cli.rs` — REPL Utama

Modul ini adalah jantung MELISA: mengimplementasikan REPL (*Read-Eval-Print Loop*) menggunakan library `rustyline`.

**Tanggung jawab:**
- Inisialisasi editor rustyline dengan konfigurasi helper
- Menampilkan banner sistem (`display_melisa_banner`)
- Mengelola riwayat perintah (simpan ke `history.txt`, muat saat startup)
- Loop utama: baca input → validasi Input Guard → eksekusi → cetak hasil
- Menangani sinyal `Ctrl+C` (batalkan input) dan `Ctrl+D` (keluar)
- Menangani `ExecResult::ResetHistory` untuk menghapus riwayat

**Alur loop utama:**
```rust
loop {
    match rl.readline(&prompt) {
        Ok(line) => {
            let input = line.trim();
            if input.is_empty() { continue; }
            
            // Input Guard
            if let FilterResult::Block(reason) = filter_input(input) {
                eprintln!("[BLOCKED] {}", reason);
                continue;
            }
            
            // Catat ke riwayat
            rl.add_history_entry(input);
            rl.append_history(&history_path);
            
            // Eksekusi
            match execute_command(input, &user, &home).await {
                ExecResult::Break         => break,
                ExecResult::ResetHistory  => { reset_history(); },
                ExecResult::Continue      => {},
                ExecResult::Error(e)      => eprintln!("{}", e),
            }
        }
        Err(ReadlineError::Interrupted) => { /* Ctrl+C */ continue; }
        Err(ReadlineError::Eof)         => { /* Ctrl+D */ break; }
        Err(e) => { eprintln!("Error: {}", e); break; }
    }
}
```

### `executor.rs` — Parser & Dispatcher

Memproses input pengguna menjadi aksi yang dapat dieksekusi.

**Fungsi utama:**
- `parse_command(input)` → `(Vec<String>, bool)` — Tokenisasi input, deteksi flag `--audit`
- `execute_command(input, user, home)` → `ExecResult` — Dispatcher level pertama
- `dispatch_melisa_subcommand(tokens, audit, user, home)` → `ExecResult` — Dispatcher subcommand `melisa`

**Enum `ExecResult`:**
```rust
pub enum ExecResult {
    Continue,         // Lanjut ke iterasi REPL berikutnya
    Break,            // Keluar dari REPL (exit/quit)
    ResetHistory,     // Hapus riwayat perintah
    Error(String),    // Tampilkan pesan error
}
```

**Fallthrough ke bash:**
Perintah yang tidak dikenali oleh MELISA dieksekusi melalui bash dengan PATH yang diperkaya `~/.cargo/bin`:
```rust
_ => {
    tokio::process::Command::new("bash")
        .env("PATH", format!("{}:{}", cargo_bin, path_env))
        .env("HOME", home)
        .env("USER", user)
        .args(["-c", input])
        .status()
        .await
}
```

### `helper.rs` — Autocomplete & Hints

Mengimplementasikan `MelisaHelper` yang menggabungkan:
- `FilenameCompleter` — tab completion untuk path file
- `HistoryHinter` — saran berdasarkan riwayat perintah
- `MatchingBracketHighlighter` — highlight bracket yang cocok
- `MatchingBracketValidator` — validasi bracket tertutup

### `wellcome.rs` — Banner & Dashboard

Menampilkan animasi boot saat MELISA pertama kali dijalankan:

1. **System Boot Sequence** — menampilkan log sistem palsu dengan delay (efek boot)
2. **Core Decryption Animation** — animasi glitch karakter yang terurai menjadi teks `M.E.L.I.S.A // SYSTEM_STABLE_ENVIRONMENT`
3. **System Dashboard** — menampilkan info sistem nyata (OS, hostname, CPU, RAM) menggunakan library `sysinfo`
4. **Security Enforcement** — display direktif isolasi keamanan

Menggunakan `rand` untuk variasi animasi dan `chrono` untuk timestamp.

---

## `src/core/container` — Manajemen Container

### `lifecycle.rs` — Siklus Hidup Container

Implementasi operasi dasar container.

**Fungsi utama:**

| Fungsi | Deskripsi |
|--------|-----------|
| `create_container(name, meta, pb, audit)` | Buat container baru via `lxc-create` |
| `delete_container(name, pb, audit)` | Hapus container via `lxc-destroy` |
| `start_container(name, audit)` | Jalankan container via `lxc-start` |
| `stop_container(name, audit)` | Hentikan container via `lxc-stop` |
| `attach_to_container(name)` | Masuk ke shell container via `lxc-attach` |

**Alur `create_container`:**
1. `verify_host_runtime(audit)` — Pastikan `lxcbr0` ada
2. `lxc-create -n <name> -t download -- -d <distro> -r <release> -a <arch>`
3. `inject_network_config(name)` — Tulis konfigurasi jaringan ke file config LXC
4. `setup_container_dns(name)` — Konfigurasi DNS di dalam container
5. `lxc-start -n <name>`
6. `wait_for_network_initialization(name)` — Poll hingga container mendapat IP (maks 30 detik)
7. Install/update system packages (apt/dnf/apk/pacman/zypper)
8. `write_container_metadata(name, meta)`

**Fungsi helper `run_sudo`:**
```rust
async fn run_sudo(args: &[&str], is_audit: bool) -> io::Result<ExitStatus> {
    let mut cmd = Command::new("sudo");
    cmd.arg("-n");  // non-interactive (tidak minta password)
    cmd.args(args);
    // ...
}
```

Semua operasi LXC menggunakan `sudo -n` untuk memastikan tidak ada prompt password interaktif yang memblokir eksekusi async.

### `network.rs` — Konfigurasi Jaringan

Mengelola infrastruktur jaringan container.

**Fungsi utama:**

| Fungsi | Deskripsi |
|--------|-----------|
| `ensure_host_network_ready(audit)` | Setup bridge `lxcbr0` dan konfigurasi dnsmasq |
| `inject_network_config(name)` | Tulis konfigurasi jaringan ke `/var/lib/lxc/<name>/config` |
| `setup_container_dns(name)` | Konfigurasi `/etc/resolv.conf` di dalam container |
| `unlock_container_dns(name)` | Hapus immutable flag dari `resolv.conf` jika ada |
| `ensure_nat_routing_ready()` | Aktifkan IP forwarding dan NAT iptables |
| `add_shared_folder(name, host, container)` | Tambah bind mount ke config container |
| `remove_shared_folder(name, host, container)` | Hapus bind mount dari config container |
| `is_virtualised_environment()` | Deteksi lingkungan virtual |

**Konfigurasi jaringan yang diinjeksikan:**
```
# Dalam /var/lib/lxc/<name>/config
lxc.net.0.type = veth
lxc.net.0.link = lxcbr0
lxc.net.0.flags = up
lxc.net.0.hwaddr = 02:xx:xx:xx:xx:xx  # MAC acak
```

### `query.rs` — Query & Perintah

Membaca status dan mengirim perintah ke container.

| Fungsi | Implementasi |
|--------|-------------|
| `list_containers(active_only)` | Parse output `lxc-ls --fancy` |
| `get_container_ip(name)` | Parse `lxc-info -n <name> -iH` |
| `is_container_running(name)` | Parse `lxc-info -n <name>` |
| `container_exists(name)` | Cek direktori `/var/lib/lxc/<name>` |
| `send_command(name, args)` | `lxc-attach -n <name> -- <args>` |
| `upload_to_container(name, dest)` | `lxc-attach` + file transfer |

### `types.rs` — Tipe Data

```rust
pub const LXC_BASE_PATH: &str = "/var/lib/lxc";
pub const LXC_PATH: &str = "/var/lib/lxc";

pub struct DistroMetadata {
    pub distro: String,   // "ubuntu"
    pub release: String,  // "jammy"
    pub arch: String,     // "amd64"
}

pub enum ContainerStatus {
    Running,
    Stopped,
    Frozen,
    Unknown,
}
```

---

## `src/core/guard` — Input Guard

### `filter.rs` — Logika Validasi

```rust
pub enum FilterResult {
    Allow,
    Block(String),  // String = pesan error human-readable
}

pub fn filter_input(raw_input: &str) -> FilterResult {
    // Aturan 1: Length
    if raw_input.len() > 1024 {
        return FilterResult::Block("Input terlalu panjang".into());
    }
    
    // Aturan 2: Null byte
    if raw_input.contains('\0') {
        return FilterResult::Block("Null byte terdeteksi".into());
    }
    
    // Tentukan konteks
    let is_send_context = raw_input.starts_with("melisa --send ");
    let is_cd_context   = raw_input.starts_with("cd ");
    
    // Aturan 3: Shell injection (kecuali --send)
    if !is_send_context {
        for pattern in SHELL_INJECTION_PATTERNS {
            if raw_input.contains(pattern) {
                return FilterResult::Block(format!("Shell injection: '{}'", pattern));
            }
        }
    }
    
    // Aturan 4: Path traversal (selalu)
    for pattern in PATH_TRAVERSAL_PATTERNS {
        if raw_input.to_lowercase().contains(pattern) {
            return FilterResult::Block("Path traversal terdeteksi".into());
        }
    }
    
    FilterResult::Allow
}
```

Modul ini mencakup **30+ unit test** yang memvalidasi berbagai skenario serangan.

---

## `src/core/user` — Manajemen Pengguna

### `management.rs`

Implementasi operasi CRUD pengguna sistem Linux terintegrasi dengan MELISA.

**`add_melisa_user(username, audit)`:**
1. Validasi `validate_server_username(username)`
2. Tanya peran (Admin/Regular) secara interaktif
3. `useradd -m -s /usr/local/bin/melisa <username>`
4. `chmod 700 /home/<username>`
5. `set_user_password(username)` — panggil `passwd <username>`
6. `configure_sudoers(username, role, audit)`
7. Jika Admin: tambah ke grup `sudo` atau `wheel`

**`delete_melisa_user(username, audit)`:**
1. Cek keberadaan pengguna
2. `userdel -r <username>` (hapus home directory)
3. `remove_orphaned_sudoers_files(username)`

### `sudoers.rs`

Mengelola file sudoers per-pengguna di `/etc/sudoers.d/`.

```rust
pub const SUDOERS_DIR: &str = "/etc/sudoers.d";
pub const SUDOERS_FILE_PREFIX: &str = "melisa-";

pub fn sudoers_file_path(username: &str) -> PathBuf {
    PathBuf::from(SUDOERS_DIR)
        .join(format!("{}{}", SUDOERS_FILE_PREFIX, username))
}
```

### `types.rs`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum UserRole {
    Admin,
    Regular,
}
```

---

## `src/core/project` — Manajemen Proyek

### `management.rs`

```rust
pub const PROJECTS_MASTER_PATH: &str = "/var/melisa/projects";
```

**Operasi utama:**

| Fungsi | Implementasi |
|--------|-------------|
| `create_new_project(name, audit)` | `mkdir -p` + `chmod 2770` + `git init --bare` |
| `delete_project(path, name)` | `rm -rf` dengan konfirmasi |
| `invite_users_to_project(project, users, audit)` | Clone bare repo ke home user + setup remote |
| `remove_users_from_project(project, users, audit)` | Hapus workspace user |
| `pull_user_workspace(user, project, audit)` | `git pull` dari workspace user ke master |
| `update_project_for_user(project, user, audit)` | `git pull` dari master ke workspace user |
| `distribute_master_to_all_members(project, audit)` | Loop `update_project_for_user` untuk semua anggota |
| `list_projects(home)` | Scan `/var/melisa/projects/` + home user |

---

## `src/deployment` — Engine Deployment

### `manifest/parser.rs`

**`load_mel_file(path) -> Result<MelManifest, MelParseError>`:**
1. Cek keberadaan file
2. Baca konten sebagai UTF-8 string
3. Parse TOML menggunakan `toml::from_str`
4. Validasi dengan `validate_manifest`

**`MelParseError`:**
```rust
pub enum MelParseError {
    NotFound(String),          // File tidak ditemukan
    TomlParse(toml::de::Error), // Sintaks TOML tidak valid
    Io(std::io::Error),        // Error I/O
    Invalid(String),           // Validasi semantik gagal
}
```

### `deployer.rs`

**`cmd_up(mel_path, audit)`** — Orkestrator deployment 7 tahap:
1. Baca dan validasi manifest
2. Buat container jika belum ada
3. Start container
4. Install system dependencies
5. Install language dependencies (pip, npm, cargo, dll.)
6. Konfigurasi port forwarding dan volume
7. Jalankan lifecycle hooks + health check

**`cmd_down(mel_path, audit)`:**
1. Baca manifest
2. Jalankan lifecycle hook `stop`
3. Stop container

**`HealthCheckPlan`:**
```rust
pub struct HealthCheckPlan {
    pub command:       String,
    pub retries:       u32,
    pub interval_secs: u64,
    pub timeout_secs:  u64,
}
```

### `dependency.rs`

Mendeteksi package manager di dalam container dan menginstall dependensi:

```rust
pub async fn detect_package_manager(container: &str) -> &'static str {
    // Cek keberadaan: apt, dnf, pacman, apk, zypper
    // Return package manager yang ditemukan pertama
}

pub async fn install_system_deps(container: &str, deps: &[String], audit: bool);
pub async fn install_lang_deps(container: &str, manifest: &MelManifest, audit: bool);
```

---

## `src/distros` — Deteksi Distribusi

### `host_distro.rs`

```rust
pub enum HostDistro {
    Debian, Fedora, Arch, Alpine, Suse, OrbStack, Unknown
}

pub struct DistroConfig {
    pub name: String,
    pub pkg_manager: String,
    pub lxc_packages: Vec<String>,
    pub firewall_tool: FirewallKind,
}

pub enum FirewallKind {
    Firewalld, Ufw, Iptables
}
```

Deteksi dilakukan dengan membaca `/etc/os-release` dan mengklasifikasikan nilai `ID=` atau `ID_LIKE=`.

### `lxc_distro.rs`

Mengunduh dan men-cache daftar distribusi container dari repositori LXC resmi.

**Konstanta:**
```rust
const DISTRO_CACHE_PATH: &str = "/tmp/melisa_global_distros.cache";
const LOCK_FILE_PATH:    &str = "/tmp/melisa_distro.lock";
const CACHE_TTL_SECS:    u64  = 3600;
const LOCK_STALE_SECS:   u64  = 60;
const MAX_LOCK_RETRIES:  u32  = 40;
const LOCK_RETRY_DELAY_MS: u64 = 500;
```

Menggunakan file lock (`O_CREAT | O_EXCL`) untuk mencegah race condition saat update cache.

---

## `src/core/setup.rs` — Setup Host

Mengkonfigurasi server Linux sebagai host MELISA secara otomatis. Dipanggil via `melisa --setup`.

**8 Tahap Setup:**

1. `install_lxc_packages(pkg_manager, packages)` — Install LXC dan tool pendukung
2. `install_ssh_server(pkg_manager)` — Install dan aktifkan OpenSSH
3. `copy_binary_to_system()` — Copy binary ke `/usr/local/bin/melisa`
4. `setup_ssh_firewall(firewall_tool)` — Buka port SSH di firewall
5. `setup_lxc_network_quota()` — Konfigurasi bridge dan NAT
6. `register_melisa_shell()` — Daftarkan ke `/etc/shells`
7. `configure_system_sudoers_access()` — Setup sudoers global
8. `fix_ui_permissions()` — Perbaiki izin terminal

**Deteksi Sesi Remote:**
```rust
async fn is_risky_remote_session() -> bool {
    env::var("SSH_CLIENT").is_ok() || 
    env::var("SSH_TTY").is_ok() || 
    env::var("SSH_CONNECTION").is_ok()
}
```