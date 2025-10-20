#[derive(Debug, schematic::Schematic, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct PluginConfig {
    pub dist_url: String,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            dist_url:
                "https://releases.nixos.org/nix/nix-{version}/nix-{version}-{arch}-{os}.tar.xz"
                    .into(),
        }
    }
}
