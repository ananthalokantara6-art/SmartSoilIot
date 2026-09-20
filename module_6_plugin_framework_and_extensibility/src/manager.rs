use crate::plugin::Plugin;
use crate::metadata::PluginMetadata;
use crate::error::PluginError;

pub struct PluginManager {}

impl PluginManager {
    pub fn register(
        &mut self,
        plugin: Box<dyn Plugin>,
    ) -> Result<(), PluginError>{
        todo!()
    }

    pub fn unregister(
        &mut self,
        plugin_id: &str,
    ) -> Result<(), PluginError>{
        todo!()
    }

    pub fn get(
        &self,
        plugin_id: &str,
    ) -> Option<&dyn Plugin>{
        todo!()
    }

    pub fn list(
        &self,
    ) -> Vec<&dyn Plugin>{
        todo!()
    }

    pub fn list_metadata(
        &self,
    ) -> Vec<&PluginMetadata>{
        todo!()
    }

    pub fn discover(
        &self,
    ) -> Result<Vec<PluginMetadata>, PluginError>{
        todo!()
    }
}