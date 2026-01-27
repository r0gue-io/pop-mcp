// use crate::common::{is_error, is_success, text, TestEnv};
// use anyhow::Result;
// use pop_mcp_server::tools::build::chain::{build_chain, BuildChainParams};
// use pop_mcp_server::tools::new::chain::{create_chain, CreateChainParams};

// #[test]
// fn build_chain_nonexistent_path_fails() -> Result<()> {
//     let env = TestEnv::new()?;
//     let params = BuildChainParams {
//         path: "/nonexistent/path/to/chain".to_string(),
//         release: None,
//     };

//     let result = build_chain(env.executor(), params)?;
//     assert!(is_error(&result));
//     assert!(text(&result)?.contains("Chain build failed"));
//     Ok(())
// }

// #[test]
// fn build_chain_creates_binary() -> Result<()> {
//     let env = TestEnv::new()?;

//     // Create a fresh chain
//     let create_result = create_chain(
//         env.executor(),
//         CreateChainParams {
//             name: "build_chain_test".to_string(),
//             provider: "pop".to_string(),
//             template: "r0gue-io/base-parachain".to_string(),
//             symbol: Some("TEST".to_string()),
//             decimals: Some(18),
//         },
//     )?;
//     assert!(is_success(&create_result));

//     let chain_path = env.workdir().join("build_chain_test");

//     // Build it
//     let result = build_chain(
//         env.executor(),
//         BuildChainParams {
//             path: chain_path.display().to_string(),
//             release: Some(true),
//         },
//     )?;

//     assert!(is_success(&result));
//     assert!(text(&result)?.contains("Chain build successful"));
//     assert!(chain_path.join("target/release").exists());
//     Ok(())
// }
