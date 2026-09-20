# Spesifikasi Desain & Perencanaan Teknis: Module 4
## Soil Analytics & Decision Support System
**Proyek:** Smart Soil IoT  
**Target Crate:** `module_4_soil_analytics_and_decision_support`  
**Mata Kuliah:** Pemrograman Fungsional  
**Bahasa Pemrograman:** Rust (Edition 2021)  

---

## 1. Pendahuluan & Ruang Lingkup Sistem

Module 4 (**Soil Analytics & Decision Support**) bertindak sebagai pusat komputasi analitik dan mesin inferensi preskriptif di dalam ekosistem **Smart Soil IoT**. Modul ini menjembatani data telemetri fisik tanah yang dihimpun oleh perangkat IoT dengan kebutuhan agrikultur nyata di lapangan, mengubah deret angka mentah menjadi evaluasi mutu kesuburan serta rekomendasi tindakan terukur (irigasi, pemupukan hara makro, dan perbaikan kimiawi tanah).

Seluruh arsitektur modul dirancang secara **fungsional murni (*pure functional core*)**, di mana seluruh logika perhitungan deterministik, bebas efek samping (*side-effect free*), dan menjamin integritas data melalui struktur data yang tidak dapat diubah (*immutable*).

### 1.1 Matriks Batas Tanggung Jawab Modul

Untuk menjaga kohesi sistem yang tinggi dan ketergantungan antar-komponen yang longgar (*loose coupling*), batas ruang lingkup Module 4 didefinisikan secara tegas:

| Bidang Tanggung Jawab | Di Dalam Lingkup Module 4 (In Scope) | Di Luar Lingkup Module 4 (Out of Scope) |
| :--- | :--- | :--- |
| **Akuisisi Data** | Menerima data telemetri yang telah tervalidasi dalam bentuk struktur data memori. | Pembacaan sinyal ADC sensor, interfacing kabel fisik, dan kalibrasi tegangan listrik (*Tanggung jawab Module 1*). |
| **Komunikasi Jaringan** | Menyediakan interface serialisasi/deserialisasi payload JSON standar. | Manajemen koneksi MQTT, HTTP client/server, otentikasi pengguna, dan edge caching (*Tanggung jawab Module 2*). |
| **Penyimpanan & UI** | Menghasilkan struktur laporan analitik dan preskripsi terstruktur (`RecommendationReport`). | Penyimpanan persisten basis data time-series dan rendering grafik visual dashboard (*Tanggung jawab Module 3*). |
| **Agronomi Tanaman** | Menggunakan parameter batas agronomi sebagai acuan komputasi. | Penelitian biologi tanaman dan pemeliharaan database varietas bibit (*Tanggung jawab Module 5*). |
| **Aktuasi Lapangan** | Menghitung volume air ($L/m^2$), durasi menit penyiraman, dan gram pupuk. | Pensaklaran relay fisik, buka-tutup katup solenoid pompa (*Tanggung jawab Orchestrator / Aktuator*). |

---

## 2. Pemodelan Domain Berbasis Algebraic Data Types (ADTs)

Modul ini memanfaatkan sistem tipe data aljabar Rust untuk merepresentasikan status tanah, kategori kualitas, dan tindakan keputusan secara presisi tanpa celah keadaan yang tidak valid (*invalid state unrepresentable*).

### 2.1 Model Telemetri dan Batas Agronomi Tanaman

```rust
use serde::{Deserialize, Serialize};

/// Klasifikasi fisik tekstur tanah yang memengaruhi retensi air dan kapur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoilTexture {
    Sandy,
    Loamy,
    Clayey,
}

/// Snapshot telemetri kondisi tanah pada satu titik waktu tertentu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoilReading {
    pub timestamp: u64,
    pub moisture_percentage: f64,     // 0.0 - 100.0%
    pub ph_level: f64,                // 0.0 - 14.0
    pub temperature_celsius: f64,     // °C
    pub nitrogen_ppm: f64,            // mg/kg
    pub phosphorus_ppm: f64,          // mg/kg
    pub potassium_ppm: f64,           // mg/kg
    pub electrical_conductivity: f64, // dS/m (Salinitas)
}

/// Konfigurasi kebutuhan optimal tanaman yang menjadi acuan diagnosis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoilThresholds {
    pub min_moisture: f64,
    pub target_moisture: f64,             // Kapasitas Lapang
    pub permanent_wilting_point: f64,     // Titik Layu Permanen
    pub min_ph: f64,
    pub max_ph: f64,
    pub target_nitrogen: f64,
    pub target_phosphorus: f64,
    pub target_potassium: f64,
    pub max_safe_ec: f64,
    pub soil_texture: SoilTexture,
}
```

