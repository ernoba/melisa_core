# Gambaran Arsitektur MELISA

Halaman ini menjelaskan arsitektur teknis MELISA secara menyeluruh — bagaimana komponen-komponen saling berinteraksi, alur data, dan keputusan desain yang mendasari sistem.

---

## Diagram Arsitektur Tingkat Tinggi

```
┌─────────────────────────────────────────────────────────────────┐
│                      MESIN LOKAL (Client)                       │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  melisa-client (Rust)                                   │   │
│  │  ├── auth.rs      → Manajemen profil & SSH key          │   │
│  │  ├── exec.rs      → Eksekusi perintah remote            │   │
│  │  ├── db.rs        → Penyimpanan profil lokal            │   │
│  │  ├── filter.rs    → Validasi input client               │   │
│  │  └── platform.rs  → Abstraksi platform (Linux/Mac/Win)  │   │
│  └─────────────────────────────────────────────────────────┘   │
└───────────────────────────┬─────────────────────────────────────┘
                            │ SSH (port 22, kunci publik/privat)
┌───────────────────────────▼─────────────────────────────────────┐
│                      SERVER LINUX (Host)                        │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  melisa (Rust) — Berjalan sebagai root                  │   │
│  │                                                         │   │
│  │  ┌──────────────────────────────────────────────────┐   │   │
│  │  │  CLI Layer                                       │   │   │
│  │  │  ├── melisa_cli.rs  → REPL loop utama            │   │   │
│  │  │  ├── executor.rs    → Parser & dispatcher         │   │   │
│  │  │  ├── helper.rs      → Tab completion & hints      │   │   │
│  │  │  ├── prompt.rs      → Tampilan prompt             │   │   │
│  │  │  ├── loading.rs     → Spinner animasi             │   │   │
│  │  │  ├── wellcome.rs    → Banner & dashboard boot     │   │   │
│  │  │  └── color.rs       → Pewarnaan terminal          │   │   │
│  │  └──────────────────────────────────────────────────┘   │   │
│  │                           │                             │   │
│  │  ┌──────────────────────────────────────────────────┐   │   │
│  │  │  Security Gate (Input Guard)                     │   │   │
│  │  │  └── guard/filter.rs → Validasi semua input      │   │   │
│  │  └──────────────────────────────────────────────────┘   │   │
│  │                           │                             │   │
│  │  ┌────────────┬────────────┬────────────────────────┐   │   │
│  │  │  Core      │ Deployment │  Distros               │   │   │
│  │  │            │            │                        │   │   │
│  │  │ container/ │ manifest/  │ host_distro.rs         │   │   │
│  │  │ user/      │ deployer.rs│ lxc_distro.rs          │   │   │
│  │  │ project/   │ dependency │                        │   │   │
│  │  │ setup.rs   │            │                        │   │   │
│  │  │ metadata.rs│            │                        │   │   │
│  │  └────────────┴────────────┴────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Infrastruktur Host                                     │   │
│  │  ├── LXC Runtime           → lxc-create, lxc-start...  │   │
│  │  ├── Network Bridge        → lxcbr0 (10.0.3.0/24)      │   │
│  │  ├── NAT Routing           → iptables PREROUTING        │   │
│  │  ├── DNS                   → dnsmasq                    │   │
│  │  ├── SSH Server            → OpenSSH                    │   │
│  │  ├── Firewall              → UFW / Firewalld / iptables │   │
│  │  └── Git                   → /var/melisa/projects/      │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Struktur Direktori Kode

```
melisa_core/
├── src/                          # Server MELISA
│   ├── main.rs                   # Entry point, root check, async runtime
│   ├── cli/                      # Lapisan antarmuka pengguna
│   │   ├── mod.rs
│   │   ├── melisa_cli.rs         # REPL loop utama (rustyline)
│   │   ├── executor.rs           # Parser perintah & dispatcher
│   │   ├── helper.rs             # Tab completion, hints, highlighting
│   │   ├── prompt.rs             # Tampilan prompt & info sesi
│   │   ├── loading.rs            # Spinner animasi (indicatif)
│   │   ├── wellcome.rs           # Banner boot & dashboard sistem
│   │   └── color.rs              # Konstanta warna ANSI
│   ├── core/                     # Logika bisnis inti
│   │   ├── mod.rs
│   │   ├── container/            # Manajemen container LXC
│   │   │   ├── mod.rs            # Re-export publik
│   │   │   ├── lifecycle.rs      # Create, start, stop, delete, attach
│   │   │   ├── network.rs        # Bridge, NAT, DNS, shared folders
│   │   │   ├── query.rs          # List, info, IP, send, upload
│   │   │   └── types.rs          # Tipe data: path, status, metadata
│   │   ├── guard/                # Input Guard (keamanan)
│   │   │   ├── mod.rs
│   │   │   └── filter.rs         # Validasi input: injection, traversal
│   │   ├── project/              # Manajemen proyek kolaboratif
│   │   │   ├── mod.rs
│   │   │   └── management.rs     # CRUD proyek, invite, pull, distribute
│   │   ├── user/                 # Manajemen pengguna sistem
│   │   │   ├── mod.rs
│   │   │   ├── management.rs     # Add, delete, list, password, upgrade
│   │   │   ├── sudoers.rs        # Konfigurasi /etc/sudoers.d/
│   │   │   └── types.rs          # Enum UserRole (Admin/Regular)
│   │   ├── metadata.rs           # Metadata container (simpan/baca/hapus)
│   │   ├── root_check.rs         # Verifikasi hak root/admin
│   │   └── setup.rs              # Setup host otomatis
│   ├── deployment/               # Engine deployment .mel
│   │   ├── mod.rs
│   │   ├── manifest/             # Parser file .mel
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs         # Load & validasi file .mel
│   │   │   └── types.rs          # Struct MelManifest dan turunannya
│   │   ├── dependency.rs         # Instalasi dependensi (apt, pip, npm...)
│   │   └── deployer.rs           # Orkestrator deployment (cmd_up, cmd_down)
│   └── distros/                  # Deteksi & konfigurasi distribusi
│       ├── mod.rs
│       ├── host_distro.rs        # Deteksi distro host, konfigurasi paket
│       └── lxc_distro.rs         # Daftar distro container dari LXC repo
│
├── client/                       # Client MELISA (lintas platform)
│   ├── src/
│   │   ├── main.rs               # Entry point client
│   │   ├── auth.rs               # Manajemen profil & deploy SSH key
│   │   ├── exec.rs               # Eksekusi perintah remote via SSH
│   │   ├── db.rs                 # Penyimpanan profil lokal
│   │   ├── filter.rs             # Validasi input client
│   │   ├── color.rs              # Output berwarna (cross-platform)
│   │   └── platform.rs           # Abstraksi path, SSH bin, dependencies
│   ├── install.sh                # Installer Linux/macOS
│   ├── install.ps1               # Installer Windows (PowerShell)
│   └── Cargo.toml
│
├── doc/                          # Aset dokumentasi
│   ├── melisa_core.drawio        # Diagram arsitektur (draw.io)
│   └── dokumentasi.pages         # Draft dokumen (Apple Pages)
│
└── Cargo.toml                    # Dependensi server
```

---

## Alur Eksekusi Perintah

Berikut adalah alur lengkap dari input pengguna hingga eksekusi:

```
Pengguna mengetik: "melisa --create myapp ubuntu/jammy/amd64"
         │
         ▼
