# Planning Function — Modul 6: Plugin Framework and Extensibility

Mata Kuliah: Pemrograman Fungsional  
Bahasa: Rust  
Cakupan:Sistem berfokus pada perubahan fungsionalitas pengolahan data tanpa harus mengubah sistem inti aplikasi secara keseluruhan. Mencakup bagaimana data analitik diproses dan tindakan berdasarkan data yang tersedia.

---

## 1. Tujuan Modul

Modul ini dirancang dengan tujuan utama untuk memperluas ruang lingkup platform Smart Soil tanpa mengubah sistem inti dan menjaga kompatibilitas plugin dengan aplikasi secara keseluruhan. Dengan menggunakan framework ini, fitur-fitur baru dapat ditambahkan ke dalam sistem IoT secara dinamis 

Modul ini bertanggung jawab untuk:

* Menyediakan antarmuka bagi developer untuk meng-extend kapabilitas dari aplikasi Smart Soil
* Memastikan kompatibilitas plugin dengan platform
* Memastikan keamanan, membentuk batasan, dan memberi akses kepada plugin terhadap data dan fungsi-fungsi inti.

### Batasan Modul

Modul ini **tidak bertanggung jawab** terhadap:

* Akuisisi, kalibrasi, dan keamanan pada perangkat keras sensor (tanggung jawab Modul 1).
* Protokol komunikasi pada IoT, pemrosesan edge, dan manajemen akses pada pengguna ( tanggung jawab Modul 2 ).
* Penyimpanan data waktu, penyediaan API REST, antarmuka dashboard ( tanggung jawab Modul 3).
* Mengubah atau memodifikasi kode sumber inti sistem.
* Membuat logika atau algoritma pemrosesan data bawaan.

## 2. Plugin Lifecycle
Merupakan state yang berurutan untuk mengatur perjalanan plugin di dalam sistem Smart Soil ini. Tujuannya dengan membagi status operasi plugin ini sistem akan dapat tetap beroperasi tanpa resiko crash atau resiko pada saat menjalankan program lainnya meskipun ada plugin yang sedang mengalami error. Update, atau tiba tiba di matikan.

#### 2.1. Register
Sistem Smart Soil memvalidasi dan mencatat metadata plugin ke dalam Registry atau katalog internal. Plugin sudah terdaftar namun belum beroperasi.

#### 2.2. Enable 
Sistem kontrol yang mengubah status plugin menjadi “ON” yang berarti plugin diizinkan untuk beroperasi

#### 2.3. Initialize
Plugin mulai menyediakan apa saja sumber daya yang dibutuhkan. Contohnya seperti plugin memuat konfigurasi dasar seperti batas toleransi tanaman, agar siap menerima data sensor tanah.

#### 2.4. Ready
Plugin telah selesai diinisialisasi dan dalam kondisi siap. Plugin dipastikan aman dan kompatibel, tinggal menunggu instruksi dari sistem inti.
 
#### 2.5. Execute 
Sistem inti mengeksekusi logika plugin. Plugin selanjutnya bekerja memproses data sensor yang selanjutnya menghasilkan skor resiko atau rekomendasi.
 
#### 2.6. Shutdown
Sistem memerintahkan plugin untuk menghentikan proses sehingga plugin selanjutnya akan membersihkan alokasi memori dan menyimpan status terakhirnya pada platform Smart Soil.

#### 2.7. Disabled
Plugin sepenuhnya dimatikan. Sistem utama tidak akan lagi mengirim data sensor ke plugin, namun plugin masih tersimpan di dalam memori dan bisa di-enable kan lagi ketika digunakan kembali. 

#### 2.8. Unregister
Sistem secara resmi mencabut nama dan data plugin dari katalog internal sehingga sistem Smart Soil tidak lagi menganggap plugin tersebut terhubung.

#### 2.9. Removed
Seluruh resources dari plugin dihapus dan dibersihkan permanen dari server atau penyimpanan platform.

