use proto_pdk_test_utils::{
    InstallStrategy, PluginType, RegisterToolInput, Version, create_empty_proto_sandbox,
};

#[tokio::test(flavor = "multi_thread")]
async fn registers_expected_tool_metadata() {
    let sandbox = create_empty_proto_sandbox();
    let plugin = sandbox.create_plugin("nix").await;

    let output = plugin.register_tool(RegisterToolInput::default()).await;

    assert_eq!(output.name, "Nix");
    assert_eq!(output.type_of, PluginType::DependencyManager);
    assert_eq!(
        output.default_install_strategy,
        InstallStrategy::DownloadPrebuilt
    );
    assert_eq!(output.minimum_proto_version, Some(Version::new(0, 46, 0)));
}
