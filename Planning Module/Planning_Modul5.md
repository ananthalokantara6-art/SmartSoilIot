# Spesifikasi Desain & Perencanaan Teknis: Module 5
## Plant-Specific Expert System
**Proyek:** Smart Soil IoT
**Target Crate:** `module_5_plant_specific_expert_system`
**Mata Kuliah:** Pemrograman Fungsional
**Bahasa Pemrograman:** Rust

---

## 1. Pendahuluan & Ruang Lingkup Sistem

Module 5 (**Plant-Specific Expert System**) bertindak sebagai mesin pakar berbasis aturan (*rule-based expert system*) di dalam ekosistem **Smart Soil IoT**. Modul ini menjembatani hasil evaluasi mutu tanah yang dihimpun oleh Module 4 dengan kebutuhan agronomi spesifik per-jenis tanaman, mengubah pasangan (kondisi tanah, nama tanaman) menjadi diagnosis kecocokan per-parameter, faktor pembatas (*limiting factors*), rekomendasi tindakan bertingkat prioritas, skor kesesuaian numerik, dan ringkasan tekstual yang siap dikonsumsi pengguna maupun sistem lain.

Seluruh arsitektur modul dirancang secara **fungsional murni (*pure functional core*)**, di mana seluruh logika evaluasi deterministik, bebas efek samping (*side-effect free*), tidak memiliki state global, dan menjamin integritas data melalui struktur data yang tidak dapat diubah (*immutable*).

### 1.1 Matriks Batas Tanggung Jawab Modul

Untuk menjaga kohesi sistem yang tinggi dan ketergantungan antar-komponen yang longgar (*loose coupling*), batas ruang lingkup Module 5 didefinisikan secara tegas:

| Bidang Tanggung Jawab | Di Dalam Lingkup Module 5 (In Scope) | Di Luar Lingkup Module 5 (Out of Scope) |
| :--- | :--- | :--- |
| **Akuisisi Data** | Menerima `SoilAssessment` yang telah tervalidasi dalam bentuk struktur data memori. | Pembacaan sinyal ADC sensor, interfacing kabel fisik, dan kalibrasi tegangan listrik (*Tanggung jawab Module 1*). |
| **Analitik Tren & SQI** | Mengonsumsi hasil evaluasi mutu tanah sebagai masukan murni tanpa menghitung ulang. | Perhitungan Soil Quality Index, statistik deret waktu, dan deteksi anomali agroekologi (*Tanggung jawab Module 4*). |
| **Rule Base Agronomi** | Menyimpan & mengelola profil kebutuhan ideal multi-tanaman (`PlantProfile`), diagnosis kecocokan per-parameter. | Penelitian biologi tanaman baru, pemeliharaan database varietas bibit resmi, dan prediksi hama/penyakit. |
| **Preskripsi Tindakan** | Menghasilkan rekomendasi tekstual bertingkat prioritas yang spesifik terhadap tanaman target. | Pensaklaran relay fisik, eksekusi aktuator pompa/dispenser pupuk (*Tanggung jawab Orchestrator / Aktuator*). |
| **Penyimpanan & UI** | Menghasilkan struktur laporan pakar terstruktur (`PlantExpertReport`) siap dikonsumsi. | Penyimpanan persisten basis data time-series dan rendering grafik visual dashboard (*Tanggung jawab Module 3*). |
| **Integrasi Sistem** | Mengekspos fungsi publik (`pub fn`) granular untuk dikomposisikan oleh `smart_soil_orchestrator`. | Mekanisme plugin dan ekstensibilitas sistem (*Tanggung jawab Module 6*). |

### 1.2 Alur Integrasi Data Antar Modul

Mengacu pada pola integrasi yang telah didefinisikan pada **Planning Modul 3** (bagian "Integrasi dengan Modul Lain") dan **Planning Modul 4** (bagian "Ruang Lingkup Sistem"), Module 5 diposisikan sebagai konsumen hasil analitik Module 4 sekaligus penyedia data preskriptif bagi Module 3/Orchestrator. Tidak ada *shared library* lintas modul pada workspace ini — setiap modul berkomunikasi hanya lewat `pub fn`/tipe publik masing-masing, dikomposisikan di `smart_soil_orchestrator`.

