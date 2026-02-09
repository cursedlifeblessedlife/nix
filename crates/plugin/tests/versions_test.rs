use proto_pdk_test_utils::{
    LoadVersionsInput, PluginUnresolvedContext, UnresolvedVersionSpec, VersionSpec,
    create_empty_proto_sandbox,
};

#[tokio::test(flavor = "multi_thread")]
async fn loads_sorted_versions_with_latest_alias() {
    let sandbox = create_empty_proto_sandbox();
    let plugin = sandbox.create_plugin("nix").await;

    let output = plugin
        .load_versions(LoadVersionsInput {
            context: PluginUnresolvedContext::default(),
            initial: UnresolvedVersionSpec::parse("latest").unwrap(),
        })
        .await;

    assert!(!output.versions.is_empty());
    assert!(output.versions.windows(2).all(|pair| pair[0] <= pair[1]));
    assert_eq!(output.aliases.get("latest"), output.latest.as_ref());
}

#[tokio::test(flavor = "multi_thread")]
async fn filters_non_semver_tags() {
    let sandbox = create_empty_proto_sandbox();
    let plugin = sandbox.create_plugin("nix").await;

    let output = plugin
        .load_versions(LoadVersionsInput {
            context: PluginUnresolvedContext::default(),
            initial: UnresolvedVersionSpec::parse("latest").unwrap(),
        })
        .await;

    assert!(
        !output.versions.iter().any(|version| match version {
            VersionSpec::Canary | VersionSpec::Alias(_) => true,
            VersionSpec::Calendar(_) | VersionSpec::Semantic(_) => false,
        })
    );
}
