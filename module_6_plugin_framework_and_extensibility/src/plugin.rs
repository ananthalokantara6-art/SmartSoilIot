use crate::metadata::PluginMetadata;
use crate::error::PluginError;
use crate::plugin_io::*;

pub trait Plugin {

    fn metadata(&self)-> &PluginMetadata;
}

pub trait AnalyticsPlugin: Plugin {
    fn analyze(
        &self,
        input: AnalyticsInput,
    ) -> Result<AnalyticsOutput, PluginError>;
}

pub trait ModelPlugin: Plugin {
    fn predict(
        &self,
        input: ModelInput,
    ) -> Result<ModelOutput, PluginError>;
}

pub trait PlantRulePlugin: Plugin {
    fn evaluate(
        &self,
        input: PlantRuleInput,
    ) -> Result<PlantRuleOutput, PluginError>;
}