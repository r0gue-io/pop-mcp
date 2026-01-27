// use crate::common::{is_error, is_success, text, Chain, TestEnv};
// use anyhow::Result;
// use pop_mcp_server::tools::test::chain::{test_chain, TestChainParams};

// #[test]
// fn test_chain_passes_on_shared_project() -> Result<()> {
//     let env = TestEnv::new()?;
//     let chain = Chain::create_build_or_use()?;

//     let result = test_chain(
//         env.executor(),
//         TestChainParams {
//             path: chain.path.display().to_string(),
//         },
//     )?;

//     assert!(is_success(&result));
//     assert!(text(&result)?.contains("Tests completed!"));
//     Ok(())
// }

// #[test]
// fn test_chain_nonexistent_path_fails() -> Result<()> {
//     let env = TestEnv::new()?;
//     let params = TestChainParams {
//         path: "/nonexistent/path/to/chain".to_string(),
//     };

//     let result = test_chain(env.executor(), params)?;
//     assert!(is_error(&result));
//     assert!(text(&result)?.contains("Tests failed"));
//     Ok(())
// }
