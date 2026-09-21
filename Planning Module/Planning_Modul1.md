# Planning Function — Modul 1: Soil Sensor Acquisition and Validation

**Konteks Proyek:** Smart Soil IoT (Pemantauan Kondisi Tanah Gambut/Peatland)
**Bahasa:** Rust
**Cakupan:** Akuisisi data sensor tanah (kelembaban, suhu, pH, EC) dari node IoT berbasis ESP32, kalibrasi & rekalibrasi otomatis, noise filtering & outlier detection, hingga menghasilkan data tanah yang tervalidasi dan terdokumentasi.

**Repository:** Part of Smart Soil IoT monorepo workspace (`crates/soil-sensor-acquisition`)
**Data Source:** Sensor node ESP32 (via WiFi/HTTP atau serial) → pipeline Rust untuk kalibrasi & validasi

> **PENTING**: Nama workspace, path crate, dan struktur `shared library` di dokumen ini adalah **asumsi default** mengikuti pola proyek sejenis (seperti pada Planning Modul 4). Sesuaikan dengan `MASTERPLAN.md` proyek Smart Soil IoT Anda yang sebenarnya jika berbeda.

> **Catatan asumsi RAB:** Berdasarkan RAB *"SMART SOIL IOT: Soil Sensor Acquisition and Validation"* yang Anda lampirkan, **sensor inti** yang dipakai (sesuai *Recommended minimum parameters* pada gambar Module 1) diasumsikan sebagai:
> - DS18B20 Waterproof Temperature Sensor → **suhu tanah**
> - DFRobot Analog Waterproof Capacitive Soil Moisture Sensor → **kelembaban tanah**
> - Sensor pH Tanah + DMS (Data Measurement System) → **pH tanah**
> - DFRobot Gravity Analog TDS Sensor Meter → **electrical conductivity (EC)**
>
> Ditambah **ESP32** (mikrokontroler) dan **Modul ADC ADS1115** (konversi sinyal analog ke digital + reduksi noise) sebagai hardware inti pemroses. Item lain pada RAB (box panel, cable gland, larutan kalibrasi, kabel jumper, lem silikon, heatsink tube, kabel AWG) diperlakukan sebagai **komponen pendukung** (enclosure, proteksi, kalibrasi fisik, wiring) — jika ada item yang seharusnya masuk kategori "sensor inti" tapi belum tercakup, beri tahu saya.

---

## 1. Tujuan Modul

Modul ini bertujuan untuk memastikan **data tanah yang diakuisisi dari node sensor IoT dapat diandalkan** (akurat, bebas noise berlebih, dan terkalibrasi) sebelum digunakan oleh modul lain (analitik, dashboard, atau sistem peringatan) dalam proyek Smart Soil IoT.

Modul ini bertanggung jawab untuk:

* Menerima/membaca data mentah dari sensor tanah (moisture, temperature, pH, EC) melalui node ESP32.
* Mengonversi sinyal analog menjadi nilai digital yang siap diproses (via modul ADC ADS1115).
* Melakukan kalibrasi sensor menggunakan titik referensi (buffer pH 4.00/6.86/9.18, larutan kalibrasi EC 1413 µS/cm) dan mendukung rekalibrasi otomatis berkala.
* Menyaring noise sinyal (filtering) dan mendeteksi outlier/anomali pembacaan.
* Memvalidasi rentang nilai pembacaan sesuai batas fisik yang wajar untuk kondisi tanah gambut.
* Menghasilkan data tanah yang tervalidasi, terstruktur, dan terdokumentasi (termasuk metadata kalibrasi & timestamp).
* Menyediakan interface publik agar modul lain dapat mengonsumsi data tervalidasi ini tanpa perlu tahu detail kalibrasi/filtering internal.

### Batasan Modul

Modul ini **tidak bertanggung jawab** terhadap:

* Desain firmware ESP32 tingkat rendah (pembacaan pin, driver sensor) — modul ini bekerja pada level data yang sudah dikirim node ke sistem (mis. melalui HTTP payload atau serial frame).
* Penyimpanan jangka panjang ke database/time-series storage (tanggung jawab modul penyimpanan/analitik).
* Visualisasi dashboard atau pembuatan alert/notifikasi (tanggung jawab modul lain).
* Desain mekanis enclosure, waterproofing fisik, dan anti-theft (aspek hardware — didokumentasikan di Bagian 2, bukan logic Rust).
* Inferensi lanjutan (mis. prediksi kondisi tanaman) di luar validasi kualitas data.