**Data masuk dari Module 4** (`module_4_soil_analytics_and_decision_support`):

Module 4 menghasilkan `SoilReading`/`RecommendationReport` dengan penamaan field yang berbeda dari tipe internal Module 5 (mis. `moisture_percentage` vs `moisture_percent`, `ph_level` vs `ph`). Karena kedua modul tidak saling bergantung secara Cargo, konversi dilakukan melalui *adapter* ringan di lapisan orchestrator:

```rust
// Dijalankan di smart_soil_orchestrator, bukan di dalam crate Module 5.
fn to_soil_assessment(reading: &module_4::SoilReading) -> module_5::SoilAssessment {
    module_5::SoilAssessment {
        ph: reading.ph_level,
        moisture_percent: reading.moisture_percentage,
        nitrogen_ppm: reading.nitrogen_ppm,
        phosphorus_ppm: reading.phosphorus_ppm,
        potassium_ppm: reading.potassium_ppm,
        temperature_celsius: reading.temperature_celsius,
    }
}
```

**Data keluar ke Module 3 & Orchestrator:**

`PlantExpertReport` yang dihasilkan `evaluate_plant_suitability` bersifat siap-tampil (*display-ready*) — Module 3 tidak perlu mengetahui logika rule base, cukup menerima `diagnoses`, `limiting_factors`, `recommendations`, `suitability_score`, dan `summary` untuk dirender pada dashboard (selaras dengan batasan Module 3 yang tidak menentukan aturan agronomi tanaman).

**Diagram alur data (gaya Planning Modul 3):**

```text
Modul 4 (Soil Analytics)
   ↓  SoilReading / RecommendationReport
Adapter (smart_soil_orchestrator)
   ↓  SoilAssessment
Modul 5 (Plant-Specific Expert System)
   ↓  PlantExpertReport
Modul 3 (Dashboard) / Orchestrator
   ↓
Rekomendasi tanaman & tindakan agronomi ke pengguna
```

Catatan: selama pengembangan paralel, Module 5 tetap dapat diuji sepenuhnya mandiri menggunakan `SoilAssessment` dummy dan `default_rule_base()`, tanpa menunggu Module 4 maupun Module 3 selesai — konsisten dengan prinsip *loose coupling* pada Bagian 1.1.

---

## 2. Pemodelan Domain Berbasis Algebraic Data Types (ADTs)

Modul ini mendefinisikan tipe domainnya sendiri secara independen (tidak ada shared library lintas modul pada workspace ini) untuk merepresentasikan asesmen tanah, profil kebutuhan tanaman, status diagnosis, dan preskripsi keputusan secara presisi tanpa celah keadaan yang tidak valid (*invalid state unrepresentable*).

### 2.1 Model Asesmen Tanah & Profil Kebutuhan Tanaman

```rust
/// Rentang nilai ideal (inklusif) untuk satu parameter agronomi.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}

/// Data hasil asesmen tanah yang diterima dari Module 4.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoilAssessment {
    pub ph: f64,                    // 0.0 - 14.0
    pub moisture_percent: f64,      // 0.0 - 100.0%
    pub nitrogen_ppm: f64,          // mg/kg
    pub phosphorus_ppm: f64,        // mg/kg
    pub potassium_ppm: f64,         // mg/kg
    pub temperature_celsius: f64,   // °C
}

/// Profil kebutuhan ideal satu jenis tanaman — satu entri rule base.
#[derive(Debug, Clone, PartialEq)]
pub struct PlantProfile {
    pub plant_name: String,
    pub ph_range: Range,
    pub moisture_range: Range,
    pub nitrogen_range: Range,
    pub phosphorus_range: Range,
    pub potassium_range: Range,
    pub temperature_range: Range,
}
```

### 2.2 Model Diagnosis Parameter & Klasifikasi Status

