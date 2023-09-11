use std::{fs};
use chrono::Utc;
use ethabi::{Token, ethereum_types::H160};
use ethers::prelude::*;
use log::{error, info};
use rand::Rng;
use reqwest::Client;
use secp256k1::{SecretKey};
use serde_json::{json, Value};
use tokio::time::{sleep, Duration, Instant};
use web3::{
    Web3, contract::{Contract, Options}, transports::Http, types::{Address, U256, U64, TransactionReceipt, TransactionParameters}
};
use crate::{
    constants::*,
    utils::{
        config::Config,
        faucet
    }
};


pub async fn execute_blockchain_operations(private_key: &str, address: &str, client: Client, config: &Config) {
    let (_web3_opt, web3_arb, web3_sep, web3_scr_sep) = generate_web3_clients(&config, client.clone());

    check_and_log_balance(&web3_arb, &address, "ETH Arbitrum").await;
    // check_and_log_balance(&web3_opt, &address, "ETH Optimism").await;
    check_and_log_balance(&web3_sep, &address, "ETH Sepolia").await;
    check_and_log_balance(&web3_scr_sep, &address, "ETH Scroll Sepolia").await;


    // Scroll Sepolia faucet
    if config.settings.execute_get_faucet {
        match faucet::bwarelabs_faucet(&client.clone(), &address, &config).await {
            Ok(_c) => info!("| {} | bwarelabs_faucet - Ok", address),
            Err(e) => {
                error!("| {} | Failed to bwarelabs_faucet: {}", address, e.to_string());
                // return;
            }
        }
        random_delay(config.settings.delay_action).await;
    }

    // Sending from Arbitrum to Sepolia.
    if config.settings.execute_get_gas_sepolia {
        match get_gas_sepolia(&private_key, &address, &web3_arb, &config, client.clone()).await {
            Ok(_c) => info!("| {} | get_gas_sepolia - Ok", address),
            Err(e) => {
                error!("| {} | Failed to get_gas_sepolia: {}", address, e.to_string());
                // return;
            }
        }
        random_delay(config.settings.delay_action).await;
    }

    // Bridge from Sepolia to Scroll Sepolia
    if config.settings.execute_bridge_from_sepolia_to_scroll {
        match bridge_from_sepolia_to_scroll(&private_key, &address, &web3_sep, &config, client.clone()).await {
            Ok(_c) => info!("| {} | bridge_from_sepolia_to_scroll - Ok", address),
            Err(e) => {
                error!("| {} | Failed to bridge_from_sepolia_to_scroll: {}", address, e.to_string());
                // return;
            }
        }
        random_delay(config.settings.delay_action).await;
    }

    // Swapping of ETH for GHO tokens.
    for _ in 0..random_reps(config.settings.swap_eth_for_token_reps) {
        if config.settings.execute_swap_eth_for_token {
            match swap_eth_for_token(&private_key, &address, &web3_scr_sep, &config).await {
                Ok(_c) => info!("| {} | swap_eth_for_token - Ok", address),
                Err(e) => {
                    error!("| {} | Failed to swap_eth_for_token: {}", address, e.to_string());
                    // return;
                }
            }
            random_delay(config.settings.delay_action).await;
        }
    }

    // Swapping of GHO tokens for ETH
    for _ in 0..random_reps(config.settings.swap_token_for_eth_reps) {
        if config.settings.execute_swap_token_for_eth {
            match swap_token_for_eth(&private_key, &address, &web3_scr_sep).await {
                Ok(_c) => info!("| {} | swap_token_for_eth - Ok", address),
                Err(e) => {
                    error!("| {} | Failed to swap_token_for_eth: {}", address, e.to_string());
                    // return;
                }
            }
            random_delay(config.settings.delay_action).await;
        }
    }

    // Adding liquidity for the ETH-GHO pair on Uniswap.
    for _ in 0..random_reps(config.settings.add_liquidity_reps) {
        if config.settings.execute_add_liquidity {
            match add_liquidity(&private_key, &address, &web3_scr_sep).await {
                Ok(_c) => info!("| {} | add_liquidity - Ok", address),
                Err(e) => {
                    error!("| {} | Failed to add_liquidity: {}", address, e.to_string());
                    // return;
                }
            }
            random_delay(config.settings.delay_action).await;
        }
    }

    check_and_log_balance(&web3_sep, &address, "ETH Sepolia").await;
    check_and_log_balance(&web3_scr_sep, &address, "ETH Scroll Sepolia").await;
}