## 4. Variasi Plugin
Untuk menjaga keamanan tipe dan kejelasan terhadap batas operasional, Plugin Framework menyediakan antarmuka yang berbeda sesuai dengan domain kerjanya. Variasi ini mengelompokkan plugin berdasarkan jenis pemrosesan data yang mereka lakukan. 

#### 4.1. Analytics Plugin
Berfokus pada pemrosesan statistik, agregasi, dan pengenalan pola dari data mentah pada sensor. Tugasnya adalah mengambil data waktu seperti suhu, kelembapan, pH, dan Electrical Conductivity (EC). Lalu mencari nilai rata-rata, tren harian, atau mendeteksi indikasi adanya anomali seperti lonjakan keasaman pH yang tidak wajar. Untuk implementasinya yaitu dengan menggunakan antarmuka dengan fungsi seperti fn analyze().

#### 4.2. Model Plugin
Plugin ini bertanggung jawab untuk menjalankan model prediksi seperti model regresi analitik. Untuk perannya sendiri yaitu mengubah data analitik menjadi prediksi masa depan. Contohnya, memprediksi resiko kekeringan atau resiko genangan air. Untuk implementasinya menggunakan fungsi fn predict() yang nantinya output yang dikeluarkan plugin ini biasanya berupa skor probabilitas.

#### 4.3. Plant Rule Plugin
Plugin ini berfungsi sebagai Sistem Pakar untuk satu jenis tanaman tertentu. Plugin ini berisi kumpulan rule based dan ambang batas argonomis. Peran plugin ini yaitu memproses kondisi tanah yang sifattnya umum menjadi evaluasi dan rekomendasi spesifik untuk tanaman tertentu. Untuk implementasinya menggunakan fungsi fn evaluate().

## 5. Struktur Modul
Modul ini menggunakan tipe data kompleks dan pembagian fungsi berdasar fungsionalitas untuk mengakomodir interaksi dengan modul lainnya.
#### 5.1  Tipe Data

Modul ini menggunakan beberapa tipe data tambahan yang digunakan untuk keperluan validasi dan pemrosesan data. Berikut adalah tipe data yang digunakan sebagai metadata plugin:
```Rust
pub struct Version{
	pub major: u32,
	pub minor: u32,
	pub patch: u32,
}

pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub api_version: Version,
    pub plugin_type: PluginType,
}

pub enum PluginType {
    Analytics,
    Model,
    PlantRule,
}
```

Tipe data yang digunakan untuk error handling
```Rust
pub enum PluginError {
    InvalidMetadata,
    DuplicatePlugin,
    PluginNotFound,
    IncompatibleVersion,
    UnsupportedPluginType,
}
```

Tipe data yang digunakan sebagai batasan input dan output plugin
```Rust
	pub struct AnalyticsInput {}
	pub struct AnalyticsOutput {}
	pub struct ModelInput {}
	pub struct ModelOutput {}
	pub struct PlantRuleInput {}
	pub struct PlantRuleOutput {}
```
#### 5.1. Interface Plugin
Terdapat tiga tipe plugin sehingga akan ada terdapat perbedaan interface, fungsi dan batasan untuk setiap kontrak. 

| Fungsi | Signature | Deskripsi |
| -------------- | ----------------------------- | ------------------ |
| `metadata` | `fn metadata(&self)-> &PluginMetadata;` | Pure function yang digunakan untuk mengembalikan referensi metadata dari instance plugin.|
| `analyze` | `fn analyze(&self, input: AnalyticsInput,) -> Result<AnalyticsOutput, PluginError>;` | Pure function yang digunakan untuk mengembalikan data hasil pemrosesan berdsasarkan implementasi analitik plugin |
 |`predict`|`fn predict(&self, input: ModelInput,) -> Result<ModelOutput, PluginError>;`|Pure function yang digunakan untuk mengembalikan data hasil pemrosesan berdsasarkan implementasi model AI |
 |`evaluate`|`    fn evaluate(&self,input: PlantRuleInput,) -> Result<PlantRuleOutput, PluginError>;`|Pure function yang digunakan untuk mengembalikan data hasil pemrosesan berdsasarkan aturan khusus jenis tanaman tertentu|

