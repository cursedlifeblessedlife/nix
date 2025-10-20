use std::collections::HashMap;

use extism_pdk::*;
use proto_pdk::*;
use schematic::SchemaBuilder;

use crate::{DistTag, PluginConfig};

#[host_fn]
extern "ExtismHost" {
    fn exec_command(input: Json<ExecCommandInput>) -> Json<ExecCommandInput>;
}

static NAME: &str = "Nix";

#[plugin_fn]
pub fn register_tool(Json(_): Json<RegisterToolInput>) -> FnResult<Json<RegisterToolOutput>> {
    Ok(Json(RegisterToolOutput {
        name: NAME.into(),
        minimum_proto_version: Some(Version::new(0, 46, 0)),
        type_of: PluginType::DependencyManager,
        default_install_strategy: InstallStrategy::DownloadPrebuilt,
        plugin_version: Version::parse(env!("CARGO_PKG_VERSION")).ok(),
        ..RegisterToolOutput::default()
    }))
}

#[plugin_fn]
pub fn define_tool_config(_: ()) -> FnResult<Json<DefineToolConfigOutput>> {
    Ok(Json(DefineToolConfigOutput {
        schema: SchemaBuilder::build_root::<PluginConfig>(),
    }))
}

#[plugin_fn]
pub fn download_prebuilt(
    Json(input): Json<DownloadPrebuiltInput>,
) -> FnResult<Json<DownloadPrebuiltOutput>> {
    let env = get_host_environment()?;
    let config = get_tool_config::<PluginConfig>()?;

    check_supported_os_and_arch(
        NAME,
        &env,
        permutations! [
            HostOS::Linux => [HostArch::X64, HostArch::Arm64, HostArch::Arm],
            HostOS::MacOS => [HostArch::X64, HostArch::Arm64],
        ],
    )?;

    let version = input.context.version;
    let arch = env.arch;
    let os = env.os;

    let arch_slug = match arch {
        HostArch::X64 => "x86_64".to_string(),
        HostArch::Arm64 => "aarch64".to_string(),
        HostArch::Arm => "armv7l".to_string(),
        _ => arch.to_string(),
    };
    let os_slug = match os {
        HostOS::Linux => "linux".to_string(),
        HostOS::MacOS => "darwin".to_string(),
        other => {
            return Err(PluginError::UnsupportedOS {
                tool: NAME.to_string(),
                os: other.to_string(),
            }
            .into());
        }
    };

    let prefix = format!("nix-{version}/nix-{version}-{arch_slug}-{os_slug}");
    let download_url = config
        .dist_url
        .replace("{version}", version.to_string().as_str())
        .replace("{arch}", &arch_slug)
        .replace("{os}", &os_slug);
    let filename = format!("{}.tar.xz", prefix);
    let checksum_url = format!("{}.sha256", download_url);

    Ok(Json(DownloadPrebuiltOutput {
        archive_prefix: Some(prefix),
        download_url: download_url,
        download_name: Some(filename),
        checksum_url: Some(checksum_url),
        ..DownloadPrebuiltOutput::default()
    }))
}

#[plugin_fn]
pub fn locate_executables(
    Json(_): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    Ok(Json(LocateExecutablesOutput {
        exes: HashMap::from_iter([(
            NAME.into(),
            ExecutableConfig::new_primary("install-multi-user"),
        )]),
        ..LocateExecutablesOutput::default()
    }))
}

#[plugin_fn]
pub fn load_versions(Json(_): Json<LoadVersionsInput>) -> FnResult<Json<LoadVersionsOutput>> {
    let mut page = 1usize;
    let size = 100usize;
    let mut tags: Vec<DistTag> = Vec::new();

    loop {
        let url = format!(
            "https://api.github.com/repos/NixOS/nix/tags?per_page={}&page={}",
            size, page
        );
        let batch: Vec<DistTag> = fetch_json(&url)?;
        if batch.is_empty() {
            break;
        }
        tags.extend(batch);
        page += 1;
    }

    let mut versions: Vec<VersionSpec> = tags
        .into_iter()
        .filter_map(|t| {
            let s = t.name.trim();
            let s = s.strip_prefix('v').unwrap_or(s);
            VersionSpec::parse(s).ok()
        })
        .collect();

    versions.sort();

    let output = LoadVersionsOutput::from_versions(versions);

    Ok(Json(output))
}