[1] rustyline REPL menerima input
         │
         ▼
[2] Input Guard (filter_input)
    ├── Cek panjang ≤ 1024 karakter ✓
    ├── Cek tidak ada null byte ✓
    ├── Cek shell injection ✓
    └── Cek path traversal ✓
         │
         ▼ (jika lolos)
[3] execute_command(input, user, home)
         │
         ▼
[4] parse_command → tokenisasi
    tokens = ["melisa", "--create", "myapp", "ubuntu/jammy/amd64"]
    audit  = false
         │
         ▼
[5] dispatch_melisa_subcommand
    match tokens[1] → "--create"
         │
         ▼
[6] handle_create_container("myapp", "ubuntu/jammy/amd64", false)
    ├── get_lxc_distro_list() → validasi kode distro
    ├── execute_with_spinner("Creating container...")
    │       ├── create_container("myapp", metadata, spinner, false)
    │       │       ├── verify_host_runtime(false)
    │       │       ├── lxc-create -n myapp -t download -- -d ubuntu ...
    │       │       ├── inject_network_config("myapp")
    │       │       ├── setup_container_dns("myapp")
    │       │       ├── lxc-start -n myapp
    │       │       ├── wait_for_network_initialization("myapp")
    │       │       ├── apt-get update && apt-get upgrade (di container)
    │       │       └── write_container_metadata("myapp", ...)
    │       └── [OK] Container ready
         │
         ▼
[7] ExecResult::Continue → REPL kembali ke prompt
```

---

## Komponen Async Runtime

MELISA menggunakan **Tokio** sebagai async runtime untuk menangani operasi I/O yang banyak (subprocess, file system, network) tanpa memblokir thread utama.

```rust
// main.rs — inisialisasi runtime
let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap();

runtime.block_on(async {
    cli::melisa_cli::melisa().await;
});
```

- **Multi-thread runtime** — memanfaatkan semua core CPU
- **`enable_all()`** — mengaktifkan timer, I/O, dan signal handling

---

## REPL — Read-Eval-Print Loop

MELISA menggunakan library **rustyline** untuk REPL yang kaya fitur:

| Fitur rustyline | Implementasi |
|-----------------|-------------|
| Tab completion | `FilenameCompleter` + `MelisaHelper` |
| Highlighting | `MatchingBracketHighlighter` |
| Hints | `HistoryHinter` (saran berdasarkan riwayat) |
| Bracket validation | `MatchingBracketValidator` |
| Riwayat persisten | `~/.local/share/melisa/history.txt` |
| Duplikasi riwayat | Diabaikan otomatis |

---

## Metadata Container

MELISA menyimpan metadata setiap container untuk tracking dan inspeksi. Metadata mencakup:

- Nama container
- Distribusi dan rilis
- Tanggal pembuatan
- Status terakhir
- Konfigurasi jaringan (IP, MAC)

Metadata disimpan sebagai file di sistem (lokasi internal, tidak terekspos ke pengguna secara langsung).

---

## Manajemen Dependensi (Cargo.toml)

| Dependensi | Kegunaan |
|------------|----------|
| `tokio` | Async runtime (multi-thread, full features) |
| `rustyline` | REPL interaktif (derive features untuk `#[derive(Helper)]`) |
| `indicatif` | Spinner dan progress bar animasi |
| `serde` + `toml` | Serialisasi/deserialisasi file `.mel` |
| `thiserror` | Tipe error terstruktur (`MelParseError`) |
| `rand` | Pembangkitan MAC address acak untuk jaringan container |
| `libc` | Deteksi UID untuk verifikasi hak root (`geteuid()`) |
| `colored` | Output terminal berwarna (cross-platform) |
| `sysinfo` | Informasi sistem (RAM, CPU, hostname) untuk dashboard |
| `chrono` | Formatting tanggal/waktu |