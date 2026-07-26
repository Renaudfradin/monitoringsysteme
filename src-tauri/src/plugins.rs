//! Internal metric plugin registry — compile-time plugins for new metric families.

use std::sync::Arc;

use serde_json::json;

use crate::error::MetricError;
use crate::models::PluginInfo;
use crate::provider::SystemProvider;

pub trait MetricPlugin: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn collect(&self, provider: &dyn SystemProvider) -> Result<serde_json::Value, MetricError>;
}

macro_rules! builtin_plugin {
    ($ty:ident, $id:expr, $name:expr, $desc:expr, $method:ident) => {
        pub struct $ty;
        impl MetricPlugin for $ty {
            fn id(&self) -> &'static str {
                $id
            }
            fn name(&self) -> &'static str {
                $name
            }
            fn description(&self) -> &'static str {
                $desc
            }
            fn collect(
                &self,
                provider: &dyn SystemProvider,
            ) -> Result<serde_json::Value, MetricError> {
                let v = provider.$method()?;
                serde_json::to_value(v).map_err(|e| MetricError::Internal(e.to_string()))
            }
        }
    };
}

builtin_plugin!(CpuPlugin, "cpu", "CPU", "Utilisation processeur", cpu);
builtin_plugin!(MemoryPlugin, "memory", "RAM", "Mémoire système", memory);
builtin_plugin!(DiskPlugin, "disk", "Disque", "Volume racine", disk);
builtin_plugin!(EnergyPlugin, "energy", "Énergie", "Consommation estimée", energy);
builtin_plugin!(
    TemperaturePlugin,
    "temperature",
    "Température",
    "Capteurs thermiques",
    temperature
);
builtin_plugin!(
    ProcessesPlugin,
    "processes",
    "Processus",
    "Top CPU / RAM",
    processes
);
builtin_plugin!(NetworkPlugin, "network", "Réseau", "Débit interfaces", network);
builtin_plugin!(GpuPlugin, "gpu", "GPU / Fans", "GPU live et ventilateurs", gpu);

pub struct PluginRegistry {
    plugins: Vec<Arc<dyn MetricPlugin>>,
}

impl PluginRegistry {
    pub fn with_builtins() -> Self {
        Self {
            plugins: vec![
                Arc::new(CpuPlugin),
                Arc::new(MemoryPlugin),
                Arc::new(DiskPlugin),
                Arc::new(EnergyPlugin),
                Arc::new(TemperaturePlugin),
                Arc::new(ProcessesPlugin),
                Arc::new(NetworkPlugin),
                Arc::new(GpuPlugin),
            ],
        }
    }

    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins
            .iter()
            .map(|p| PluginInfo {
                id: p.id().into(),
                name: p.name().into(),
                description: p.description().into(),
                builtin: true,
            })
            .collect()
    }

    pub fn invoke(
        &self,
        id: &str,
        provider: &dyn SystemProvider,
    ) -> Result<serde_json::Value, MetricError> {
        let plugin = self
            .plugins
            .iter()
            .find(|p| p.id() == id)
            .ok_or_else(|| MetricError::Unavailable(format!("plugin inconnu: {id}")))?;
        plugin.collect(provider)
    }

    #[allow(dead_code)]
    pub fn catalog_json(&self) -> serde_json::Value {
        json!({
            "plugins": self.list(),
            "extensionPoint": "Register MetricPlugin implementations in PluginRegistry::with_builtins (or a future dynamic loader)."
        })
    }

    /// Extension hook for future dynamic plugins.
    #[allow(dead_code)]
    pub fn register(&mut self, plugin: Arc<dyn MetricPlugin>) {
        self.plugins.push(plugin);
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::with_builtins()
    }
}
