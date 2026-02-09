use std::collections::HashMap;

use extism_pdk::*;
use proto_pdk::*;
use schematic::SchemaBuilder;

use crate::{DistTag, PluginConfig};

static TOOL_NAME: &str = "Nix";
static EXE_NAME: &str = "nix";

pub fn parse_dist_tag_version(tag_name: &str) -> Option<VersionSpec> {
    let value = tag_name.trim();
    let value = value.strip_prefix('v').unwrap_or(value);

    if !value
        .chars()
        .next()
        .is_some_and(|char| char.is_ascii_digit())
    {
        return None;
    }

    VersionSpec::parse(value).ok()
}

fn to_arch_slug(arch: HostArch) -> String {
    match arch {
        HostArch::X64 => "x86_64".to_string(),
        HostArch::Arm64 => "aarch64".to_string(),
        HostArch::Arm => "armv7l".to_string(),
        _ => arch.to_string(),
    }
}

fn to_os_slug(os: HostOS) -> Result<String, PluginError> {
    match os {
        HostOS::Linux => Ok("linux".to_string()),
        HostOS::MacOS => Ok("darwin".to_string()),
        other => Err(PluginError::UnsupportedOS {
            tool: TOOL_NAME.to_string(),
            os: other.to_string(),
        }),
    }
}

fn create_download_prebuilt_output(
    config: &PluginConfig,
    version: &VersionSpec,
    arch: HostArch,
    os: HostOS,
) -> Result<DownloadPrebuiltOutput, PluginError> {
    let arch_slug = to_arch_slug(arch);
    let os_slug = to_os_slug(os)?;
    let version = version.to_string();
    let prefix = format!("nix-{version}/nix-{version}-{arch_slug}-{os_slug}");
    let download_url = config
        .dist_url
        .replace("{version}", &version)
        .replace("{arch}", &arch_slug)
        .replace("{os}", &os_slug);
    let filename = format!("nix-{version}-{arch_slug}-{os_slug}.tar.xz");
    let checksum_url = format!("{}.sha256", download_url);

    Ok(DownloadPrebuiltOutput {
        archive_prefix: Some(prefix),
        download_url,
        download_name: Some(filename),
        checksum_url: Some(checksum_url),
        ..DownloadPrebuiltOutput::default()
    })
}

fn locate_nix_executables() -> [(String, ExecutableConfig); 1] {
    [(EXE_NAME.into(), ExecutableConfig::new_primary("bin/nix"))]
}

#[plugin_fn]
pub fn register_tool(Json(_): Json<RegisterToolInput>) -> FnResult<Json<RegisterToolOutput>> {
    Ok(Json(RegisterToolOutput {
        name: TOOL_NAME.into(),
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
        TOOL_NAME,
        &env,
        permutations! [
            HostOS::Linux => [HostArch::X64, HostArch::Arm64, HostArch::Arm],
            HostOS::MacOS => [HostArch::X64, HostArch::Arm64],
        ],
    )?;

    let output =
        create_download_prebuilt_output(&config, &input.context.version, env.arch, env.os)?;

    Ok(Json(output))
}

#[plugin_fn]
pub fn locate_executables(
    Json(_): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    Ok(Json(LocateExecutablesOutput {
        exes: HashMap::from_iter(locate_nix_executables()),
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
        .filter_map(|t| parse_dist_tag_version(&t.name))
        .collect();

    versions.sort();

    let output = LoadVersionsOutput::from_versions(versions);

    Ok(Json(output))
}