---

## 2. Pemilihan Sensor & Spesifikasi Hardware (RAB)

Ringkasan RAB *Smart Soil IoT — Soil Sensor Acquisition and Validation* (total **Rp 1.507.154**):

| No | Uraian | Kategori | Fungsi |
| -- | ------ | -------- | ------ |
| 1 | ESP32-32 30 PIN DOIT WiFi Bluetooth IoT | Hardware inti | Mikrokontroler pemroses data sensor |
| 2 | Modul ADC ADS1115 | Hardware inti | Konversi sinyal analog → digital, minimalisir noise |
| 3 | DS18B20 Waterproof Temperature Sensor Probe + Adapter | **Sensor inti (suhu)** | Mengukur suhu tanah |
| 4 | DFRobot Analog Waterproof Capacitive Soil Moisture Sensor | **Sensor inti (kelembaban)** | Mengukur kelembaban tanah |
| 5 | Sensor pH Tanah + DMS (Data Measurement System) | **Sensor inti (pH)** | Mengukur pH tanah, stabilisasi sinyal |
| 6 | DFRobot Gravity Analog TDS Sensor Meter | **Sensor inti (EC)** | Mengukur konduktivitas/EC tanah |
| 7 | Box Panel Outdoor | Pendukung | Enclosure IoT |
| 8 | Cable Gland PG-9 | Pendukung | Penutup jalur kabel pada box |
| 9 | Serbuk Kalibrasi pH (buffer 4.00/6.86/9.18) | Pendukung — kalibrasi | Referensi kalibrasi sensor pH |
| 10 | Cairan Kalibrasi EC/TDS 1413 µS/cm | Pendukung — kalibrasi | Referensi kalibrasi sensor EC |
| 11–12 | Kabel Jumper F-F & M-F | Pendukung | Wiring sensor ke mikrokontroler |
| 13 | Breadboard kecil (opsional) | Pendukung | Pengetesan |
| 14 | Lem Silikon (Dextone) | Pendukung | Sealing bagian sensor rentan air |
| 15–17 | Heatsink tube 4/6/8mm | Pendukung | Isolasi bakar/pelindung sambungan dari air |
| 18 | Kabel AWG 24 (20m) | Pendukung | Perpanjangan kabel perangkat |

> Bagian ini murni dokumentasi hardware (bukti pemenuhan Tahap "Select required sensors and design the sensor node"). Logic Rust pada modul ini **dimulai dari titik data sudah terbaca dari sensor** (raw value / tegangan ADC), bukan driver hardware itu sendiri.

---

## 3. Struktur Data (Domain Model)

### 3.1 Tipe dari Shared Library (asumsi)

```rust
use soil_iot_shared::core::{SensorType, NodeInfo};
```

- `SensorType` — enum jenis sensor tanah (`Moisture`, `Temperature`, `Ph`, `ElectricalConductivity`).
- `NodeInfo` — identitas & lokasi node sensor (mis. `node_id`, `latitude`, `longitude`).

> **Catatan:** Jika proyek Anda belum memiliki shared library, tipe-tipe di atas dapat didefinisikan langsung di modul ini terlebih dahulu, lalu dipindah ke shared library saat modul lain (dashboard/analitik) sudah membutuhkannya juga.

### 3.2 Module-Specific Types