async fn get_gas_sepolia(private_key: &str, address: &str, web3: &Web3<Http>, config: &Config, client: Client) -> Result<(), Box<dyn std::error::Error>> {

    let address_str = if address.starts_with("0x") {
                &address[2..]
            } else {
                &address
            };

    let random_value = rand::thread_rng().gen_range(config.settings.sepolia_eth_min..config.settings.sepolia_eth_max);
    let parsed_amount = (random_value * 10f64.powi(config.settings.sepolia_eth_decimal.clone() as i32)).round() / 10f64.powi(config.settings.sepolia_eth_decimal.clone() as i32);

    let final_amount = if parsed_amount > 0.1 {
            0.1
        } else {
            parsed_amount
        };

    let merkly_arb: Address = MERKLY_ARB.parse().expect("Failed to parse Ethereum address");

    let abi_bytes: Vec<u8> = fs::read("abi/sepolia_refuel.json")?;
    let parsed_abi: ethabi::Contract = ethabi::Contract::load(abi_bytes.as_slice())?;
    let contract = Contract::new(web3.eth(), merkly_arb, parsed_abi.clone());

    let zro_payment_address_bytes: Vec<u8> = hex::decode("0000000000000000000000000000000000000000").expect("Failed to convert to bytes");

    let amount_wei: U256 = U256::from((final_amount * 1e18) as u64);

    // Convert the amount_wei to a hex string with 32 characters (64 hex digits)
    let amount_wei_str = format!("{:064x}", amount_wei);

    // Construct the adapter_params
    let adapter_params = format!("00020000000000000000000000000000000000000000000000000000000000030d40{}{}", amount_wei_str, address_str);

    // Decode the adapter_params to bytes
    let adapter_params_bytes: Vec<u8> = hex::decode(&adapter_params).expect("Failed to convert to bytes");

    // Construct the params for the function
    let params = (161u16, zro_payment_address_bytes, adapter_params_bytes.clone());

    // Query the contract
    let fees: (U256, U256) = contract.query("estimateSendFee", params, None, Options::default(), None).await?;
    let gas_price = web3.eth().gas_price().await?;
    let bridge_gas_function = parsed_abi.function("bridgeGas").expect("bridgeGas function not found in ABI");
    // Decode the adapter_params to bytes
    let address_wallet_bytes: Vec<u8> = hex::decode(&address_str).expect("Failed to convert to bytes");

    let data = bridge_gas_function.encode_input(&[
        Token::Uint(U256::from(161)),
        Token::Bytes(address_wallet_bytes.into()),
        Token::Bytes(adapter_params_bytes.clone())
    ])?;
    // println!("{:?}", data);
    let nonce = web3.eth().transaction_count((&address_str).parse().unwrap(), None).await?;
    
    // Check gas price
    check_gas_price(&config).await;

    let txn_request = web3::types::CallRequest {
        from: Some(address_str.parse().unwrap()),
        to: Some(merkly_arb),
        gas: None,
        gas_price: Some(gas_price),
        value: Some(fees.0),
        data: Some(data.clone().into()),
        transaction_type: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };

    let estimated_gas = web3.eth().estimate_gas(txn_request, None).await?;

    let txn_parameters = TransactionParameters {
        nonce: Some(nonce),
        to: Some(merkly_arb),
        value: fees.0,
        gas_price: Some(gas_price),
        gas: estimated_gas,
        data: data.clone().into(),
        chain_id: None,
        transaction_type: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };


    let key_bytes = hex::decode(&private_key).expect("Failed to decode hex");
    let secret_key = SecretKey::from_slice(&key_bytes).expect("Invalid private key bytes");
    let signed_txn = web3.accounts().sign_transaction(txn_parameters, &secret_key).await?;


    let _ = sleep(Duration::from_secs(2));

    let tx_hash = web3.eth().send_raw_transaction(signed_txn.raw_transaction).await?;
    // println!("tx_hash: https://arbiscan.io/tx/{:?}", tx_hash);

    match wait_until_tx_finished(&web3, tx_hash, 360).await {
        Ok((success, returned_tx_hash)) => {
            if success {
                info!("| {} | Transaction was successful! https://arbiscan.io/tx/{:?}", &address, returned_tx_hash);
            } else {
                error!("| {} |Transaction failed! https://arbiscan.io/tx/{:?}", &address, returned_tx_hash);
            }
        },
        Err(err) => error!("Error: {}", err),
    }

    let tx_hash_str = format!("{:?}", tx_hash);
    wait_for_stargate_completion(&tx_hash_str, client).await;

    Ok(())
}