### 2.2 Model Diagnosis Mutu Tanah dan Tren

```rust
/// Klasifikasi mutu kesuburan tanah berdasarkan skor komposit SQI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SoilGrade {
    Excellent,
    Good,
    Moderate,
    Degraded,
    Critical,
}

/// Dinamika perubahan kadar air tanah terhadap waktu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    RapidlyDecreasing,
    Decreasing,
    Stable,
    Increasing,
    RapidlyIncreasing,
}

/// Indeks Kualitas Tanah (Soil Quality Index - SQI) terbobot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoilQualityIndex {
    pub overall_score: f64,
    pub grade: SoilGrade,
    pub sub_scores: QualitySubScores,
}

/// Rincian skor individual multi-parameter kualitas tanah.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualitySubScores {
    pub moisture_score: f64,
    pub ph_score: f64,
    pub nutrient_score: f64,
    pub salinity_score: f64,
}

/// Metrik agregasi statistik deret waktu sensor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoilStatistics {
    pub mean_moisture: f64,
    pub variance_moisture: f64,
    pub std_dev_moisture: f64,
    pub mean_ph: f64,
    pub mean_temperature: f64,
    pub moisture_depletion_rate_per_hour: Option<f64>,
    pub moisture_trend: TrendDirection,
    pub estimated_hours_to_wilting: Option<f64>,
}
```

### 2.3 Model Preskripsi Keputusan dan Tindakan Lapangan

```rust
/// Tingkat urgensi eksekusi tindakan agronomi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UrgencyLevel {
    Normal,
    Advisory,
    Urgent,
    Critical,
}

/// Rekomendasi volume air dan durasi sistem penyiraman.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrrigationAction {
    pub water_volume_liters_per_m2: f64,
    pub recommended_duration_minutes: u32,
    pub moisture_deficit_percentage: f64,
    pub reason: String,
}

/// Defisit unsur hara makro tanah terhadap ambang kebutuhan tanaman.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NutrientDeficit {
    pub nitrogen_deficit_ppm: f64,
    pub phosphorus_deficit_ppm: f64,
    pub potassium_deficit_ppm: f64,
}

/// Rekomendasi takaran pupuk komersial spesifik.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FertilizerAction {
    pub urea_grams_per_m2: f64,
    pub sp36_grams_per_m2: f64,
    pub kcl_grams_per_m2: f64,
    pub alternative_npk_15_15_15_grams_per_m2: Option<f64>,
    pub application_notes: String,
}

/// Rekomendasi tindakan perbaikan karakteristik tanah.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AmendmentAction {
    ApplyAgriculturalLime {
        grams_per_m2: f64,
        target_ph: f64,
        rationale: String,
    },
    ApplyElementalSulfur {
        grams_per_m2: f64,
        target_ph: f64,
        rationale: String,
    },
    FlushSaltsWithLeaching {
        excess_ec: f64,
        recommended_flush_volume_liters_per_m2: f64,
        rationale: String,
    },
}

/// Laporan rekomendasi akhir terintegrasi hasil pemrosesan Module 4.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationReport {
    pub generated_at_timestamp: u64,
    pub soil_quality_index: SoilQualityIndex,
    pub statistics: SoilStatistics,
    pub anomalies: AnomalyAlerts,
    pub irrigation_action: Option<IrrigationAction>,
    pub fertilizer_action: Option<FertilizerAction>,
    pub soil_amendment_action: Option<AmendmentAction>,
    pub overall_urgency: UrgencyLevel,
}
```