```rust
use chrono::{DateTime, Utc};

/// Nilai mentah dari sensor sebelum dikalibrasi.
#[derive(Debug, Clone)]
pub struct RawSensorReading {
    pub node_id: String,
    pub sensor_type: SensorType,
    pub raw_value: f64,       // hasil ADC (ADS1115) atau nilai digital langsung (DS18B20)
    pub timestamp: DateTime<Utc>,
}

/// Profil kalibrasi satu sensor pada satu node.
#[derive(Debug, Clone)]
pub struct CalibrationProfile {
    pub node_id: String,
    pub sensor_type: SensorType,
    pub offset: f64,
    pub scale: f64,
    pub reference_points: Vec<(f64, f64)>, // (raw, nilai_aktual) dari larutan/buffer kalibrasi
    pub calibrated_at: DateTime<Utc>,
    pub recalibration_interval_days: u32,
}

/// Nilai setelah kalibrasi diterapkan, sebelum divalidasi.
#[derive(Debug, Clone)]
pub struct CalibratedReading {
    pub node_id: String,
    pub sensor_type: SensorType,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
}

/// Hasil akhir: satu snapshot data tanah yang sudah divalidasi & terdokumentasi.
#[derive(Debug, Clone)]
pub struct ValidatedSoilReading {
    pub node_id: String,
    pub timestamp: DateTime<Utc>,
    pub moisture_percent: f64,
    pub temperature_celsius: f64,
    pub ph: f64,
    pub ec_us_cm: f64,
    pub is_valid: bool,
    pub validation_notes: Vec<String>,
    pub calibration_version: DateTime<Utc>, // referensi ke CalibrationProfile.calibrated_at
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    OutOfRange { sensor: SensorType, value: f64 },
    OutlierDetected { sensor: SensorType, value: f64 },
    MissingCalibration { sensor: SensorType },
    SensorTimeout { sensor: SensorType },
    IncompleteReadingSet { missing: Vec<SensorType> },
}
```

### 3.3 Rentang Nilai Wajar (Validasi Fisik — Kondisi Gambut)

| Parameter | Rentang Wajar | Satuan | Catatan |
| --- | --- | --- | --- |
| Kelembaban tanah | 0 – 100 | % | Dari kalibrasi kapasitif |
| Suhu tanah | 10 – 45 | °C | Disesuaikan iklim tropis/gambut |
| pH tanah | 2.5 – 8.0 | – | Tanah gambut cenderung asam (umumnya 3–5) |
| EC (konduktivitas) | 0 – 5000 | µS/cm | Disesuaikan hasil kalibrasi TDS meter |

> Nilai di atas adalah **asumsi awal** — sepakati bersama pembimbing/tim berdasarkan referensi literatur tanah gambut setempat sebelum difinalkan di Bagian 12.

---

## 4. Tahap 1 — Sensor Data Acquisition

Membaca data mentah dari node sensor (via ADC ADS1115 untuk sensor analog, atau protokol digital untuk DS18B20), lalu menyusunnya menjadi struktur `RawSensorReading` yang siap diproses tahap berikutnya.

| Fungsi | Signature | Deskripsi |
| --- | --- | --- |
| `parse_adc_payload` | `fn parse_adc_payload(node_id: &str, sensor: SensorType, raw_adc: u16, timestamp: DateTime<Utc>) -> RawSensorReading` | Mengonversi nilai mentah ADS1115 (0–32767) menjadi `RawSensorReading`. |
| `parse_digital_payload` | `fn parse_digital_payload(node_id: &str, sensor: SensorType, value: f64, timestamp: DateTime<Utc>) -> RawSensorReading` | Untuk sensor digital seperti DS18B20 yang mengirim nilai langsung tanpa ADC. |
| `collect_node_readings` | `fn collect_node_readings(payload: &NodePayload) -> Vec<RawSensorReading>` | Mengumpulkan seluruh pembacaan sensor dari satu payload node menjadi `Vec<RawSensorReading>` menggunakan `.map().collect()`. |
| `check_completeness` | `fn check_completeness(readings: &[RawSensorReading]) -> Result<(), ValidationError>` | Memastikan keempat jenis sensor (moisture, temperature, pH, EC) hadir dalam satu batch; mengembalikan `ValidationError::IncompleteReadingSet` jika ada yang hilang. |

**Person in Charge:** *(isi nama Anda / anggota tim jika berkelompok)* — Data acquisition & payload parsing.

---

## 5. Tahap 2 — Sensor Calibration & Automated Recalibration

Menerapkan kalibrasi (linear atau multi-titik) pada nilai mentah, serta menentukan kapan node perlu direkalibrasi.