async fn bridge_from_sepolia_to_scroll(private_key: &str, address: &str, web3: &Web3<Http>, config: &Config, client: Client) -> Result<(), Box<dyn std::error::Error>> {
    let address_str = if address.starts_with("0x") {
                &address[2..]
            } else {
                &address
            };

    let address: Address = address.parse().expect("Failed to parse Ethereum address");

    let balance_sepolia: U256;
    let mut attempts = 0;

    loop {
        match web3.eth().balance(address, None).await {
            Ok(balance) => {
                balance_sepolia = balance;
                break;
            },
            Err(e) => {
                error!("Error fetching balance: {}", e);
                if attempts >= 3 {  // max 3 retries
                    return Err("Failed to fetch balance after multiple attempts.".into());
                }
                attempts += 1;
                sleep(Duration::from_secs(20)).await;
            }
        }
    }


    let percentage_to_send = &config.settings.deposit_from_sepolia_to_scroll;
    let percentage_value = (percentage_to_send * 100.0).to_string();
    let value_to_send = balance_sepolia * U256::from_dec_str(&percentage_value)? / U256::from(100);

    let fees_in_wei: U256 = U256::from_dec_str(&format!("{:.0}", &config.settings.fees * 10f64.powi(18))).unwrap(); // Assuming `fees` is in ether
    let value_after_fees = value_to_send - fees_in_wei;

    let gas: u64 = 600_000;

    // Fetch the current gas price from the network
    let current_gas_price: U256 = web3.eth().gas_price().await.expect("Failed to fetch gas price");

    // Convert 2 gwei to its wei representation
    let two_gwei_in_wei: U256 = U256::from(2_000_000_000); // 2 * 10^9

    let gas_price = current_gas_price + two_gwei_in_wei;
    let gas_cost = gas_price * U256::from(gas);

    let value = value_after_fees - gas_cost;

    let scroll_bridge: Address = SCROLL_BRIDGE.parse().expect("Failed to parse Ethereum address");

    let abi_scroll_bytes: Vec<u8> = fs::read("abi/scroll.json")?;
    let parsed_abi: ethabi::Contract = ethabi::Contract::load(abi_scroll_bytes.as_slice())?;

    // Subtract fees (converted to wei) from value
    let fees_in_wei: U256 = U256::from_dec_str(&format!("{:.0}", &config.settings.fees * 10f64.powi(18))).unwrap(); // Assuming `fees` is in ether
    let amount_out = value - fees_in_wei;

    if amount_out < U256::zero() {
        error!("Amount for bridge < 0, possibly due to high fees");
        return Err("Amount for bridge is less than zero, possibly due to high fees".into());
    }

    // Create a transaction
    let deposit_eth_function = parsed_abi.function("depositETH").expect("depositETH function not found in ABI");
    let data = deposit_eth_function.encode_input(&[
        Token::Uint(amount_out),
        Token::Uint(U256::from(168000))
    ])?;

    let nonce = web3.eth().transaction_count(address, None).await?;

    let txn_request = web3::types::CallRequest {
        from: Some(address_str.parse().unwrap()),
        to: Some(scroll_bridge),
        gas: None,
        gas_price: Some(gas_price),
        value: Some(value),
        data: Some(data.clone().into()),
        transaction_type: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };

    let estimated_gas = web3.eth().estimate_gas(txn_request, None).await?;

    let txn_parameters = TransactionParameters {
        nonce: Some(nonce),
        to: Some(scroll_bridge),
        value: value,
        gas_price: Some(gas_price),
        gas: estimated_gas,
        data: data.clone().into(),
        chain_id: None,
        transaction_type: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };

    let key_bytes = hex::decode(&private_key).expect("Failed to decode hex");
    let secret_key = SecretKey::from_slice(&key_bytes).expect("Invalid private key bytes");
    let signed_txn = web3.accounts().sign_transaction(txn_parameters, &secret_key).await?;

    let _ = sleep(Duration::from_secs(2));

    let tx_hash = web3.eth().send_raw_transaction(signed_txn.raw_transaction).await?;

    match wait_until_tx_finished(&web3, tx_hash, 360).await {
        Ok((success, returned_tx_hash)) => {
            if success {
                info!("| 0x{} | Transaction was successful! https://sepolia.etherscan.io/tx/{:?}", &address_str, returned_tx_hash);
            } else {
                error!("| 0x{} |Transaction failed! https://sepolia.etherscan.io/tx/{:?}", &address_str, returned_tx_hash);
            }
        },
        Err(err) => error!("Error: {}", err),
    }

    let tx_hash_str = format!("{:?}", tx_hash);
    wait_for_bridge_completion(&tx_hash_str, client).await;


    Ok(())
}


