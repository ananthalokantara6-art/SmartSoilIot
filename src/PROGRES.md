# Progres Modul 2 — Buffer & Queue

**Nama:** [isi nama kamu]
**Tanggal:** [isi tanggal]

---

## 1. Tugas Saya

Membuat **antrean (queue) immutable** untuk menampung data sensor
sebelum diproses ke database.

---

## 2. Apa Itu Immutable Queue?

Antrean yang **tidak bisa diubah**. Setiap kali tambah/ambil data,
kita membuat antrean BARU — antrean lama tetap utuh.

**Contoh:**
- Antrean A punya 2 data
- Tambah 1 data → bikin Antrean B (3 data)
- Antrean A tetap 2 data (tidak berubah)

---

## 3. Kenapa Pakai Immutable?

1. **Aman** — data tidak berubah tiba-tiba
2. **Gampang di-test** — hasil selalu konsisten
3. **Aman diakses banyak worker** — tidak rebutan
4. **Sesuai prinsip functional programming**

---

## 4. Struktur Data

### Queue<T>

Antrean generik — bisa menampung tipe data apa saja.

| Fungsi | Kegunaan |
|--------|----------|
| `new()` | Bikin antrean kosong |
| `enqueue(item)` | Tambah data, kembalikan antrean BARU |
| `dequeue()` | Ambil data depan, kembalikan (data, sisa) |
| `peek()` | Lihat data depan tanpa ambil |
| `len()` | Hitung jumlah data |
| `is_empty()` | Cek kosong |

---

## 5. Alur Kerja

---

## 6. Status Progres

- [x] Desain struktur data
- [x] Coding `Queue<T>`
- [x] Program demo (`main.rs`) jalan
- [x] Unit test (4 test lolos)
- [ ] Integrasi dengan Modul 1 & 3
- [ ] Worker otomatis + retry

---

## 7. Kendala

- Menunggu bentuk data final dari Modul 1
- **Solusi:** bikin `Queue<T>` generik dulu biar bisa jalan paralel
  tanpa nunggu modul lain

---

## 8. Rencana Selanjutnya

1. Sepakati kontrak data dengan Modul 1
2. Integrasi `Queue<Reading>` dengan data sensor
3. Bikin worker otomatis yang proses antrean
4. Bikin fungsi `retry()` untuk handle kegagalan

---

## 9. Bukti

- `queue.rs` — kode antrean
- `main.rs` — program demo
- `cargo test` — 4 test lolos
- Screenshot hasil `cargo run` dan `cargo test`