| Fungsi | Signature | Deskripsi |
| --- | --- | --- |
| `linear_calibrate` | `fn linear_calibrate(raw: f64, profile: &CalibrationProfile) -> f64` | Pure function: `value = raw * profile.scale + profile.offset`. |
| `fit_calibration_from_points` | `fn fit_calibration_from_points(reference_points: &[(f64, f64)]) -> CalibrationProfile` | Menghitung `scale`/`offset` dari titik referensi (mis. buffer pH 4.00/6.86/9.18) menggunakan regresi linear sederhana (least squares). |
| `apply_calibration` | `fn apply_calibration(reading: &RawSensorReading, profile: &CalibrationProfile) -> Result<CalibratedReading, ValidationError>` | Menerapkan `linear_calibrate` dan membungkus hasil menjadi `CalibratedReading`; mengembalikan `ValidationError::MissingCalibration` jika profil tidak ditemukan untuk `sensor_type`. |
| `needs_recalibration` | `fn needs_recalibration(profile: &CalibrationProfile, now: DateTime<Utc>) -> bool` | Pure function: `true` jika `now - profile.calibrated_at` melebihi `recalibration_interval_days`. |
| `auto_recalibrate` | `fn auto_recalibrate(profile: &CalibrationProfile, new_reference_points: &[(f64, f64)], now: DateTime<Utc>) -> CalibrationProfile` | Menghasilkan `CalibrationProfile` baru dengan `reference_points` terbaru dan `calibrated_at = now`, tanpa memodifikasi profil lama (immutability). |

**Person in Charge:** *(isi nama)* — Calibration logic & recalibration scheduling.

---

## 6. Tahap 3 — Noise Filtering & Outlier Detection

Menghaluskan sinyal dan menyaring pembacaan yang secara statistik tidak wajar sebelum divalidasi lebih lanjut.

| Fungsi | Signature | Deskripsi |
| --- | --- | --- |
| `moving_average` | `fn moving_average(values: &[f64], window: usize) -> Vec<f64>` | Menghaluskan noise sinyal menggunakan rata-rata bergerak dengan `.windows(window).map()`. |
| `median_filter` | `fn median_filter(values: &[f64], window: usize) -> Vec<f64>` | Alternatif filter median, lebih tahan terhadap lonjakan (spike) dibanding rata-rata. |
| `z_score` | `fn z_score(value: f64, mean: f64, std_dev: f64) -> f64` | Pure function: menghitung skor-z suatu nilai terhadap distribusi historis sensor. |
| `is_outlier` | `fn is_outlier(value: f64, history: &[f64], threshold: f64) -> bool` | Menandai nilai sebagai outlier jika `\|z_score\| > threshold` (default `threshold = 3.0`). |
| `remove_outliers` | `fn remove_outliers(readings: &[CalibratedReading], history: &[f64], threshold: f64) -> Vec<CalibratedReading>` | Menyaring `readings` menggunakan `.filter(|r| !is_outlier(...))`. |

**Person in Charge:** *(isi nama)* — Filtering & anomaly detection.

---

## 7. Tahap 4 — Data Validation & Documentation

Memvalidasi hasil akhir terhadap rentang fisik wajar (Bagian 3.3), lalu menyusun `ValidatedSoilReading` yang terdokumentasi lengkap dengan metadata kalibrasi.

| Fungsi | Signature | Deskripsi |
| --- | --- | --- |
| `validate_range` | `fn validate_range(sensor: SensorType, value: f64) -> Result<(), ValidationError>` | Pure function: memeriksa nilai terhadap rentang wajar per jenis sensor (Bagian 3.3); mengembalikan `ValidationError::OutOfRange` jika di luar batas. |
| `build_validated_reading` | `fn build_validated_reading(node_id: &str, timestamp: DateTime<Utc>, readings: &[CalibratedReading], calibration_version: DateTime<Utc>) -> ValidatedSoilReading` | Menggabungkan empat `CalibratedReading` (moisture, temperature, pH, EC) menjadi satu `ValidatedSoilReading`, menjalankan `validate_range` untuk tiap parameter dan mengisi `validation_notes`. |
| `document_reading` | `fn document_reading(reading: &ValidatedSoilReading) -> String` | Menghasilkan ringkasan teks/log terdokumentasi dari satu pembacaan (untuk audit trail), mis. `"[node-01] 2026-09-20T08:00Z: moisture=42.3% (valid), ph=4.1 (valid), ..."`. |

**Person in Charge:** *(isi nama)* — Validation rules & documentation output.

---

## 8. Tahap 5 — Integration Pipeline & Testing

Menggabungkan seluruh tahap menjadi satu pipeline `acquire_and_validate_soil_data()` yang dapat dipanggil modul lain (mis. handler Axum yang menerima payload dari node ESP32).