```rust
/// Klasifikasi posisi nilai parameter terhadap rentang ideal tanaman.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterStatus {
    Deficient,
    Optimal,
    Excess,
}

/// Hasil diagnosis satu parameter tanah terhadap kebutuhan tanaman target.
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterDiagnosis {
    pub parameter: String,
    pub current_value: f64,
    pub ideal_range: Range,
    pub status: ParameterStatus,
    pub deviation: f64,
}
```

### 2.3 Model Preskripsi Rekomendasi & Laporan Pakar

```rust
/// Tingkat urgensi eksekusi satu rekomendasi tindakan agronomi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Satu tindakan yang disarankan terhadap satu parameter yang tidak optimal.
#[derive(Debug, Clone, PartialEq)]
pub struct Recommendation {
    pub parameter: String,
    pub action: String,
    pub priority: Priority,
}

/// Laporan evaluasi kesesuaian tanah-tanaman terintegrasi hasil pemrosesan Module 5.
#[derive(Debug, Clone, PartialEq)]
pub struct PlantExpertReport {
    pub plant_name: String,
    pub diagnoses: Vec<ParameterDiagnosis>,
    pub limiting_factors: Vec<String>,
    pub recommendations: Vec<Recommendation>,
    pub suitability_score: f64,
    pub summary: String,
}
```

### 2.4 Penanganan Galat Fungsional (Monadic Error Type)

Tidak ada penggunaan runtime `panic!` atau pengecualian tidak tertangani. Kesalahan didefinisikan secara deklaratif:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ExpertError {
    UnknownPlant(String),
    EmptyRuleBase,
    InvalidAssessment(String),
}
```

---

## 3. Spesifikasi Fungsi: Rule Base & Mesin Diagnosis (Pure Diagnosis Engine)

Seluruh fungsi diagnosis dirancang tanpa mutasi keadaan luar, deterministik, dan memanfaatkan fitur ekspresif Rust seperti *closure*, *iterator combinators*, dan fungsi tingkat tinggi (*Higher-Order Functions*).

### 3.1 Sub-Domain: Rule Base & Validasi Masukan (Tahap 1)

| Nama Fungsi | Definisi Tanda Tangan (Signature) | Deskripsi Algoritma & Karakteristik Fungsional |
| :--- | :--- | :--- |
| `default_rule_base` | `fn default_rule_base() -> Vec<PlantProfile>` | Rule base bawaan berisi profil kebutuhan tanah untuk beberapa tanaman umum (Padi, Jagung, Cabai, Kelapa Sawit, Kopi). Nilai indikatif untuk pengujian, disusun sebagai literal data tanpa I/O eksternal. |
| `find_plant_profile` | `fn find_plant_profile<'a>(plant_name: &str, rule_base: &'a [PlantProfile]) -> Result<&'a PlantProfile, ExpertError>` | Mencari profil tanaman berdasarkan nama secara *case-insensitive* menggunakan `.iter().find()`. Mengembalikan `ExpertError::UnknownPlant` apabila tidak ditemukan tanpa menimbulkan panic. |
| `validate_soil_assessment` | `fn validate_soil_assessment(assessment: &SoilAssessment) -> Result<(), ExpertError>` | Memvalidasi bahwa seluruh nilai berada pada rentang fisik yang masuk akal (pH 0–14, kelembapan 0–100%, N/P/K ≥ 0, suhu −10..60 °C) sebelum diproses tahap berikutnya. Mengembalikan `ExpertError::InvalidAssessment` jika tidak valid. |

### 3.2 Sub-Domain: Diagnosis Parameter (Tahap 2)

