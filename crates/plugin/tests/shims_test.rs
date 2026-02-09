use std::path::{Path, PathBuf};

use proto_pdk_test_utils::{
    LocateExecutablesInput, PluginContext, VersionSpec, VirtualPath, create_empty_proto_sandbox,
};

fn ensure_proto_shim_binary() {
    let shim_name = if cfg!(windows) {
        "proto-shim.exe"
    } else {
        "proto-shim"
    };

    let target_dirs = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .into_iter()
        .chain(std::iter::once(PathBuf::from("target")));

    for target_dir in target_dirs {
        let bin_dir = target_dir.join("debug");
        std::fs::create_dir_all(&bin_dir).unwrap();

        let shim_path = bin_dir.join(shim_name);

        if !shim_path.exists() {
            std::fs::write(&shim_path, b"proto-shim-test-binary").unwrap();

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                let mut permissions = std::fs::metadata(&shim_path).unwrap().permissions();
                permissions.set_mode(0o755);
                std::fs::set_permissions(&shim_path, permissions).unwrap();
            }
        }

        if shim_path.exists() {
            return;
        }
    }

    panic!("Failed to create test proto-shim binary");
}

#[tokio::test(flavor = "multi_thread")]
async fn creates_shims() {
    ensure_proto_shim_binary();

    let sandbox = create_empty_proto_sandbox();
    let mut plugin = sandbox.create_plugin("nix").await;

    plugin.tool.generate_shims(false).await.unwrap();

    let shim_path = sandbox.proto_dir.join("shims").join(if cfg!(windows) {
        "nix.exe"
    } else {
        "nix"
    });
    assert!(shim_path.exists());

    let registry_path = sandbox.path().join(".proto/shims/registry.json");
    assert!(registry_path.exists());

    let registry = std::fs::read_to_string(registry_path).unwrap();
    assert!(registry.contains("\"nix\""));
}

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
