use std::path::Path;

use proto_pdk_test_utils::{
    DownloadPrebuiltInput, HostArch, HostOS, PluginContext, VersionSpec, VirtualPath,
    create_empty_proto_sandbox,
};

#[tokio::test(flavor = "multi_thread")]
async fn builds_expected_download_output() {
    let sandbox = create_empty_proto_sandbox();
    let plugin = sandbox
        .create_plugin_with_config("nix", |config| {
            config.host(HostOS::MacOS, HostArch::X64);
        })
        .await;

    let output = plugin
        .download_prebuilt(DownloadPrebuiltInput {
            context: PluginContext {
                version: VersionSpec::parse("2.24.8").unwrap(),
                ..PluginContext::default()
            },
            install_dir: VirtualPath::default(),
        })
        .await;

    assert_eq!(
        output.download_url,
        "https://releases.nixos.org/nix/nix-2.24.8/nix-2.24.8-x86_64-darwin.tar.xz"
    );
    assert_eq!(
        output.archive_prefix,
        Some("nix-2.24.8/nix-2.24.8-x86_64-darwin".into())
    );
    assert_eq!(
        output.download_name,
        Some("nix-2.24.8-x86_64-darwin.tar.xz".into())
    );
    assert_eq!(
        output.checksum_url,
        Some(
            "https://releases.nixos.org/nix/nix-2.24.8/nix-2.24.8-x86_64-darwin.tar.xz.sha256"
                .into()
        )
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn creates_expected_archive_filename() {
    let sandbox = create_empty_proto_sandbox();
    let plugin = sandbox
        .create_plugin_with_config("nix", |config| {
            config.host(HostOS::Linux, HostArch::Arm64);
        })
        .await;

    let output = plugin
        .download_prebuilt(DownloadPrebuiltInput {
            context: PluginContext {
                version: VersionSpec::parse("2.24.8").unwrap(),
                ..PluginContext::default()
            },
            install_dir: VirtualPath::default(),
        })
        .await;

    assert_eq!(
        output.download_name.as_deref(),
        Some("nix-2.24.8-aarch64-linux.tar.xz")
    );
    assert_eq!(
        output
            .download_name
            .as_deref()
            .and_then(|name| Path::new(name).extension())
            .and_then(|ext| ext.to_str()),
        Some("xz")
    );
}