#### 5.2. Manager Plugin
Untuk dapat mengakomodir kebutuhan lifecycle dari plugin maka didefinisikan fungsi-fungsi berikut:

| Function | Signature | Description |
|---|---|---|
| `register` | `pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), PluginError>` | Daftarkan plugin dalam argumen dan kembalikan Respon atau Error |
| `unregister` | `pub fn unregister(&mut self, plugin_id: &str) -> Result<(), PluginError>` |Hapus plugin dalam argumen dari dafter dan kembalikan Respon atau Error|
| `get` | `pub fn get(&self, plugin_id: &str) -> Option<&dyn Plugin>` |Pure function yang mengembalikan referensi plugin yang sesuai berdasarkan argumen `plugin_id` atau `None` jika plugin tidak ditemukan |
| `list` | `pub fn list(&self) -> Vec<&dyn Plugin>` |Pure function yang mengembalikan list referensi seluruh plugin yang sudah terdaftar|
| `list_metadata` | `pub fn list_metadata(&self) -> Vec<&PluginMetadata>` |Pure function yang mengembalikan list referensi metadara seluruh plugin yang sudah terdaftar |
| `discover` | `pub fn discover(&self) -> Result<Vec<PluginMetadata>, PluginError>` | Mencari seluruh plugin yang tersedia pada direktori dan mengembalikan list metadata seluruh plugin yang tersedia pada direktori plugin atau Error|
| `validate` | `pub fn validate(&self, plugin: &dyn Plugin) -> Result<(), PluginError>` | Pure function yang digunakan untuk memvalidasi plugin |
| `enable` | `pub fn enable(&mut self, plugin_id: &str) -> Result<(), PluginError>` | Mengaktifkan referensi plugin pada argumen `plugin_id` dan mengembalikan Respon atau Error|
| `disable` | `pub fn disable(&mut self, plugin_id: &str) -> Result<(), PluginError>` | Menonaktifkan referensi plugin pada argumen `plugin_id` dan mengembalikan Respon atau Error |
| `initialize` | `pub fn initialize(&mut self, plugin_id: &str) -> Result<(), PluginError>` | Mempersiapkan resource yang diperlukan referensi plugin pada argumen `plugin_id` sebelum eksekusi dan kembalikan Respon atau Error |
| `shutdown` | `pub fn shutdown(&mut self, plugin_id: &str) -> Result<(), PluginError>` | Menghentikan dan membersikan resource yang diperlukan referensi plugin pada argumen `plugin_id` dan kembalikan Respon atau Error|
| `execute_analytics` | `pub fn execute_analytics(&self, plugin_id: &str, input: AnalyticsInput) -> Result<AnalyticsOutput, PluginError>` |Eksekusi plugin Analytics menggunakan data yang tersedia dan kembalikan outputnya |
| `execute_model` | `pub fn execute_model(&self, plugin_id: &str, input: ModelInput) -> Result<ModelOutput, PluginError>` |Eksekusi plugin Model menggunakan data yang tersedia dan kembalikan prediksi modelnya |
| `execute_plant_rule` | `pub fn execute_plant_rule(&self, plugin_id: &str, input: PlantRuleInput) -> Result<PlantRuleOutput, PluginError>` |Eksekusi plugin PlantRule menggunakan data yang tersedia dan kembalikan respon spesifik tanaman

##6. Interaksi Antar Modul
Fungsionalitas plugin dapat dieksekusi oleh modul lainnya menggunakan fungsi-fungsi yang tersedia menggunakan input yang sudah ditetapkan pada kontrak. Contohnya adalah Module 4- dapat menggunakan plugin yang bertujuan untuk memroses data dengan metode analitik yang lebih kompleks hanya dengan mengeksekusi plugin dengan data yang diperlukan tanpa harus merubah source-code atau mengompilasi ulang seluruh program.