| Nama Fungsi | Definisi Tanda Tangan (Signature) | Deskripsi Algoritma & Karakteristik Fungsional |
| :--- | :--- | :--- |
| `classify_parameter` | `fn classify_parameter(value: f64, range: &Range) -> ParameterStatus` | *Pure function* yang mengklasifikasikan nilai sebagai `Deficient`/`Optimal`/`Excess` relatif terhadap `range` menggunakan *pattern matching guard*. |
| `deviation_score` | `fn deviation_score(value: f64, range: &Range) -> f64` | Menghitung besar penyimpangan nilai dari rentang ideal secara deklaratif:<br>$$\text{deviation}(v) = \begin{cases} \text{range.min} - v & v < \text{range.min} \\ v - \text{range.max} & v > \text{range.max} \\ 0.0 & \text{lainnya} \end{cases}$$ |
| `diagnose_parameter` | `fn diagnose_parameter(parameter: &str, value: f64, range: Range) -> ParameterDiagnosis` | Membangun satu `ParameterDiagnosis` dari nilai dan rentang ideal, menggabungkan hasil `classify_parameter` dan `deviation_score`. |
| `diagnose_all_parameters` | `fn diagnose_all_parameters(assessment: &SoilAssessment, profile: &PlantProfile) -> Vec<ParameterDiagnosis>` | Menerapkan `diagnose_parameter` ke keenam parameter (pH, kelembapan, N, P, K, suhu) menggunakan `.map().collect()` atas array pasangan `(nama, nilai, rentang)`, tanpa `for` loop manual. |

### 3.3 Sub-Domain: Analisis Faktor Pembatas & Severity (Tahap 3)

| Nama Fungsi | Definisi Tanda Tangan (Signature) | Deskripsi Algoritma & Karakteristik Fungsional |
| :--- | :--- | :--- |
| `rank_diagnoses_by_severity` | `fn rank_diagnoses_by_severity(diagnoses: &[ParameterDiagnosis]) -> Vec<ParameterDiagnosis>` | Mengurutkan salinan diagnosis dari `deviation` terbesar ke terkecil menggunakan `.to_vec().sort_by(...)`, menjamin slice masukan tidak termutasi (*non-destructive sort*). |
| `find_limiting_factors` | `fn find_limiting_factors(diagnoses: &[ParameterDiagnosis]) -> Vec<String>` | Memfilter parameter yang berstatus bukan `Optimal` melalui `.filter()`, diurutkan via `rank_diagnoses_by_severity`, lalu diformat menjadi teks deskriptif `"parameter: status (nilai=.. ideal=..-..)"`. |

---

## 4. Spesifikasi Fungsi: Mesin Rekomendasi & Skoring (Recommendation & Scoring Engine)

Submodul ini mentranslasikan hasil diagnosis parameter menjadi panduan tindakan preskriptif bertingkat prioritas serta metrik kesesuaian numerik bagi petani maupun sistem rekomendasi tanaman.

### 4.1 Sub-Domain: Generasi Rekomendasi Bertingkat Prioritas

| Nama Fungsi | Definisi Tanda Tangan (Signature) | Deskripsi Algoritma & Formula Agronomi |
| :--- | :--- | :--- |
| `recommend_for_parameter` | `fn recommend_for_parameter(diagnosis: &ParameterDiagnosis) -> Option<Recommendation>` | Pemetaan berbasis aturan `(parameter, status)` → teks tindakan (mis. `ph` + `Deficient` → "Tambahkan kapur pertanian (dolomit) untuk menaikkan pH tanah"). Mengembalikan `None` jika parameter sudah `Optimal`. |
| `severity_to_priority` *(private)* | `fn severity_to_priority(deviation: f64, range: &Range) -> Priority` | *Pure function* yang mengonversi rasio penyimpangan relatif terhadap lebar rentang ideal menjadi tingkat prioritas:<br>$$\text{ratio} = \frac{\text{deviation}}{\text{range.max} - \text{range.min}}$$<br>• $ratio \le 0.15$ → `Low`<br>• $0.15 < ratio \le 0.35$ → `Medium`<br>• $0.35 < ratio \le 0.70$ → `High`<br>• $ratio > 0.70$ → `Critical` |
| `generate_recommendations` | `fn generate_recommendations(diagnoses: &[ParameterDiagnosis]) -> Vec<Recommendation>` | Menghasilkan seluruh rekomendasi via `.iter().filter_map(recommend_for_parameter).collect()`, diurutkan menurun dari prioritas tertinggi ke terendah. |