async fn swap_eth_for_token(private_key: &str, address: &str, web3: &Web3<Http>, config: &Config) -> Result<(), Box<dyn std::error::Error>> {


    let address_str = if address.starts_with("0x") {
                &address[2..]
            } else {
                &address
            };
    let address: Address = address.parse().expect("Failed to parse Ethereum address");

    let uniswap_router: Address = UNISWAP_ROUTER.parse().expect("Failed to parse Ethereum address");
    let eth_scroll_sepolia: Address = ETH_SCROLL_SEPOLIA.parse().expect("Failed to parse Ethereum address");
    let gho_scroll_sepolia: Address = GHO_SCROLL_SEPOLIA.parse().expect("Failed to parse Ethereum address");

    let uniswap_router_abi_bytes: Vec<u8> = fs::read("abi/uniswap.json")?;
    let uniswap_router_parsed_abi: ethabi::Contract = ethabi::Contract::load(uniswap_router_abi_bytes .as_slice())?;

    let random_value = rand::thread_rng().gen_range(config.settings.value_swap_min..config.settings.value_swap_max);
    let parsed_amount = (random_value * 10f64.powi(config.settings.value_swap_decimal.clone() as i32)).round() / 10f64.powi(config.settings.value_swap_decimal.clone() as i32);
    // println!("parsed_amount: {}", parsed_amount);

    let mut parsed_amount_u256: U256 = U256::from_dec_str(&format!("{:.0}", &parsed_amount * 10f64.powi(18))).unwrap();

    let balance_eth_scrooll: U256;
    let mut attempts = 0;
    loop {
        match web3.eth().balance(address, None).await {
            Ok(balance) => {
                balance_eth_scrooll = balance;
                break;
            },
            Err(e) => {
                error!("Error fetching balance: {}", e);
                if attempts >= 3 {  // max 3 retries
                    return Err("Failed to fetch balance after multiple attempts.".into());
                }
                attempts += 1;
                sleep(Duration::from_secs(20)).await;
            }
        }
    }

    let gas: u64 = 500_000;
    let current_gas_price: U256 = web3.eth().gas_price().await.expect("Failed to fetch gas price");
    let gas_cost = current_gas_price * U256::from(gas);

    if parsed_amount_u256 > (balance_eth_scrooll - gas_cost) {
        let scaled_value = (balance_eth_scrooll - gas_cost).low_u64() as f64 * 0.9;
        parsed_amount_u256 = U256::from_dec_str(&(scaled_value.round().to_string())).expect("Failed to convert f64 to U256");
    }

    if parsed_amount_u256 <= U256::zero() {
        return Err("Low balance".into());
    }

    let data0 = uniswap_router_parsed_abi.function("exactInputSingle")
        .expect("ExactInputSingle function not found in ABI")
        .encode_input(&[Token::Tuple(vec![
            Token::Address(eth_scroll_sepolia),
            Token::Address(gho_scroll_sepolia),
            Token::Uint(U256::from(3000)),
            Token::Address(address),
            Token::Uint(parsed_amount_u256),
            Token::Uint(U256::from(500)),
            Token::Uint(U256::zero()),
        ])])
        .expect("Failed to encode input");

    // println!("data0: {:?}", data0);

    let multicall_function = uniswap_router_parsed_abi.functions_by_name("multicall")
    .expect("multicall function not found in ABI")
    .into_iter()
    .find(|function| {
        function.inputs.len() == 2
        && matches!(function.inputs[0].kind, ethabi::ParamType::Uint(_))
        && matches!(function.inputs[1].kind, ethabi::ParamType::Array(_))
    })
    .expect("Specific multicall function overload not found");
    // println!("multicall_function: {:?}", multicall_function);

    let deadline = U256::from(Utc::now().timestamp() + 1200); // 20 minutes from now

    let data3 = multicall_function.encode_input(&[
        Token::Uint(U256::from(deadline)),
        Token::Array(vec![Token::Bytes(data0)]),
    ])?;
    // println!("data3: {:?}", data3);


    let gas_price: U256 = web3.eth().gas_price().await.expect("Failed to fetch gas price");
    // println!("gas_price: {:?}", gas_price);

    let nonce = web3.eth().transaction_count(address, None).await?;


    let txn_parameters = TransactionParameters {
        nonce: Some(nonce),
        to: Some(uniswap_router),
        value: U256::from(parsed_amount_u256),
        gas_price: Some(gas_price),
        gas: U256::from(500000),
        data: data3.clone().into(),
        chain_id: None,
        transaction_type: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };
    // println!("txn_parameters: {:?}", txn_parameters);

    let key_bytes = hex::decode(&private_key).expect("Failed to decode hex");
    let secret_key = SecretKey::from_slice(&key_bytes).expect("Invalid private key bytes");
    let signed_txn = web3.accounts().sign_transaction(txn_parameters, &secret_key).await?;

    let _ = sleep(Duration::from_secs(2));

    let tx_hash = web3.eth().send_raw_transaction(signed_txn.raw_transaction).await?;
    // println!("tx_hash: {:?}", tx_hash);

    match wait_until_tx_finished(&web3, tx_hash, 360).await {
            Ok((success, returned_tx_hash)) => {
                if success {
                    info!("| 0x{} | Transaction was successful! https://sepolia-blockscout.scroll.io/tx/{:?}", &address_str, returned_tx_hash);
                } else {
                    error!("| 0x{} |Transaction failed! https://sepolia-blockscout.scroll.io/tx/{:?}", &address_str, returned_tx_hash);
                }
            },
            Err(err) => error!("Error: {}", err),
        }

    Ok(())
}


