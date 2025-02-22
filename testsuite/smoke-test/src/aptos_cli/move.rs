// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::smoke_test_environment::SwarmBuilder;
use aptos::move_tool::MemberId;
use aptos::test::CliTestFramework;
use aptos_logger::info;
use move_deps::move_package::source_package::manifest_parser::parse_move_manifest_from_file;
use std::collections::BTreeMap;
use std::str::FromStr;

const PACKAGE_NAME: &str = "AwesomePackage";
const HELLO_BLOCKCHAIN: &str = "HelloBlockchain";

#[tokio::test]
async fn test_move_compile_flow() {
    let mut cli = CliTestFramework::local_new(1);

    cli.init_move_dir();
    let move_dir = cli.move_dir();
    let account = cli.account_id(0).to_hex_literal();

    let mut package_addresses = BTreeMap::new();
    package_addresses.insert(HELLO_BLOCKCHAIN, "_");

    cli.init_package(PACKAGE_NAME.to_string(), package_addresses)
        .await
        .expect("Should succeed");

    // The manifest should work to compile
    let mut named_addresses = BTreeMap::new();
    named_addresses.insert(HELLO_BLOCKCHAIN, account.as_str());
    match cli.compile_package(named_addresses.clone()).await {
        Ok(modules) => assert!(modules.is_empty()),
        Err(err) => panic!("Error compiling: {:?}", err),
    }

    // Let's check that it's setup correctly
    let manifest = parse_move_manifest_from_file(move_dir.join("Move.toml").as_path())
        .expect("Expect a Move.toml file");
    assert_eq!(manifest.package.name.as_str(), PACKAGE_NAME);
    // Expect "0.0.0"
    assert_eq!(manifest.package.version.0, 0);
    assert_eq!(manifest.package.version.1, 0);
    assert_eq!(manifest.package.version.2, 0);

    let addresses = manifest.addresses.expect("Expect some addresses");
    assert_eq!(addresses.len(), 1);
    let (key, value) = addresses.iter().next().expect("Expect an address");
    assert_eq!(key.as_str(), HELLO_BLOCKCHAIN);
    assert!(value.is_none());

    assert_eq!(manifest.dependencies.len(), 1);

    let dependency = manifest.dependencies.iter().next().unwrap();
    assert_eq!("AptosFramework", dependency.0.to_string());
    dependency
        .1
        .git_info
        .as_ref()
        .expect("Expect some git information");

    // Now try to compile real code
    cli.add_move_files();

    match cli.compile_package(named_addresses.clone()).await {
        Ok(modules) => assert!(!modules.is_empty()),
        Err(err) => panic!("Error compiling: {:?}", err),
    }

    // Run tests to ensure they work too
    match cli.test_package(named_addresses.clone(), None).await {
        Ok(result) => assert_eq!("Success", result),
        Err(err) => panic!("Error testing: {:?}", err),
    }
}

#[tokio::test]
async fn test_move_publish_flow() {
    let (_swarm, mut cli, _faucet) = SwarmBuilder::new_local(1)
        .with_aptos()
        .build_with_cli(2)
        .await;

    let account = cli.account_id(0).to_hex_literal();
    // Setup move package
    cli.init_move_dir();
    let mut package_addresses = BTreeMap::new();
    package_addresses.insert(HELLO_BLOCKCHAIN, "_");
    cli.init_package(PACKAGE_NAME.to_string(), package_addresses)
        .await
        .expect("Should succeed");
    cli.add_move_files();

    cli.wait_for_account(0)
        .await
        .expect("Should create account");
    info!("Move package dir: {}", cli.move_dir().display());

    // Let's publish it
    let mut named_addresses = BTreeMap::new();
    named_addresses.insert(HELLO_BLOCKCHAIN, account.as_str());
    let _ = match cli
        .publish_package(0, None, named_addresses, false, None)
        .await
    {
        Ok(response) => response,
        Err(err) => panic!("Should not have failed to publish package {:?}", err),
    };

    // TODO: Verify transaction summary

    // Wrong number of args will definitely fail
    let function_id = MemberId::from_str(&format!("{}::message::set_message", account)).unwrap();

    assert!(cli
        .run_function(0, None, function_id.clone(), vec![], vec![])
        .await
        .is_err());

    assert!(cli
        .run_function(0, None, function_id, vec!["string:hello_world"], vec![])
        .await
        .is_ok());
    // TODO: Verify output
}
