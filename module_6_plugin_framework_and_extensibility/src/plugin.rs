use crate::metadata::PluginMetadata;
use crate::error::PluginError;
use crate::plugin_io::*;
/// Basis interface yang digunakan seluruh plugin.
pub trait Plugin {
    /// Mengembalikan referensi metadata dari plugin.
    fn metadata(&self)-> &PluginMetadata;
}

/// Interface plugin analitik yang berhubungan dengan Module-4.
pub trait AnalyticsPlugin: Plugin {
    /// Analisa data dari sensor dan statistik.
    /// 
    /// # Arguments
    /// 
    /// * `input` - Soil data yang akan diproses olehh plugin.
    /// 
    /// # Returns
    /// Kembalikan data analisis, atau "PluginError" apabila
    /// analisis tidak dapat diselesaikan.
    fn analyze(
        &self,
        input: AnalyticsInput,
    ) -> Result<AnalyticsOutput, PluginError>;
}

/// Interface plugin pemrosesan data menggunakan AI berhubungan dengan Module-4.
pub trait ModelPlugin: Plugin {
    /// Proses data sensor dan statistik menggunakan model AI.
    /// 
    /// # Arguments
    /// 
    /// * `input` - Soil data yang akan diproses olehh plugin.
    /// 
    /// # Returns
    /// Kembalikan output AI, atau "PluginError" apabila
    /// proses tidak dapat diselesaikan.
    fn predict(
        &self,
        input: ModelInput,
    ) -> Result<ModelOutput, PluginError>;
}

/// Interface plugin untuk pemrosesan data pada jenis tanaman tertentu
pub trait PlantRulePlugin: Plugin {
    /// Proses data sensor dan statistik berdasarkan dan rekomendasi
    /// untuk jenis tanaman tertentu.
    /// 
    /// # Arguments
    /// 
    /// * `input` - Soil data yang akan diproses olehh plugin.
    /// 
    /// # Returns
    /// Kembalikan aturan spesifik tumbuhan, atau "PluginError" apabila
    /// proses tidak dapat diselesaikan.
    fn evaluate(
        &self,
        input: PlantRuleInput,
    ) -> Result<PlantRuleOutput, PluginError>;
}