# Instalasi

MELISA terdiri dari dua komponen yang diinstall secara terpisah:

1. **Server MELISA** — diinstall di mesin Linux yang akan menjadi host container
2. **Client MELISA** — diinstall di mesin lokal (laptop/PC) administrator

---

## Gambaran Alur Instalasi

```
┌─────────────────────────────┐      ┌─────────────────────────────┐
│      MESIN LOKAL            │      │         SERVER LINUX        │
│  (Linux / macOS / Windows)  │      │    (host container LXC)     │
│                             │      │                             │
│  melisa-client              │──SSH─▶  melisa (server binary)    │
│  - auth add                 │      │  - Setup LXC               │
│  - Mengelola profil SSH     │      │  - Manajemen container      │
│                             │      │  - Manajemen pengguna       │
└─────────────────────────────┘      └─────────────────────────────┘
```

---

## Pilih Panduan Instalasi

Lanjutkan ke bagian yang sesuai dengan kebutuhan Anda:

### → [Instalasi Server](./instalasi-server.md)
Untuk menginstal MELISA di server Linux yang akan menjadi host container LXC.

### → [Instalasi Client](./instalasi-client.md)
Untuk menginstal klien MELISA di mesin lokal Anda (Linux, macOS, atau Windows) agar dapat terhubung ke server dari jarak jauh.

---

## Urutan yang Disarankan

Jika Anda memulai dari awal:

1. ✅ Install server MELISA di mesin Linux
2. ✅ Jalankan `melisa --setup` untuk konfigurasi host
3. ✅ Install client MELISA di mesin lokal
4. ✅ Tambahkan profil koneksi dengan `melisa auth add`
5. ✅ Mulai membuat container!