### 4.2 Sub-Domain: Skoring Kesesuaian & Ringkasan Tekstual

| Nama Fungsi | Definisi Tanda Tangan (Signature) | Deskripsi Algoritma & Formula |
| :--- | :--- | :--- |
| `calculate_suitability_score` | `fn calculate_suitability_score(diagnoses: &[ParameterDiagnosis]) -> f64` | Menghitung persentase parameter berstatus `Optimal` terhadap total parameter menggunakan kombinasi `.iter().filter().count()`:<br>$$\text{score} = \frac{|\{d : d.status = \text{Optimal}\}|}{|diagnoses|} \times 100.0$$<br>Mengembalikan `0.0` apabila `diagnoses` kosong, menghindari pembagian dengan nol. |
| `generate_summary` | `fn generate_summary(plant_name: &str, limiting_factors: &[String], suitability_score: f64) -> String` | Menyusun ringkasan tekstual deklaratif dari nama tanaman, daftar faktor pembatas, dan skor kesesuaian akhir. *Pure function*, tanpa I/O. |

### 4.3 Sub-Domain: Perangkingan Multi-Tanaman

| Nama Fungsi | Definisi Tanda Tangan (Signature) | Deskripsi Algoritma & Karakteristik Fungsional |
| :--- | :--- | :--- |
| `rank_best_plants` | `fn rank_best_plants(assessment: &SoilAssessment, rule_base: &[PlantProfile]) -> Vec<(String, f64)>` | Mengevaluasi satu `SoilAssessment` terhadap **seluruh** entri `rule_base` melalui `.iter().map(...)`, menghitung `suitability_score` masing-masing tanaman, lalu mengurutkan menurun via `.sort_by(...)` untuk merekomendasikan tanaman paling cocok pada kondisi tanah tersebut. |

---

## 5. Integrasi Alur Monadik (Functional Pipeline Orchestration)

Seluruh fungsi diagnosis dan penentuan rekomendasi dirangkai ke dalam satu fungsi orkestrasi fungsional murni sebagai entry point modul:

```rust
pub fn evaluate_plant_suitability(
    plant_name: &str,
    assessment: &SoilAssessment,
    rule_base: &[PlantProfile],
) -> Result<PlantExpertReport, ExpertError>
```

### 5.1 Diagram Alir Transformasi Data Fungsional

```text
          (plant_name, &SoilAssessment, &[PlantProfile])
                                │
                                ▼
    [Tahap 1a: rule_base.is_empty()?] ──── (Ya) ────► Err(ExpertError::EmptyRuleBase)
                                │
                                ▼ (Tidak)
    [Tahap 1b: validate_soil_assessment] ── (Gagal) ──► Err(ExpertError::InvalidAssessment)
                                │
                                ▼ (Lolos)
    [Tahap 1c: find_plant_profile] ──────── (Tidak Ada) ──► Err(ExpertError::UnknownPlant)
                                │
                                ▼ (Ditemukan)
    [Tahap 2: diagnose_all_parameters] ──► Vec<ParameterDiagnosis>
                                │
                                ▼
    [Tahap 3: find_limiting_factors]  ──► Vec<String>
                                │
                                ▼
    [Tahap 4: generate_recommendations & calculate_suitability_score]
                                │
                                ▼
    [Tahap 5: generate_summary & perakitan struktur]
                                │
                                ▼
                 PlantExpertReport (Immutable Output)
```

Alur eksekusi ini menjamin bahwa seluruh transformasi data berlangsung linier, tanpa mutasi *shared memory* maupun state global, dan aman dari interupsi kegagalan berkat penanganan error monadik (`?` operator dengan `Result<T, ExpertError>`).

---

## 6. Spesifikasi Antarmuka Publik & Protokol Komunikasi Sistem

### 6.1 Antarmuka Crate Eksternal (`pub fn`)

Modul 5 mengekspos fungsi publik utama yang dapat dikonsumsi langsung oleh unit `smart_soil_orchestrator` atau diuji secara independen:

| Fungsi Publik (`pub fn`) | Provider | Konsumen Potensial | Masukan (Input) | Keluaran (Output) | Peran Komunikasi |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `evaluate_plant_suitability` | Module 5 | `smart_soil_orchestrator` | `&str`, `&SoilAssessment`, `&[PlantProfile]` | `Result<PlantExpertReport, ExpertError>` | **Entry Point Utama:** Menghasilkan laporan diagnosis dan preskripsi kesesuaian tanaman lengkap. |
| `rank_best_plants` | Module 5 | Orchestrator / Aplikasi Rekomendasi Tanaman | `&SoilAssessment`, `&[PlantProfile]` | `Vec<(String, f64)>` terurut menurun | Merekomendasikan tanaman paling cocok untuk satu kondisi tanah aktual. |
| `default_rule_base` | Module 5 | `smart_soil_orchestrator` / Pengujian | *(tanpa parameter)* | `Vec<PlantProfile>` | Menyediakan rule base awal siap pakai tanpa memerlukan sumber data eksternal. |
| `default_rule_base` (hasil), `diagnose_all_parameters` | Module 4 (sumber `SoilAssessment`) → Module 5 | Module 3 (Visualisasi) | `&SoilAssessment`, `&PlantProfile` | `Vec<ParameterDiagnosis>` | Menyediakan rincian diagnosis per-parameter untuk ditampilkan pada dashboard. |

### 6.2 Contoh Representasi JSON (Lapisan Integrasi Orchestrator)

Inti komputasi Module 5 sengaja dijaga bebas dependensi eksternal (`Cargo.toml` tanpa `[dependencies]`) agar tetap murni dan mudah diuji. Serialisasi ke JSON — apabila diperlukan lintas proses (mis. API/web) — dilakukan di lapisan `smart_soil_orchestrator` menggunakan tipe adapter, bukan di dalam crate ini. Representasi konseptualnya:

```json
{
  "plant_name": "Padi",
  "diagnoses": [
    { "parameter": "ph", "current_value": 4.0, "ideal_range": { "min": 5.5, "max": 7.0 }, "status": "Deficient", "deviation": 1.5 },
    { "parameter": "moisture_percent", "current_value": 30.0, "ideal_range": { "min": 70.0, "max": 90.0 }, "status": "Deficient", "deviation": 40.0 }
  ],
  "limiting_factors": [
    "moisture_percent: Deficient (nilai=30.0 ideal=70.0-90.0)",
    "ph: Deficient (nilai=4.0 ideal=5.5-7.0)"
  ],
  "recommendations": [
    { "parameter": "moisture_percent", "action": "Tingkatkan frekuensi irigasi", "priority": "Critical" },
    { "parameter": "ph", "action": "Tambahkan kapur pertanian (dolomit) untuk menaikkan pH tanah", "priority": "High" }
  ],
  "suitability_score": 66.7,
  "summary": "Kondisi tanah kurang sesuai untuk Padi dengan skor kesesuaian 66.7%. Faktor pembatas utama: kelembapan dan pH tanah."
}
```

---

## 7. Rencana Pengujian Mutu & Skenario Lapangan

Pengujian dilakukan secara komprehensif tanpa bergantung pada hardware sensor fisik atau modul lain melalui *unit tests* berbasis data tiruan (*fixture*) langsung pada `mod tests` di `src/lib.rs`, serta *integration tests* pada direktori `tests/`.

1. **Skenario Tanah Ideal untuk Padi:**
   * Masukan `SoilAssessment { ph: 6.2, moisture_percent: 80.0, nitrogen_ppm: 30.0, phosphorus_ppm: 20.0, potassium_ppm: 20.0, temperature_celsius: 27.0 }` terhadap profil Padi.
   * Sistem harus menghasilkan `diagnoses` yang seluruhnya `Optimal`, `limiting_factors` kosong, `recommendations` kosong, dan `suitability_score = 100.0`.