async fn swap_token_for_eth(private_key: &str, address: &str, web3: &Web3<Http>) -> Result<(), Box<dyn std::error::Error>> {


    let address_str = if address.starts_with("0x") {
                &address[2..]
            } else {
                &address
            };

    let address: Address = address.parse().expect("Failed to parse Ethereum address");

    let uniswap_router: Address = UNISWAP_ROUTER.parse().expect("Failed to parse Ethereum address");
    let eth_scroll_sepolia: Address = ETH_SCROLL_SEPOLIA.parse().expect("Failed to parse Ethereum address");
    let gho_scroll_sepolia: Address = GHO_SCROLL_SEPOLIA.parse().expect("Failed to parse Ethereum address");

    let uniswap_router_abi_bytes: Vec<u8> = fs::read("abi/uniswap.json")?;
    let uniswap_router_parsed_abi: ethabi::Contract = ethabi::Contract::load(uniswap_router_abi_bytes .as_slice())?;

    let gho_token_abi_bytes: Vec<u8> = fs::read("abi/token_gho.json")?;
    let gho_token_parsed_abi: ethabi::Contract = ethabi::Contract::load(gho_token_abi_bytes .as_slice())?;
    let gho_token_contract  = Contract::new(web3.eth(), gho_scroll_sepolia, gho_token_parsed_abi.clone());


    let balance_gho: U256 = gho_token_contract.query("balanceOf", (address,), None, Default::default(), None).await?;
    let random_percentage_num = rand::thread_rng().gen_range(20..60);
    let random_balance_slice: U256 = balance_gho * U256::from(random_percentage_num) / U256::from(100);

    check_approved(&private_key, address, gho_scroll_sepolia, uniswap_router, &web3, gho_token_abi_bytes).await.expect("Not approved");

    let data0 = uniswap_router_parsed_abi.function("exactInputSingle")
        .expect("ExactInputSingle function not found in ABI")
        .encode_input(&[Token::Tuple(vec![
            Token::Address(gho_scroll_sepolia),
            Token::Address(eth_scroll_sepolia),
            Token::Uint(U256::from(3000)),
            Token::Address(address),
            Token::Uint(random_balance_slice),
            Token::Uint(U256::from(500)),
            Token::Uint(U256::zero()),
        ])])
        .expect("Failed to encode input");

    // println!("data0: {:?}", data0);

    let multicall_function = uniswap_router_parsed_abi.functions_by_name("multicall")
    .expect("multicall function not found in ABI")
    .into_iter()
    .find(|function| {
        function.inputs.len() == 2
        && matches!(function.inputs[0].kind, ethabi::ParamType::Uint(_))
        && matches!(function.inputs[1].kind, ethabi::ParamType::Array(_))
    })
    .expect("Specific multicall function overload not found");
    // println!("multicall_function: {:?}", multicall_function);

    let deadline = U256::from(Utc::now().timestamp() + 1200); // 20 minutes from now

    let data3 = multicall_function.encode_input(&[
        Token::Uint(U256::from(deadline)),
        Token::Array(vec![Token::Bytes(data0)]),
    ])?;
    // println!("data3: {:?}", data3);


    let gas_price: U256 = web3.eth().gas_price().await.expect("Failed to fetch gas price");
    // println!("gas_price: {:?}", gas_price);

    let nonce = web3.eth().transaction_count(address, None).await?;

    let txn_parameters = TransactionParameters {
        nonce: Some(nonce),
        to: Some(uniswap_router),
        value: U256::zero(),
        gas_price: Some(gas_price),
        gas: U256::from(500000),
        data: data3.clone().into(),
        chain_id: None,
        transaction_type: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };
    // println!("txn_parameters: {:?}", txn_parameters);

    let key_bytes = hex::decode(&private_key).expect("Failed to decode hex");
    let secret_key = SecretKey::from_slice(&key_bytes).expect("Invalid private key bytes");
    let signed_txn = web3.accounts().sign_transaction(txn_parameters, &secret_key).await?;

    let _ = sleep(Duration::from_secs(2));

    let tx_hash = web3.eth().send_raw_transaction(signed_txn.raw_transaction).await?;
    // println!("tx_hash: {:?}", tx_hash);

    match wait_until_tx_finished(&web3, tx_hash, 360).await {
            Ok((success, returned_tx_hash)) => {
                if success {
                    info!("| 0x{} | Transaction was successful! https://sepolia-blockscout.scroll.io/tx/{:?}", &address_str, returned_tx_hash);
                } else {
                    error!("| 0x{} | Transaction failed! https://sepolia-blockscout.scroll.io/tx/{:?}", &address_str, returned_tx_hash);
                }
            },
            Err(err) => error!("Error: {}", err),
        }

    Ok(())
}


