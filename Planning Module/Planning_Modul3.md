# Planning Function - Modul 3 Soil Data Platform & Visualization

## 1. Tujuan Modul

Membangun platform data tanah yang mengubah data sensor IoT menjadi data yang tersimpan, dapat dicari, dapat diolah berdasarkan waktu, dan dapat ditampilkan kepada pengguna.

Adapun tujuan Modul 3 adalah membuat platform yang dapat:

1. Menerima data tanah dari Modul 2.
2. Menyimpan data tanah ke database.
3. Mengambil data tanah terbaru.
4. Mengambil data historis berdasarkan rentang waktu.
5. Melakukan pengolahan dan agregasi data time series.
6. Menyediakan data yang diperlukan untuk dashboard visualisasi.
7. Menyediakan riwayat warning atau alert.
8. Melakukan filtering data sesuai kebutuhan.
9. Membandingkan kondisi beberapa plot.
10. Menyediakan informasi status perangkat dan kondisi sensor.
11. Menyediakan informasi metadata plot.
12. Menyediakan data yang dapat digunakan oleh Modul 4 untuk analisis.

## 2. Batasan Modul

Modul ini tidak bertanggung jawab terhadap:

1. Membaca sensor fisik secara langsung.
2. Melakukan komunikasi HTTP langsung dengan ESP32 sebagai tanggung jawab utama Modul 2.
3. Melakukan analisis untuk prediksi atau rekomendasi kondisi tanah.
4. Menentukan rekomendasi khusus tanaman.
5. Mengelola plugin.
6. Menentukan aturan agronomi tanaman.

## 3. Integrasi dengan Modul Lain

### Data dari Modul 2

Modul 2 bertanggung jawab terhadap komunikasi IoT dan menghasilkan data sensor yang sudah dapat dikirim ke platform. Data tersebut kemudian diterima oleh Modul 3 untuk disimpan dan diolah.

Struktur data yang digunakan:

```rust
pub struct SoilData {
    pub device_id: String,
    pub plot_id: String,
    pub timestamp: String,
    pub moisture: f64,
    pub temperature: f64,
    pub ph: f64,
    pub ec: f64,
}
```

Keterangan:

- `device_id`: ID perangkat atau sensor.
- `plot_id`: ID lokasi atau plot tempat sensor berada.
- `timestamp`: waktu data diterima atau dicatat.
- `moisture`: nilai kelembapan tanah.
- `temperature`: nilai suhu tanah.
- `ph`: nilai pH tanah.
- `ec`: nilai electrical conductivity tanah.

## 4. Pembagian Function

### 4.1 Menerima dan Menyimpan Data

```rust
pub fn receive_sensor_data(data: SoilData)
```

Fungsi ini digunakan untuk menerima data tanah yang dikirim dari Modul 2.

```rust
fn save_sensor_data(data: SoilData)
```

Fungsi ini digunakan untuk menyimpan data tanah yang telah diterima ke dalam database.

### 4.2 Mengambil Data

```rust
pub fn get_current_soil_data(plot_id: &str)
```

Fungsi ini digunakan untuk mengambil data tanah terbaru dari suatu plot.

```rust
pub fn get_historical_soil_data(
    plot_id: &str,
    start_time: &str,
    end_time: &str
)
```

Fungsi ini digunakan untuk mengambil data tanah berdasarkan rentang waktu tertentu. Data tersebut nantinya dapat digunakan untuk membuat grafik atau melihat perubahan kondisi tanah dari waktu ke waktu.

### 4.3 Pengolahan Data

```rust
fn filter_soil_data(data: &[SoilData]) -> Vec<SoilData>
```

Fungsi ini digunakan untuk menyaring data tanah sesuai dengan kebutuhan pengolahan.

```rust
fn aggregate_soil_data(data: &[SoilData])
```

Fungsi ini digunakan untuk melakukan pengolahan dan agregasi data, seperti menghitung rata-rata:

- moisture
- temperature
- pH
- EC

### 4.4 Dashboard

```rust
pub fn get_dashboard_data(plot_id: &str)
```

Fungsi ini digunakan untuk menyiapkan data yang diperlukan oleh dashboard, seperti data terbaru, data historis, atau hasil pengolahan data tanah.

### 4.5 Warning / Alert

```rust
pub fn get_alert_history(plot_id: &str)
```

Fungsi ini digunakan untuk mengambil riwayat warning atau alert yang berkaitan dengan suatu plot.

### 4.6 Device Status
```rust
pub fn get_device_status(device_id: &str)
```

Fungsi ini digunakan untuk mengambil informasi status perangkat yang digunakan untuk mengirimkan data sensor, seperti status aktif atau tidak aktif.

### 4.7 Sensor Health
```rust
pub fn get_sensor_health(device_id: &str)
```

Fungsi ini digunakan untuk mengetahui kondisi sensor pada suatu perangkat, sehingga dashboard dapat menampilkan informasi apakah sensor berjalan dengan normal atau mengalami masalah.

### 4.8 Plot Metadata
```rust
pub fn get_plot_metadata(plot_id: &str)
```

Fungsi ini digunakan untuk mengambil informasi dasar mengenai suatu plot, seperti identitas plot atau perangkat yang digunakan pada plot tersebut.

### 4.9 Fungsi Tambahan

```rust
pub fn compare_plots(plot_ids: Vec<&str>)
```

Fungsi ini digunakan untuk membandingkan kondisi data tanah dari beberapa plot.

Fungsi ini bersifat opsional dan dapat dikembangkan apabila diperlukan oleh dashboard.

### 4.10 Integrasi dengan Modul 4

Data yang telah disimpan dan diolah oleh Modul 3 dapat digunakan oleh Modul 4 untuk melakukan analisis kondisi tanah.

Data yang dapat digunakan Modul 4 antara lain:

```rust
get_current_soil_data()
get_historical_soil_data()
aggregate_soil_data()
```
Alurnya:
Modul 2
   ↓
Modul 3
   ↓
Data tanah
   ↓
Modul 4
   ↓
Analisis kondisi tanah

Catatan: Modul 3 hanya menyediakan data. Analisis dan interpretasi kondisi tanah menjadi tanggung jawan Modul 4.

## 5. Public Interface Modul 3

| Function | Komunikasi | Keterangan |
|---|---|---|
| `receive_sensor_data()` | Modul 2 → Modul 3 | Menerima data sensor dari Modul 2 |
| `get_current_soil_data()` | Modul lain/Dashboard → Modul 3 | Mengambil data tanah terbaru |
| `get_historical_soil_data()` | Modul lain/Dashboard → Modul 3 | Mengambil data tanah berdasarkan rentang waktu |
| `get_dashboard_data()` | Dashboard → Modul 3 | Menyediakan data yang diperlukan dashboard |
| `get_alert_history()` | Dashboard → Modul 3 | Mengambil riwayat warning atau alert |
| `compare_plots()` | Dashboard → Modul 3 | Membandingkan kondisi beberapa plot, opsional |
| `get_device_status()` | Dashboard → Modul 3 | Mengambil status perangkat |
| `get_sensor_health()` | Dashboard → Modul 3 | Mengambil kondisi sensor |
| `get_plot_metadata()` | Dashboard → Modul 3 | Mengambil informasi dasar plot |
| `get_historical_soil_data()` | Modul 4 → Modul 3 | Menyediakan data historis untuk analisis Modul 4 |