| Fungsi / Komponen | Signature / Bentuk | Deskripsi |
| --- | --- | --- |
| `acquire_and_validate_soil_data` | `fn acquire_and_validate_soil_data(payload: &NodePayload, profiles: &[CalibrationProfile], history: &SensorHistory) -> Result<ValidatedSoilReading, ValidationError>` | **MODULE ENTRY POINT**: pipeline utama Tahap 1–4. |
| Unit Tests | `mod tests { ... }` | Menguji tiap fungsi Tahap 1–4 secara independen dengan data dummy (`RawSensorReading` buatan, tanpa hardware asli). |
| Pipeline Validation | `cargo test` | Menjalankan seluruh skenario & edge case (Bagian 15) secara otomatis. |

**Person in Charge:** *(seluruh anggota, jika berkelompok)* — Integrasi pipeline & end-to-end testing.

---

## 9. Komposisi Pipeline Utama

```rust
pub fn acquire_and_validate_soil_data(
    payload: &NodePayload,
    profiles: &[CalibrationProfile],
    history: &SensorHistory,
) -> Result<ValidatedSoilReading, ValidationError> {
    let raw_readings = collect_node_readings(payload);
    check_completeness(&raw_readings)?;

    let calibrated: Vec<CalibratedReading> = raw_readings
        .iter()
        .map(|r| {
            let profile = find_profile(profiles, r.node_id.as_str(), r.sensor_type)
                .ok_or(ValidationError::MissingCalibration { sensor: r.sensor_type })?;
            apply_calibration(r, profile)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let filtered: Vec<CalibratedReading> = calibrated
        .into_iter()
        .filter(|r| !is_outlier(r.value, history.for_sensor(r.sensor_type), 3.0))
        .collect();

    if filtered.len() < 4 {
        return Err(ValidationError::IncompleteReadingSet {
            missing: missing_sensor_types(&filtered),
        });
    }

    let calibration_version = profiles
        .iter()
        .map(|p| p.calibrated_at)
        .max()
        .unwrap_or_else(Utc::now);

    Ok(build_validated_reading(
        &payload.node_id,
        payload.timestamp,
        &filtered,
        calibration_version,
    ))
}
```

Pipeline konseptual:

```text
NodePayload (raw ADC / digital values)
  |
[Tahap 1: Sensor Data Acquisition]
  | Vec<RawSensorReading>
[Tahap 2: Calibration & Automated Recalibration]
  | Vec<CalibratedReading>
[Tahap 3: Noise Filtering & Outlier Detection]
  | Vec<CalibratedReading> (bersih)
[Tahap 4: Validation & Documentation]
  |
ValidatedSoilReading (output lengkap, siap dikonsumsi modul lain)
```

---

## 10. Prinsip Functional Programming yang Perlu Dipegang

### Pure Functions

`linear_calibrate`, `z_score`, `is_outlier`, dan `validate_range` sebaiknya murni — hanya bergantung pada argumen, tanpa efek samping. Fungsi yang melibatkan I/O (mis. membaca payload dari jaringan) sebaiknya dipisah tegas dari fungsi analitik murni.

### Immutability

`auto_recalibrate` menghasilkan `CalibrationProfile` **baru**, bukan memodifikasi profil lama secara langsung — penting agar riwayat kalibrasi tetap dapat diaudit.

### Higher-Order Functions

Manfaatkan iterator seperti `.map()`, `.filter()`, `.windows()`, `.fold()`, `.collect()` untuk transformasi data sensor, hindari `for` loop manual di logic inti (filtering, kalibrasi, validasi).

### Function Composition

```text
collect_node_readings
    |
apply_calibration (per reading)
    |
remove_outliers
    |
build_validated_reading
    |
document_reading
```

Setiap fungsi punya satu tanggung jawab jelas dan dapat diuji independen dengan data dummy tanpa hardware nyata.

---

## 11. Pembagian Kerja

> Sesuaikan jumlah baris dengan jumlah anggota tim Anda di Modul 1. Jika dikerjakan sendiri, satu orang dapat memegang seluruh tahap secara berurutan.

