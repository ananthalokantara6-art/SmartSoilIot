pub struct Version {
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