### 2.4 Penanganan Galat Fungsional (Monadic Error Type)

Tidak ada penggunaan runtime `panic!` atau pengecualian tidak tertangani. Kesalahan didefinisikan secara deklaratif:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SoilError {
    EmptyDataset { operation: &'static str },
    InsufficientData { required: usize, found: usize },
    InvalidPhysicalRange { parameter: &'static str, value: f64, valid_min: f64, valid_max: f64 },
    InvalidWeightConfiguration { total_weight: f64 },
    ZeroDivisionRisk { context: &'static str },
}
```

---

## 3. Spesifikasi Fungsi: Mesin Analitik Murni (Pure Analytics Engine)

Seluruh fungsi analitik dirancang tanpa mutasi keadaan luar, deterministik, dan memanfaatkan fitur ekspresif Rust seperti *closure*, *iterator combinators*, dan fungsi tingkat tinggi (*Higher-Order Functions*).

### 3.1 Sub-Domain: Statistik Deskriptif & Pemulusan Deret Waktu

| Nama Fungsi | Definisi Tanda Tangan (Signature) | Deskripsi Algoritma & Karakteristik Fungsional |
| :--- | :--- | :--- |
| `mean` | `fn mean(values: &[f64]) -> Option<f64>` | Menghitung rata-rata aritmetika menggunakan `Iterator::fold`. Mengembalikan `None` apabila deret masukan kosong tanpa menimbulkan pembagian dengan nol. |
| `variance` | `fn variance(values: &[f64]) -> Option<f64>` | Menghitung varians sampel secara murni dengan memetakan kuadrat deviasi tiap elemen terhadap rata-rata menggunakan kombinasi `.map()` dan `.fold()`. |
| `standard_deviation` | `fn standard_deviation(values: &[f64]) -> Option<f64>` | Menghitung standar deviasi melalui komposisi monadik `variance(values).map(f64::sqrt)`. |
| `simple_moving_average` | `fn simple_moving_average(window_size: usize) -> impl Fn(&[f64]) -> Vec<f64>` | **Higher-Order Function (HOF)** yang menerima ukuran jendela pengamatan dan mengembalikan closure pemulus deret waktu menggunakan `.windows().filter_map(mean)`. |
| `exponential_moving_average` | `fn exponential_moving_average(alpha: f64, values: &[f64]) -> Vec<f64>` | Menerapkan pemulusan eksponensial ($EMA$) secara fungsional melalui penumpukan state perataan dalam akumulator `Iterator::fold`. |

### 3.2 Sub-Domain: Penilaian Kualitas Tanah (Soil Quality Index - SQI)

Indeks mutu tanah dihitung dengan formula pembobotan multi-kriteria:
$$SQI = (W_m \cdot S_m) + (W_{ph} \cdot S_{ph}) + (W_n \cdot S_n) + (W_{ec} \cdot S_{ec})$$
dengan batas normalisasi $\sum W = 1.0$ dan skor individual ternormalisasi pada rentang $[0.0, 100.0]$.

| Nama Fungsi | Definisi Tanda Tangan (Signature) | Deskripsi Algoritma & Karakteristik Fungsional |
| :--- | :--- | :--- |
| `calculate_moisture_subscore` | `fn calculate_moisture_subscore(moisture: f64, thresholds: &SoilThresholds) -> f64` | Menghitung skor kecukupan hidrologis tanah berdasarkan posisi kelembapan aktual di antara titik layu permanen ($PWP$) dan target kapasitas lapang. |
| `calculate_ph_subscore` | `fn calculate_ph_subscore(ph: f64, thresholds: &SoilThresholds) -> f64` | Menghitung skor kimiawi keasaman tanah berbasis kurva linear deviasi terhadap batas toleransi optimal tanaman. |
| `calculate_nutrient_subscore` | `fn calculate_nutrient_subscore(reading: &SoilReading, thresholds: &SoilThresholds) -> f64` | Menghitung rasio pemenuhan hara makro (N, P, K) menggunakan reduksi iterator fungsional dan memberikan penalti pada pemupukan berlebih. |
| `calculate_salinity_subscore` | `fn calculate_salinity_subscore(ec: f64, max_safe_ec: f64) -> f64` | Menghitung skor toleransi salinitas berdasarkan ambang aman konduktivitas listrik ($EC$). |
| `classify_soil_grade` | `fn classify_soil_grade(score: f64) -> SoilGrade` | Mengklasifikasikan skor numerik ke dalam varian enum `SoilGrade` menggunakan *pattern matching guard*. |
| `calculate_sqi` | `fn calculate_sqi(reading: &SoilReading, thresholds: &SoilThresholds, weights: &SqiWeights) -> Result<SoilQualityIndex, SoilError>` | Mengagregasikan seluruh sub-skor secara terbobot dan mengembalikan evaluasi mutu kesuburan tanah lengkap. |

### 3.3 Sub-Domain: Tren Temporal & Prediksi Kekeringan

| Nama Fungsi | Definisi Tanda Tangan (Signature) | Deskripsi Algoritma & Karakteristik Fungsional |
| :--- | :--- | :--- |
| `compute_moisture_depletion_rate` | `fn compute_moisture_depletion_rate(readings: &[SoilReading]) -> Option<f64>` | Menghitung laju pengeringan tanah (% penurunan kelembapan per jam) melalui formula regresi linear satu dimensi yang dihitung secara fungsional via `.fold()`. |
| `detect_trend_direction` | `fn detect_trend_direction(depletion_rate_per_hour: f64) -> TrendDirection` | Mengkategorikan gradien penurunan kelembapan ke dalam enum arah pergerakan tren secara deklaratif. |
| `estimate_hours_to_wilting` | `fn estimate_hours_to_wilting(current_moisture: f64, permanent_wilting_point: f64, depletion_rate_per_hour: f64) -> Option<f64>` | Memprediksi sisa waktu sebelum tanaman menyentuh batas kritis kelayuan permanen jika pola pengeringan berlanjut. Mengembalikan `None` jika tanah tidak sedang mengering. |

### 3.4 Sub-Domain: Deteksi Anomali & Risiko Agroekologi

| Nama Fungsi | Definisi Tanda Tangan (Signature) | Deskripsi Algoritma & Karakteristik Fungsional |
| :--- | :--- | :--- |
| `detect_waterlogging_risk` | `fn detect_waterlogging_risk(readings: &[SoilReading], saturation_threshold: f64, consecutive_readings: usize) -> bool` | Mendeteksi kondisi jenuh air berkelanjutan yang berisiko membusukkan akar menggunakan iterator jendela geser `.windows().any(...)`. |
| `detect_acid_shock` | `fn detect_acid_shock(readings: &[SoilReading], max_drop_delta: f64) -> Option<f64>` | Mendeteksi penurunan nilai pH tanah secara drastis dalam jendela waktu pengamatan singkat. |
| `detect_salinity_stress` | `fn detect_salinity_stress(reading: &SoilReading, max_safe_ec: f64) -> bool` | Memeriksa apakah konduktivitas listrik melampaui batas toleransi stres osmotik akar. |
| `evaluate_all_anomalies` | `fn evaluate_all_anomalies(readings: &[SoilReading], thresholds: &SoilThresholds) -> AnomalyAlerts` | Mengombinasikan seluruh evaluasi risiko menjadi struktur data peringatan anomali yang memuat pesan deskriptif. |

---

## 4. Spesifikasi Fungsi: Mesin Pendukung Keputusan (Decision Support System)

Submodul ini mentranslasikan metrik analitik tanah menjadi panduan tindakan preskriptif kuantitatif bagi petani dan aktuator irigasi otomatis.

### 4.1 Sub-Domain: Preskripsi Irigasi Presisi

| Nama Fungsi | Definisi Tanda Tangan (Signature) | Deskripsi Algoritma & Formula Agronomi |
| :--- | :--- | :--- |
| `calculate_water_deficit` | `fn calculate_water_deficit(current_moisture: f64, target_moisture: f64, root_depth_cm: f64) -> f64` | Menghitung kekurangan volume air volumetrik menuju kapasitas lapang dalam satuan liter per meter persegi ($L/m^2$):<br>$$\text{Defisit Air } (L/m^2) = \frac{\theta_{\text{target}} - \theta_{\text{aktual}}}{100} \times (\text{Kedalaman Perakaran mm})$$ |
| `recommend_irrigation` | `fn recommend_irrigation(reading: &SoilReading, thresholds: &SoilThresholds, emitter_flow_rate_lph_per_m2: f64) -> Option<IrrigationAction>` | Menghasilkan rekomendasi irigasi lengkap beserta estimasi durasi menit operasional emitter penyiraman jika kadar air berada di bawah batas minimum aman. |

### 4.2 Sub-Domain: Preskripsi Pemupukan Hara Makro (N, P, K)

| Nama Fungsi | Definisi Tanda Tangan (Signature) | Deskripsi Algoritma & Formula Agronomi |
| :--- | :--- | :--- |
| `calculate_nutrient_deficits` | `fn calculate_nutrient_deficits(reading: &SoilReading, targets: &SoilThresholds) -> NutrientDeficit` | Menghitung selisih defisit elemen murni Nitrogen, Fosfor, dan Kalium terhadap target tanaman secara independen. |
| `recommend_fertilizers` | `fn recommend_fertilizers(deficit: &NutrientDeficit) -> Option<FertilizerAction>` | Mengonversi defisit hara murni (ppm) ke takaran pupuk komersial riil ($g/m^2$) berdasarkan kadar aktif hara:<br>• **Urea:** $46\% \text{ N}$<br>• **SP-36:** $36\% \text{ P}_2\text{O}_5$<br>• **KCl:** $60\% \text{ K}_2\text{O}$<br>serta estimasi opsi pupuk majemuk NPK 15-15-15. |

### 4.3 Sub-Domain: Preskripsi Pembenah Tanah (Soil Amendment)

| Nama Fungsi | Definisi Tanda Tangan (Signature) | Deskripsi Algoritma & Formula Agronomi |
| :--- | :--- | :--- |
| `recommend_soil_amendment` | `fn recommend_soil_amendment(reading: &SoilReading, thresholds: &SoilThresholds) -> Option<AmendmentAction>` | Menerapkan aturan ameliorasi berbasis tekstur tanah:<br>• **Tanah Masam ($pH < min$):** Dosis kapur dolomit pertanian ($CaCO_3$) dihitung dengan faktor penyangga tekstur ($150\text{ g/m}^2$ untuk pasir, $250\text{ g/m}^2$ untuk lempung, $350\text{ g/m}^2$ untuk liat per $1.0\text{ kenaikan pH}$).<br>• **Tanah Basa ($pH > max$):** Dosis bubuk belerang murni ($S$) untuk menormalkan pH.<br>• **Tanah Salin:** Rekomendasi volume air pencucian garam (*leaching*). |
| `evaluate_overall_urgency` | `fn evaluate_overall_urgency(sqi: &SoilQualityIndex, anomalies: &AnomalyAlerts, irrigation: Option<&IrrigationAction>, fertilizer: Option<&FertilizerAction>, amendment: Option<&AmendmentAction>) -> UrgencyLevel` | Menentukan prioritas tindakan keseluruhan (`Normal`, `Advisory`, `Urgent`, `Critical`) melalui evaluasi *pattern matching* berhirarki. |

---

## 5. Integrasi Alur Monadik (Functional Pipeline Orchestration)

Seluruh fungsi analitik dan penentuan keputusan dirangkai ke dalam satu fungsi orkestrasi fungsional murni pada berkas `src/pipeline.rs`:

```rust
pub fn process_soil_pipeline(
    readings: &[SoilReading],
    thresholds: &SoilThresholds,
    config: &PipelineConfig,
) -> Result<RecommendationReport, SoilError>
```

### 5.1 Diagram Alir Transformasi Data Fungsional

```text
               Deret Bacaan Sensor: &[SoilReading]
                                │
                                ▼
    [Tahap 1: Validasi Fungsional] ──── (Gagal) ────► Err(SoilError::InvalidPhysicalRange)
                                │
                                ▼ (Lolos)
    [Tahap 2: Agregasi Statistik Deskriptif & HOF SMA]
                                │
                                ▼
    [Tahap 3 & 4: Evaluasi SQI & Estimasi Waktu Titik Layu]
                                │
                                ▼
    [Tahap 5: Pemeriksaan Anomali Agroekologi]
                                │
                                ▼
    [Tahap 6: Eksekusi Mesin Keputusan (Irigasi, Hara, Amelioran)]
                                │
                                ▼
                 RecommendationReport (Immutable Output)
```

Alur eksekusi ini menjamin bahwa seluruh transformasi data berlangsung linier, tanpa mutasi *shared memory*, dan aman dari interupsi kegagalan berkat penanganan error monadik.

---

## 6. Spesifikasi Antarmuka Publik & Protokol Komunikasi Sistem

### 6.1 Antarmuka Crate Eksternal (`pub fn`)

Modul 4 mengekspos fungsi publik utama yang dapat dikonsumsi langsung oleh unit `smart_soil_orchestrator` atau diuji secara independen:

| Fungsi Publik (`pub fn`) | Konsumen Potensial | Masukan (Input) | Keluaran (Output) | Peran Komunikasi |
| :--- | :--- | :--- | :--- | :--- |
| `process_soil_pipeline` | `smart_soil_orchestrator` / Modul 3 | `&[SoilReading]`, `&SoilThresholds`, `&PipelineConfig` | `Result<RecommendationReport, SoilError>` | **Entry Point Utama:** Menghasilkan laporan analitik dan preskripsi terpadu. |
| `calculate_sqi` | Module 3 (Visualisasi) | `&SoilReading`, `&SoilThresholds`, `&SqiWeights` | `Result<SoilQualityIndex, SoilError>` | Menyediakan metrik kesuburan tanah untuk grafik dashboard. |
| `recommend_irrigation` | Unit Aktuator Pompa | `&SoilReading`, `&SoilThresholds`, `f64` (debit) | `Option<IrrigationAction>` | Menyediakan perintah preskriptif volume air irigasi. |
| `recommend_fertilizers` | Aplikasi Mobile Petani | `&NutrientDeficit` | `Option<FertilizerAction>` | Menyediakan takaran dosis pupuk NPK. |
| `recommend_soil_amendment` | Aplikasi Mobile Petani | `&SoilReading`, `&SoilThresholds` | `Option<AmendmentAction>` | Menyediakan panduan pemberian kapur dolomit / belerang. |

### 6.2 Contoh Format Pertukaran Data Serialisasi JSON

Hasil pemrosesan Module 4 secara otomatis dapat diserialisasikan ke format JSON standar untuk menjamin kemudahan integrasi dengan modul backend web maupun sistem broker IoT:

```json
{
  "generated_at_timestamp": 1710010800,
  "soil_quality_index": {
    "overall_score": 78.45,
    "grade": "Good",
    "sub_scores": {
      "moisture_score": 68.2,
      "ph_score": 82.5,
      "nutrient_score": 81.0,
      "salinity_score": 92.0
    }
  },
  "statistics": {
    "mean_moisture": 41.75,
    "variance_moisture": 14.25,
    "std_dev_moisture": 3.77,
    "mean_ph": 5.35,
    "mean_temperature": 27.12,
    "moisture_depletion_rate_per_hour": 3.0,
    "moisture_trend": "RapidlyDecreasing",
    "estimated_hours_to_wilting": 5.33
  },
  "irrigation_action": {
    "water_volume_liters_per_m2": 80.0,
    "recommended_duration_minutes": 192,
    "moisture_deficit_percentage": 32.0,
    "reason": "Kelembapan tanah (38.0%) berada di bawah batas minimum (45.0%). Defisit air sebesar 80.0 L/m²."
  },
  "fertilizer_action": {
    "urea_grams_per_m2": 19.56,
    "sp36_grams_per_m2": 14.58,
    "kcl_grams_per_m2": 17.50,
    "alternative_npk_15_15_15_grams_per_m2": 60.0
  },
  "soil_amendment_action": {
    "ApplyAgriculturalLime": {
      "grams_per_m2": 275.0,
      "target_ph": 6.4,
      "rationale": "pH tanah (5.3) terlalu masam. Berikan kapur dolomit sebesar 275.0 g/m²."
    }
  },
  "overall_urgency": "Urgent"
}
```

---

## 7. Rencana Pengujian Mutu & Skenario Lapangan

Pengujian dilakukan secara komprehensif tanpa bergantung pada hardware sensor fisik melalui *unit tests* dan *integration tests* berbasis data tiruan (*fixture*) pada direktori `tests/`:

1. **Skenario Lahan Kekeringan Ekstrem:**
   * Masukan kelembapan 28.0% (ambang minimum 45.0%, target 70.0%).
   * Sistem harus mengaktifkan `IrrigationAction`, menghitung defisit volume air, dan durasi operasional secara presisi.
2. **Skenario Koreksi Tanah Masam (Acidic Liming):**
   * Masukan pH tanah 4.8 pada lahan bertekstur lempung (*Loamy*).
   * Sistem harus menghasilkan `AmendmentAction::ApplyAgriculturalLime` dengan perhitungan dosis dolomit yang tepat sesuai faktor penyangga tekstur.
3. **Skenario Anomali Genangan Air & Toksisitas Garam:**
   * Deret kelembapan $\ge 90.0\%$ secara kontinu selama lebih dari 3 sampel pengamatan dengan nilai $EC \ge 3.0\text{ dS/m}$.
   * Sistem harus menyalakan peringatan waterlogging dan salinitas, menetapkan `UrgencyLevel::Critical`, menonaktifkan aksi irigasi, dan merekomendasikan tindakan pencucian garam (*leaching*).
4. **Skenario Validasi Batas Galat (*Edge Cases*):**
   * Masukan dataset kosong (`&[]`) harus mengembalikan `Err(SoilError::EmptyDataset)`.
   * Masukan pembacaan anomali (misal pH bernilai 16.0 atau kelembapan negatif) harus ditolak saat proses validasi awal.

---

## 8. Pembagian Tanggung Jawab & Tata Kelola Tim

Pengerjaan proyek dibagi secara modular ke dalam empat peran spesialisasi untuk menjamin kontribusi aktif seluruh anggota tim pada repositori Git:

| Peran Anggota | Fokus Sub-Domain Kode | Berkas Tanggung Jawab |
| :--- | :--- | :--- |
| **Anggota 1 (Project Lead)** | Arsitektur Modul, Pipeline Monadik & Integrasi Sistem Eksternal | `src/lib.rs`, `src/pipeline.rs`, `src/error.rs`, `examples/demo.rs` |
| **Anggota 2** | Komputasi Matematika, Statistik Deskriptif Murni, HOF SMA & Evaluasi Tren | `src/analytics/statistics.rs`, `src/analytics/trends.rs`, `src/analytics/sqi.rs` |
| **Anggota 3** | Mesin Keputusan Preskriptif (Irigasi, Hara Makro NPK & Ameliorasi Kimiawi) | `src/decision/irrigation.rs`, `src/decision/fertilizer.rs`, `src/decision/amendment.rs`, `src/decision/mod.rs` |
| **Anggota 4** | Pemodelan Data Domain, Aturan Anomali Agroekologi & Skenario Uji Terintegrasi | `src/models/reading.rs`, `src/models/analytics.rs`, `src/analytics/anomaly.rs`, `tests/*` |

### Standar Tata Kelola Kode:
* Setiap anggota bekerja pada cabang fitur masing-masing (`feature/pipeline-core`, `feature/analytics-math`, `feature/decision-support`, `feature/domain-models`).
* Integrasi ke cabang utama (`main`) wajib melalui mekanisme Pull Request dengan telaah kode (*code review*).
* Seluruh fungsi publik wajib lolos pengujian dokumentasi dengan perintah `cargo test --doc` dan pengujian unit dengan `cargo test`.
