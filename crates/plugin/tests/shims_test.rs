use std::path::Path;

use proto_pdk_test_utils::{
    LocateExecutablesInput, PluginContext, VersionSpec, VirtualPath, create_empty_proto_sandbox,
    create_plugin, generate_shims_test,
};

generate_shims_test!("nix", ["nix"]);

#[tokio::test(flavor = "multi_thread")]
async fn maps_executable_to_nix_binary() {
    let sandbox = create_empty_proto_sandbox();
    let plugin = sandbox.create_plugin("nix").await;

    let output = plugin
        .locate_executables(LocateExecutablesInput {
            context: PluginContext {
                version: VersionSpec::parse("2.24.8").unwrap(),
                ..PluginContext::default()
            },
            install_dir: VirtualPath::default(),
        })
        .await;

    let nix = output.exes.get("nix").unwrap();

    assert_eq!(nix.exe_path.as_deref(), Some(Path::new("bin/nix")));
    assert!(nix.primary);
    assert!(!output.exes.contains_key("Nix"));
}