async fn add_liquidity(private_key: &str, address: &str, web3: &Web3<Http>) -> Result<(), Box<dyn std::error::Error>> {

    let address_str = if address.starts_with("0x") {
                &address[2..]
            } else {
                &address
            };

    let address: Address = address.parse().expect("Failed to parse Ethereum address");

    let address_quoter: Address = ADDRESS_QUOTER.parse().expect("Failed to parse Ethereum address");
    let address_liquid: Address = ADDRESS_LIQUID.parse().expect("Failed to parse Ethereum address");
    let eth_scroll_sepolia: Address = ETH_SCROLL_SEPOLIA.parse().expect("Failed to parse Ethereum address");
    let gho_scroll_sepolia: Address = GHO_SCROLL_SEPOLIA.parse().expect("Failed to parse Ethereum address");

    let quoter_abi_bytes: Vec<u8> = fs::read("abi/quoter.json")?;
    let quoter_parsed_abi: ethabi::Contract = ethabi::Contract::load(quoter_abi_bytes .as_slice())?;
    let contract_quoter  = Contract::new(web3.eth(), address_quoter, quoter_parsed_abi.clone());

    let uniswap_liquid_abi_bytes: Vec<u8> = fs::read("abi/uniswap_liquid.json")?;
    let uniswap_liquid_parsed_abi: ethabi::Contract = ethabi::Contract::load(uniswap_liquid_abi_bytes .as_slice())?;
    let contract_uniswap_liquid  = Contract::new(web3.eth(), address_liquid, uniswap_liquid_parsed_abi.clone());

    let gho_token_abi_bytes: Vec<u8> = fs::read("abi/token_gho.json")?;
    let gho_token_parsed_abi: ethabi::Contract = ethabi::Contract::load(gho_token_abi_bytes .as_slice())?;
    let gho_token_contract  = Contract::new(web3.eth(), gho_scroll_sepolia, gho_token_parsed_abi.clone());


    let balance_gho: U256 = gho_token_contract.query("balanceOf", (address,), None, Default::default(), None).await?;
    let random_percentage_num = rand::thread_rng().gen_range(10..30);
    let random_balance_slice: U256 = balance_gho * U256::from(random_percentage_num) / U256::from(100);


    let params = Token::Tuple(vec![
        Token::Address(gho_scroll_sepolia),
        Token::Address(eth_scroll_sepolia),
        Token::Uint(U256::from(random_balance_slice)),
        Token::Uint(U256::from(500)),
        Token::Uint(U256::from(0))
    ]);

    let result: (U256, U256, U256, U256) = contract_quoter.query("quoteExactInputSingle", params, None, Default::default(), None).await?;
    let (amount_out_eth, _, _, _) = result;

    check_approved(&private_key, address, gho_scroll_sepolia, address_liquid, &web3, gho_token_abi_bytes).await.expect("Not approved");

    let deadline = U256::from(Utc::now().timestamp() + 10000);

    const TICK_SPACING: i32 = 60;
    const RANGE_WIDTH: i32 = 5 * TICK_SPACING;

    let random_start_tick = rand::thread_rng().gen_range(60000..71000);
    let tick_lower = (random_start_tick / TICK_SPACING) * TICK_SPACING;
    let tick_upper = tick_lower + RANGE_WIDTH;

    let mint_args = Token::Tuple(vec![
        Token::Address(eth_scroll_sepolia),
        Token::Address(gho_scroll_sepolia),
        Token::Uint(U256::from(3000)),
        Token::Int(tick_lower.into()),
        Token::Int(tick_upper.into()),
        Token::Uint(U256::from(random_balance_slice)),
        Token::Uint(U256::from(amount_out_eth)),
        Token::Uint(U256::zero()),
        Token::Uint(U256::zero()),
        Token::Address(address),
        Token::Uint(U256::from(deadline))
    ]);
    // println!("mint_args: {:?}", mint_args);

    let txn_data = contract_uniswap_liquid.abi().function("mint")
        .expect("mint function not found in ABI")
        .encode_input(&[mint_args])
        .expect("Failed to encode input");

    // println!("txn_data: {:?}", txn_data);
    let nonce = web3.eth().transaction_count(address, None).await?;
    let gas_price: U256 = web3.eth().gas_price().await.expect("Failed to fetch gas price");

    let extra_data = hex::decode("12210e8a").expect("Failed to decode extra data");

    let data0 = uniswap_liquid_parsed_abi.function("multicall")
            .expect("multicall function not found in ABI")
            .encode_input(&[
                Token::Array(vec![
                    Token::Bytes(txn_data.into()),
                    Token::Bytes(extra_data.into())
                ]),
            ])
            .expect("Failed to encode input");

    let txn_parameters = TransactionParameters {
        nonce: Some(nonce),
        to: Some(address_liquid),
        value: amount_out_eth,
        gas_price: Some(gas_price),
        gas: U256::from(1000000),
        data: data0.into(),
        chain_id: None,
        transaction_type: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };
    // println!("txn_parameters: {:?}", txn_parameters);

    let key_bytes = hex::decode(&private_key).expect("Failed to decode hex");
    let secret_key = SecretKey::from_slice(&key_bytes).expect("Invalid private key bytes");
    let signed_txn = web3.accounts().sign_transaction(txn_parameters, &secret_key).await?;

    let _ = sleep(Duration::from_secs(2));

    let tx_hash = web3.eth().send_raw_transaction(signed_txn.raw_transaction).await?;
    // println!("tx_hash: {:?}", tx_hash);

    match wait_until_tx_finished(&web3, tx_hash, 360).await {
            Ok((success, returned_tx_hash)) => {
                if success {
                    info!("| 0x{} | Transaction was successful! https://sepolia-blockscout.scroll.io/tx/{:?}", &address_str, returned_tx_hash);
                } else {
                    error!("| 0x{} | Transaction failed! https://sepolia-blockscout.scroll.io/tx/{:?}", &address_str, returned_tx_hash);
                }
            },
            Err(err) => error!("Error: {}", err),
        }

    Ok(())
}



async fn check_gas_price(config: &Config) {

    let gas_tracker_wei = &config.settings.gas_tracker * 10u64.pow(9);

    // Initialize the web3 instance
    let transport = Http::new(ETH_RPC).expect("Failed to create HTTP transport");
    let web3 = Web3::new(transport);

    let mut current_gas_price = web3.eth().gas_price().await.expect("Failed to fetch gas price");
    let mut current_gas_price_gwei: f64 = current_gas_price.as_u64() as f64 / 10u64.pow(9) as f64;
    println!("GAS_TRACKER: {} Gwei. Current gas price: {:.2} Gwei", &config.settings.gas_tracker, current_gas_price_gwei);

    if current_gas_price > gas_tracker_wei.into() {
        println!("Gas is above the value of {} Gwei. Waiting for gas to decrease. Current gas price: {:.2} Gwei", &config.settings.gas_tracker, current_gas_price_gwei);

        while current_gas_price > gas_tracker_wei.into() {
            // Wait for 120 seconds
            tokio::time::sleep(Duration::from_secs(120)).await;
            current_gas_price = web3.eth().gas_price().await.expect("Failed to fetch gas price");
            current_gas_price_gwei = current_gas_price.as_u64() as f64 / 10u64.pow(9) as f64;

            println!("Gas still exceeds the value of {} Gwei. We are waiting for the gas price to decrease. Current gas price: {:.2} Gwei", &config.settings.gas_tracker, current_gas_price_gwei);
        }

        println!("Gas is below the mark of {} Gwei. Continuing operation", &config.settings.gas_tracker);
    }
}