| # | Tahap | Fungsi Utama | PIC | Status |
| - | --- | --- | --- | --- |
| 1 | Sensor Data Acquisition | `parse_adc_payload`, `parse_digital_payload`, `collect_node_readings`, `check_completeness` | *(isi nama)* | Belum dimulai |
| 2 | Calibration & Automated Recalibration | `linear_calibrate`, `fit_calibration_from_points`, `apply_calibration`, `needs_recalibration`, `auto_recalibrate` | *(isi nama)* | Belum dimulai |
| 3 | Noise Filtering & Outlier Detection | `moving_average`, `median_filter`, `z_score`, `is_outlier`, `remove_outliers` | *(isi nama)* | Belum dimulai |
| 4 | Validation & Documentation | `validate_range`, `build_validated_reading`, `document_reading` | *(isi nama)* | Belum dimulai |
| 5 | Integration & Testing | `acquire_and_validate_soil_data`, unit test, pipeline validation | Seluruh anggota | Belum dimulai |

---

## 12. Kesepakatan (Jika Berkelompok / dengan Pembimbing)

Sebelum implementasi dimulai, sepakati:

* Rentang nilai wajar final untuk moisture/temperature/pH/EC pada kondisi tanah gambut lokasi Anda (Bagian 3.3 masih asumsi awal).
* Format payload yang dikirim node ESP32 ke sistem (JSON via HTTP, atau frame serial) — struktur `NodePayload`.
* Interval rekalibrasi default (`recalibration_interval_days`) — mis. 14 atau 30 hari.
* Threshold outlier (`z_score` default 3.0) — sesuaikan dengan karakteristik noise sensor kapasitif di lapangan.
* Jumlah minimum titik referensi kalibrasi (mis. minimal 2 titik untuk linear, 3 titik untuk buffer pH 4.00/6.86/9.18).
* Format `validation_notes` dan `document_reading` (teks bebas atau template terstruktur/JSON log).
* Strategi testing: unit test dengan data dummy `RawSensorReading`, tanpa perlu koneksi node fisik aktif.

---

## 13. Independensi Modul

Modul ini dirancang sebagai komponen independen dalam proyek Smart Soil IoT:

* Modul dapat dikembangkan dan diuji secara mandiri menggunakan data dummy (`RawSensorReading` buatan), tanpa node ESP32 fisik yang menyala.
* Modul hanya bergantung pada tipe data bersama (`SensorType`, `NodeInfo`) jika proyek Anda memakai shared library; jika belum ada, definisikan lokal dulu di modul ini.
* Output modul (`ValidatedSoilReading`) menjadi kontrak data untuk modul lain (mis. modul dashboard/analitik/alert) — modul lain **tidak perlu tahu** detail kalibrasi/filtering internal.
* Jangan asumsikan dependency langsung ke modul lain (dashboard, alert, dsb.) — komunikasi terjadi lewat interface publik (Bagian 17).

```text
      Smart Soil IoT
            │
       ┌────┴────┐
       │         │
  Modul ini   Modul lain
   (M1)     (Dashboard/Analitik/Alert, dst.)
```

Modul ini (M1) adalah **sumber data tervalidasi** bagi modul-modul lain dalam proyek.

---

## 14. Kriteria Selesai Modul

* [ ] Seluruh fungsi Tahap 1–4 telah diimplementasikan.
* [ ] Setiap fungsi memiliki unit test dengan data dummy (mencakup kondisi normal & edge case).
* [ ] Pipeline `acquire_and_validate_soil_data()` berjalan end-to-end dari payload mentah hingga `ValidatedSoilReading`.
* [ ] Kalibrasi linear/multi-titik terbukti benar menggunakan data referensi buffer pH & larutan EC dari RAB.
* [ ] Rekalibrasi otomatis (`needs_recalibration`/`auto_recalibrate`) teruji dengan skenario waktu berbeda.
* [ ] Noise filtering & outlier detection terbukti menghilangkan lonjakan nilai palsu tanpa membuang data valid secara berlebihan.
* [ ] Validasi rentang nilai berjalan untuk keempat parameter (moisture, temperature, pH, EC).
* [ ] Tidak ada dependency yang tidak perlu terhadap modul lain.
* [ ] Dokumentasi fungsi (rustdoc) tersedia untuk seluruh `pub fn`.
* [ ] Ada contoh penggunaan modul (payload contoh → hasil `ValidatedSoilReading`).
* [ ] Modul dapat dijalankan/diuji secara independen (`cargo test`).

---

## 15. Contoh Skenario Pengujian

### Skenario 1 — Pembacaan Normal, Semua Sensor Valid

**Input:**

