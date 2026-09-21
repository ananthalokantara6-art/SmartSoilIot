pub mod plugin;
pub mod metadata;
pub mod error;
pub mod plugin_io;
pub mod manager;

pub use manager::PluginManager;
pub use plugin::{
    Plugin,
    AnalyticsPlugin,
    ModelPlugin,
    PlantRulePlugin,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {

    }
}