fn format_ether_to_float(value: &U256) -> f64 {
    value.as_u128() as f64 / 1_000_000_000_000_000_000.0
}

pub fn generate_web3_clients(config: &Config, client: Client) -> (Web3<Http>, Web3<Http>, Web3<Http>, Web3<Http>) {
    let optimism_http = Http::with_client(client.clone(), (&config.rpc.optimism).parse().unwrap());
    let web3_opt = Web3::new(optimism_http);

    let arbitrum_http = Http::with_client(client.clone(), (&config.rpc.arbitrum).parse().unwrap());
    let web3_arb = Web3::new(arbitrum_http);

    let sepolia_http = Http::with_client(client.clone(), (&config.rpc.sepolia).parse().unwrap());
    let web3_sep = Web3::new(sepolia_http);

    let scroll_sep_http = Http::with_client(client.clone(), (&config.rpc.scroll_sepolia).parse().unwrap());
    let web3_scr_sep = Web3::new(scroll_sep_http);

    (web3_opt, web3_arb, web3_sep, web3_scr_sep)
}

fn random_reps(range: (usize, usize)) -> usize {
    let (min, max) = range;
    rand::thread_rng().gen_range(min..=max)
}

async fn random_delay(range: (u64, u64)) {
    let (min, max) = range;
    let delay_duration = rand::thread_rng().gen_range(min..=max);
    tokio::time::sleep(tokio::time::Duration::from_secs(delay_duration)).await;
}

async fn check_and_log_balance(web3: &Web3<Http>, address: &str, network_name: &str) {
    match check_balance(web3, address).await {
        Ok(balance) => {
            info!("| {} | {}", address, format!(
                    "Balance: {} {}",
                    format_ether_to_float(&balance).to_string(),
                    network_name.to_string()));
        },
        Err(e) => {
            eprintln!("Failed to check balance on {}: {}", network_name, e);
        }
    }
}

async fn check_balance(web3: &Web3<Http>, address: &str) -> web3::Result<U256> {
    match address.parse::<Address>() {
        Ok(address_h160) => web3.eth().balance(address_h160, None).await,
        Err(_) => {
            println!("Failed to parse address: {}", address);
            Err(web3::Error::InvalidResponse("Failed to parse address".into()))
        }
    }
}

async fn wait_for_tx_receipt(web3: &Web3<Http>, tx_hash: web3::types::H256, max_wait_secs: u64, poll_interval_secs: u64) -> Option<TransactionReceipt> {
    let start_time = Instant::now();
    let max_wait_time = Duration::from_secs(max_wait_secs);

    while start_time.elapsed() < max_wait_time {
        if let Ok(Some(tx_receipt)) = web3.eth().transaction_receipt(tx_hash).await {
            return Some(tx_receipt);
        }
        // Wait for a short duration before polling again
        sleep(Duration::from_secs(poll_interval_secs.clone())).await;
    }
    None
}