```text
NodePayload {
  node_id: "node-01",
  raw: { moisture: 18500 (ADC), temperature: 27.4 (digital), ph_raw: 15200 (ADC), ec_raw: 9800 (ADC) }
}
CalibrationProfile untuk keempat sensor sudah tersedia dan belum kedaluwarsa.
```

**Expected Output:**

```text
ValidatedSoilReading {
  moisture_percent: 42.3,
  temperature_celsius: 27.4,
  ph: 4.1,
  ec_us_cm: 1320.0,
  is_valid: true,
  validation_notes: [],
}
```

**Fungsi yang diuji:** `collect_node_readings`, `apply_calibration`, `remove_outliers`, `validate_range`, `build_validated_reading`.

---

### Skenario 2 — Nilai pH di Luar Rentang Wajar

**Input:** hasil kalibrasi pH menghasilkan `ph = 9.5` (di luar rentang 2.5–8.0 pada Bagian 3.3).

**Expected Output:**

```text
ValidatedSoilReading {
  ...
  is_valid: false,
  validation_notes: ["ph: 9.5 out of expected range (2.5-8.0)"],
}
```

**Fungsi yang diuji:** `validate_range`, `build_validated_reading`.

---

### Skenario 3 — Rekalibrasi Diperlukan

**Input:** `CalibrationProfile.calibrated_at` = 40 hari lalu, `recalibration_interval_days = 30`.

**Expected Output:** `needs_recalibration(...) == true`, sistem memicu `auto_recalibrate` menggunakan titik referensi baru.

**Fungsi yang diuji:** `needs_recalibration`, `auto_recalibrate`.

---

### Edge Cases

| Case | Input | Expected Result |
| --- | --- | --- |
| Salah satu sensor tidak mengirim data | payload hanya berisi 3 dari 4 sensor | `check_completeness` mengembalikan `Err(ValidationError::IncompleteReadingSet)`. |
| Profil kalibrasi belum ada untuk node | `profiles` kosong untuk `node_id` tersebut | `apply_calibration` mengembalikan `Err(ValidationError::MissingCalibration)`. |
| Lonjakan noise tunggal (spike) | satu nilai jauh di luar riwayat (`z_score` > 3.0) | `is_outlier` bernilai `true`, `remove_outliers` membuang nilai tersebut tanpa panic. |
| Riwayat data kosong (node baru) | `history` kosong | `is_outlier` mengembalikan `false` (tidak ada dasar pembanding), bukan panic. |
| Nilai sensor identik terus-menerus (sensor macet) | seluruh nilai sama dalam window tertentu | Ditandai pada `validation_notes` sebagai potensi sensor macet (opsional, sepakati formatnya). |
| Payload dengan timestamp mundur (clock drift node) | `timestamp` lebih lama dari data terakhir | Pipeline tetap berjalan tanpa panic; dicatat sebagai catatan validasi. |

---

## 16. Langkah Selanjutnya

1. Konfirmasi ulang rentang nilai wajar (Bagian 3.3) dengan data lapangan/tanah gambut yang sebenarnya.
2. Tentukan format `NodePayload` final (JSON/HTTP atau serial) bersama tim yang mengerjakan firmware ESP32.
3. Sepakati interval rekalibrasi dan threshold outlier (Bagian 12).
4. Siapkan data dummy `RawSensorReading` (independen dari hardware) untuk unit test awal.
5. Implementasikan Tahap 1–4 secara berurutan (atau paralel jika berkelompok, sesuai Bagian 11).
6. Tulis unit test untuk tiap fungsi, termasuk edge case pada Bagian 15.
7. Gabungkan seluruh tahap ke dalam `acquire_and_validate_soil_data()`.
8. Uji end-to-end dengan data dummy, lalu dengan data node ESP32 sungguhan setelah hardware siap (lihat RAB Bagian 2).
9. Lengkapi rustdoc untuk seluruh `pub fn` dan tipe publik.
10. Dokumentasikan contoh penggunaan modul untuk dikonsumsi modul lain (dashboard/analitik).

---

## 17. Interface Publik & Komunikasi Antar Modul

### 17.1 Batas Modul (Owns / Does Not Own / Internal / Public)