2. **Skenario Tanah Kurang Lembap & pH Rendah untuk Padi:**
   * Masukan `ph: 4.0, moisture_percent: 30.0` (parameter lain ideal) terhadap profil Padi.
   * Sistem harus menghasilkan dua `limiting_factors` (kelembapan dan pH), `recommendations` dengan prioritas bervariasi hingga `Critical`, dan `suitability_score = 66.7` (4 dari 6 parameter optimal).
3. **Skenario Perangkingan Tanaman Terbaik:**
   * `SoilAssessment` ideal untuk Padi dibandingkan terhadap seluruh entri `default_rule_base()` melalui `rank_best_plants`.
   * Padi harus berada pada urutan pertama karena skor kesesuaiannya tertinggi (100.0).
4. **Skenario Validasi Batas Galat (*Edge Cases*):**
   * Masukan `rule_base: &[]` harus mengembalikan `Err(ExpertError::EmptyRuleBase)`.
   * Masukan `plant_name: "Durian"` (tidak ada di rule base) harus mengembalikan `Err(ExpertError::UnknownPlant("Durian".to_string()))`.
   * Masukan `plant_name: "PADI"` (beda kapitalisasi) tetap ditemukan (`find_plant_profile` bersifat *case-insensitive*).
   * Masukan `moisture_percent: 150.0` (di luar rentang fisik 0–100%) harus ditolak oleh `validate_soil_assessment` dengan `Err(ExpertError::InvalidAssessment(..))`.
   * Pemanggilan `calculate_suitability_score(&[])` secara langsung harus mengembalikan `0.0` tanpa panic.

---

## 8. Public Interface Modul 5 (Ringkasan Komunikasi Antar Modul)

Sebagai ringkasan cepat — mengikuti format tabel "Public Interface" pada **Planning Modul 3** — berikut peta komunikasi seluruh fungsi publik Module 5:

| Function | Komunikasi | Keterangan |
|---|---|---|
| `evaluate_plant_suitability()` | Modul 4 (via adapter Orchestrator) → Modul 5 | Entry point utama: evaluasi kesesuaian tanah-tanaman lengkap |
| `rank_best_plants()` | Modul 5 → Orchestrator/Dashboard | Merekomendasikan tanaman paling cocok untuk satu kondisi tanah |
| `default_rule_base()` | Modul 5 → Orchestrator/Pengujian | Menyediakan rule base bawaan tanpa sumber data eksternal |
| `find_plant_profile()` | Internal Modul 5 (dipakai `evaluate_plant_suitability`) | Pencarian profil tanaman case-insensitive |
| `validate_soil_assessment()` | Internal Modul 5 (dipakai `evaluate_plant_suitability`) | Validasi rentang fisik data tanah sebelum diproses |
| `diagnose_all_parameters()` | Modul 5 → Modul 3 (Visualisasi) | Rincian diagnosis per-parameter untuk ditampilkan pada dashboard |
| `find_limiting_factors()` | Modul 5 → Modul 3 (Visualisasi) / Aplikasi Petani | Daftar faktor pembatas kesesuaian tanaman |
| `generate_recommendations()` | Modul 5 → Aplikasi Mobile Petani | Daftar tindakan bertingkat prioritas |
| `calculate_suitability_score()` | Modul 5 → Modul 3 (Visualisasi) | Skor kesesuaian numerik (0–100) untuk kartu ringkasan dashboard |
| `generate_summary()` | Modul 5 → Modul 3 / Orchestrator | Ringkasan tekstual siap tampil |

Alurnya (gaya Planning Modul 3):

```text
Modul 4
   ↓
Adapter Orchestrator (SoilReading → SoilAssessment)
   ↓
Modul 5
   ↓
PlantExpertReport
   ↓
Modul 3 (Dashboard) / Orchestrator / Modul 6
```

Catatan: Modul 5 hanya menyediakan evaluasi kesesuaian tanaman berbasis rule base. Analisis tren, SQI, dan keputusan platform tetap menjadi tanggung jawab Modul 4; penyimpanan dan rendering visual tetap menjadi tanggung jawab Modul 3.