async fn wait_until_tx_finished(web3: &Web3<Http>, tx_hash: web3::types::H256, max_wait_secs: u64) -> Result<(bool, web3::types::H256), &'static str> {
    let start_time = Instant::now();
    let max_wait_time = Duration::from_secs(max_wait_secs);

    while start_time.elapsed() < max_wait_time {
        match web3.eth().transaction_receipt(tx_hash).await {
            Ok(Some(receipt)) => {
                let one = U64::from(1);
                match receipt.status {
                    Some(status) if status == one => {
                        // println!("Transaction was successful! {:?}", tx_hash);
                        return Ok((true, tx_hash));
                    },
                    Some(_) => {
                        println!("Transaction failed! {:?}", receipt);
                        return Ok((false, tx_hash));
                    },
                    None => {
                        tokio::time::sleep(Duration::from_millis(300)).await;
                    },
                }
            },
            Ok(None) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            },
            Err(_) => {
                if start_time.elapsed() > max_wait_time {
                    println!("FAILED TX: {:?}", tx_hash);
                    return Ok((false, tx_hash));
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
    Err("Reached maximum wait time without transaction confirmation.")
}

async fn get_transaction_data(web3: &Web3<Http>, tx_hash: web3::types::H256) -> web3::Result<web3::types::Transaction> {
                        //    Example:
    // let tx_hash_str1 = "0x32693ebe29bb2eb6be3f380ddb881d2d95d723cc601424cfe289970e6d2ff808";
    // let tx_hash1 = web3::types::H256::from_slice(&hex::decode(&tx_hash_str1[2..]).expect("Failed to decode hex"));
    //
    // // Get transaction data
    // match get_transaction_data(&web3, tx_hash1).await {
    //     Ok(tx_data) => {
    //         println!("Transaction: {:?}", tx_data);
    //     }
    //     Err(e) => {
    //         println!("Error fetching transaction data: {:?}", e);
    //     }
    // }

    let result = web3.eth().transaction(web3::types::TransactionId::Hash(tx_hash)).await?;
    match result {
        Some(tx_data) => Ok(tx_data),
        None => {
            let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "Transaction not found");
            Err(web3::Error::Transport(web3::error::TransportError::Message(io_err.to_string())))
        }
    }
}

async fn check_stargate(hash_: &str, client: Client) -> Result<bool, reqwest::Error> {
    let url = format!("https://api-mainnet.layerzero-scan.com/tx/{}", hash_);
    let res: Value = client.get(&url).send().await?.json().await?;

    if let Some(messages) = res["messages"].as_array() {
        if messages.is_empty() || messages[0]["status"] != "DELIVERED" {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }
    Ok(true)
}

async fn wait_for_stargate_completion(hash_: &str, client: Client) {
    loop {
        match check_stargate(hash_, client.clone()).await {
            Ok(true) => {
                println!("Bridge is not yet complete...");
                tokio::time::sleep(tokio::time::Duration::from_secs(50)).await;
            },
            Ok(false) => {
                println!("Bridge has completed!");
                break;
            },
            Err(e) => {
                eprintln!("Error while checking: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    }
}

async fn check_status_bridge(tx_hash: &str, client: &Client) -> Result<bool, reqwest::Error> {
    let url = "https://sepolia-api-bridge.scroll.io/api/txsbyhashes";
    let json_data = json!({
        "txs": [tx_hash]
    });

    let response: Value = client.post(url).json(&json_data).send().await?.json().await?;

    if let Some(result) = response["data"]["result"].as_array() {
    if !result.is_empty() && result[0].is_object() {
        // println!("response: {}", response);
        if let Some(finalize_tx) = result[0]["finalizeTx"].as_object() {
            if finalize_tx["blockNumber"] != 0 {
                return Ok(false);
            } else {
                return Ok(true);
            }
        }
    }
}

    Ok(true)
}

async fn wait_for_bridge_completion(tx_hash: &str, client: Client) {
    loop {
        match check_status_bridge(tx_hash, &client).await {
            Ok(true) => {
                println!("Bridge is not yet complete...");
                tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
            },
            Ok(false) => {
                println!("Bridge has completed!");
                break;
            },
            Err(e) => {
                eprintln!("Error while checking: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    }
}

async fn check_approved(
    private_key: &str,
    wallet_address: H160,
    contract_address: H160,
    router_address: H160,
    web3: &Web3<Http>,
    contract_token_abi: Vec<u8>
) -> web3::Result<()> {

    let contract_token_parsed_abi: ethabi::Contract = ethabi::Contract::load(contract_token_abi .as_slice())
        .map_err(|e| web3::Error::Transport(web3::error::TransportError::Message(format!("{:?}", e))))?;
    let contract  = Contract::new(web3.eth(), contract_address, contract_token_parsed_abi.clone());

    // Check current allowance
    let current_allowance: U256 = contract.query("allowance", (wallet_address, router_address), None, Options::default(), None)
        .await
        .map_err(|e| web3::Error::Transport(web3::error::TransportError::Message(format!("{:?}", e))))?;

    // info!("Current allowance: {:?}", current_allowance);

    let balance_gho: U256 = contract.query("balanceOf", (wallet_address,), None, Default::default(), None)
        .await
        .map_err(|e| web3::Error::Transport(web3::error::TransportError::Message(format!("{:?}", e))))?;

    if current_allowance < balance_gho {
        send_approval(private_key, wallet_address, contract_address, router_address, web3, contract_token_abi).await?;
    } else {
        info!("Token approval is sufficient.");
    }

    Ok(())
}

async fn send_approval(
    private_key: &str,
    wallet_address: H160,
    contract_address: H160,
    router_address: H160,
    web3: &Web3<Http>,
    contract_token_abi: Vec<u8>
) -> web3::Result<()> {
    let max_amount = U256::max_value();
    let nonce = web3.eth().transaction_count(wallet_address, None).await?;
    let gas_price: U256 = web3.eth().gas_price().await?;

    let contract_token_parsed_abi: ethabi::Contract = ethabi::Contract::load(contract_token_abi .as_slice())
        .map_err(|e| web3::Error::Transport(web3::error::TransportError::Message(format!("{:?}", e))))?;

    let data = contract_token_parsed_abi.function("approve")
        .expect("approve function not found in ABI")
        .encode_input(&[Token::Address(router_address), Token::Uint(max_amount)])
        .expect("Failed to encode input");

    let txn_parameters = TransactionParameters {
        nonce: Some(nonce),
        to: Some(contract_address),
        value: U256::zero(),
        gas_price: Some(gas_price),
        gas: U256::from(500000),
        data: data.into(),
        ..Default::default()
    };

    let key_bytes = hex::decode(&private_key).expect("Failed to decode hex");
    let secret_key = SecretKey::from_slice(&key_bytes).expect("Invalid private key bytes");
    let signed_txn = web3.accounts().sign_transaction(txn_parameters, &secret_key).await?;

    sleep(Duration::from_secs(2)).await;

    let tx_hash = web3.eth().send_raw_transaction(signed_txn.raw_transaction).await?;
    info!("Sent approval transaction, tx_hash: {:?}", tx_hash);

    match wait_until_tx_finished(&web3, tx_hash, 360).await {
        Ok((success, _returned_tx_hash)) => {
            if success {
                info!("| {} | Approved - OK", &wallet_address);
            } else {
                error!("| {} | Approved - Error", &wallet_address);
            }
        },
        Err(err) => {
            error!("Error: {}", err);
            return Err(web3::Error::Transport(web3::error::TransportError::Message(format!("Failed to send approval: {:?}", err))));
        },
    }

    Ok(())
}