| Aspek | Isi |
| --- | --- |
| **Owns** | Parsing payload node, kalibrasi & rekalibrasi, noise filtering & outlier detection, validasi rentang nilai, dokumentasi hasil pembacaan. |
| **Does Not Own** | Firmware/driver hardware ESP32, penyimpanan jangka panjang, dashboard/visualisasi, alerting. |
| **Internal** | `parse_adc_payload`, `parse_digital_payload`, `check_completeness`, `linear_calibrate`, `fit_calibration_from_points`, `needs_recalibration`, `auto_recalibrate`, `moving_average`, `median_filter`, `z_score`, `is_outlier`, `remove_outliers`, `validate_range`, `build_validated_reading`, `document_reading`. |
| **Publicly Exposes** | `acquire_and_validate_soil_data` (utama) dan `apply_calibration` (opsional, jika modul lain butuh kalibrasi ad-hoc). |

### 17.2 Fungsi Publik (`pub fn`) — Modul 1

| `pub fn` | Provider | Consumer Potensial | Purpose | Input | Output |
| --- | --- | --- | --- | --- | --- |
| `acquire_and_validate_soil_data` | Modul 1 | Handler Axum `/api/v1/soil-data`, modul dashboard/analitik | Pipeline utama Tahap 1–4 | `&NodePayload`, `&[CalibrationProfile]`, `&SensorHistory` | `Result<ValidatedSoilReading, ValidationError>` |
| `apply_calibration` *(opsional publik)* | Modul 1 | Internal pipeline; opsional modul lain | Kalibrasi satu pembacaan | `&RawSensorReading`, `&CalibrationProfile` | `Result<CalibratedReading, ValidationError>` |

```rust
/// Mengakuisisi dan memvalidasi data tanah dari satu payload node sensor.
///
/// # Arguments
/// * `payload` - Data mentah dari node ESP32.
/// * `profiles` - Profil kalibrasi aktif untuk node terkait.
/// * `history` - Riwayat pembacaan untuk deteksi outlier.
///
/// # Returns
/// `ValidatedSoilReading` yang siap dikonsumsi modul lain, atau `ValidationError`.
pub fn acquire_and_validate_soil_data(
    payload: &NodePayload,
    profiles: &[CalibrationProfile],
    history: &SensorHistory,
) -> Result<ValidatedSoilReading, ValidationError>
```

### 17.3 Komunikasi Antar Modul (Provider → Receiver)

```text
Modul 1 (Provider)
      │
      │ pub fn acquire_and_validate_soil_data(...)
      ▼
Axum handler /api/v1/soil-data  (Receiver / Aplikasi)
      │  hasil: ValidatedSoilReading
      ▼
Modul Dashboard / Analitik / Alert (Receiver)
```

| Provider | `pub fn` | Receiver | Purpose | Data yang Dikirim | Priority |
| --- | --- | --- | --- | --- | --- |
| Modul 1 | `acquire_and_validate_soil_data` | Axum handler | Endpoint ingest data sensor | `NodePayload` | **Required** (API) |
| Modul 1 | `acquire_and_validate_soil_data` | Modul Dashboard/Analitik | Sumber data tervalidasi untuk ditampilkan/dianalisis | `ValidatedSoilReading` | Required |

### 17.4 Rustdoc — Modul 1

Rustdoc diwajibkan untuk seluruh `pub fn` dan tipe publik yang muncul di signature-nya (`ValidatedSoilReading`, `ValidationError`, `CalibrationProfile`, dll).

```bash
cargo doc --no-deps --open
cargo check
cargo test
```

Status saat ini di dokumen: **Rustdoc planned** (belum diklaim verified).

---

## 18. Rubrik — Evidence Modul 1

| Rubrik | Evidence di Planning Modul 1 | Bukti Implementasi yang Masih Diperlukan |
| --- | --- | --- |
| Repositori Github (15%) | Lokasi `crates/soil-sensor-acquisition` dan command build/test/rustdoc | Repositori dibuat & diakses dosen |
| Prioritas Modul (25%) | Prioritas fitur: akuisisi → kalibrasi → filtering → validasi → dokumentasi | Implementasi fitur prioritas |
| Rustdoc (20%) | Rustdoc diwajibkan untuk semua `pub fn` (Bagian 17.4) | Generate & aksesibel |
| Komunikasi Antar Module (40%) | Batas `mod`/`pub fn` (17.1–17.2), matriks komunikasi (17.3) | Interface diimplementasikan & diuji |

> Sesuaikan bobot/nama rubrik di atas jika rubrik penilaian mata kuliah Anda untuk Modul 1 berbeda dari yang digunakan pada Planning Modul 4.
