use toml::Value;
use std::fs;

// Structure to represent the configuration
#[derive(Clone)]
pub struct Config {
    pub rpc: RPC,
    pub threads: Threads,
    pub settings: Settings,
}

#[derive(Clone)]
pub struct RPC {
    pub optimism: String,
    pub arbitrum: String,
    pub sepolia: String,
    pub scroll_sepolia: String,
}

#[derive(Clone)]
pub struct Threads {
    pub number_of_threads: u32,
    pub delay_between_threads: (u64, u64),
}

#[derive(Clone)]
pub struct Settings {
    pub gas_tracker: u64,
    pub cap_key: String,
    pub delay_action: (u64, u64),

    pub sepolia_eth_min: f64,
    pub sepolia_eth_max: f64,
    pub sepolia_eth_decimal: i32,

    pub deposit_from_sepolia_to_scroll: f64,
    pub fees: f64,

    pub value_swap_min: f64,
    pub value_swap_max: f64,
    pub value_swap_decimal: i32,

    // New fields for enabling/disabling functions
    pub execute_get_faucet: bool,
    pub execute_get_gas_sepolia: bool,
    pub execute_bridge_from_sepolia_to_scroll: bool,
    pub execute_swap_eth_for_token: bool,
    pub execute_swap_token_for_eth: bool,
    pub execute_add_liquidity: bool,

    // Repetition counts
    pub swap_eth_for_token_reps: (usize, usize),
    pub swap_token_for_eth_reps: (usize, usize),
    pub add_liquidity_reps: (usize, usize),

}

pub fn read_config(path: &str) -> Result<Config, std::io::Error> {
    let content = fs::read_to_string(path)?;
    let value: Value = content.parse().expect("Failed to parse TOML");

    Ok(Config {
        rpc: RPC {
            optimism: value["RPC"]["optimism"].as_str().unwrap().to_string(),
            arbitrum: value["RPC"]["arbitrum"].as_str().unwrap().to_string(),
            sepolia: value["RPC"]["sepolia"].as_str().unwrap().to_string(),
            scroll_sepolia: value["RPC"]["scroll_sepolia"].as_str().unwrap().to_string(),
        },
        threads: Threads {
            number_of_threads: value["threads"]["number_of_threads"].as_integer().unwrap() as u32,
            delay_between_threads: (
                value["settings"]["delay_action"][0].as_integer().unwrap() as u64,
                value["settings"]["delay_action"][1].as_integer().unwrap() as u64,
            ),
        },
        settings: Settings {
            gas_tracker: value["settings"]["gas_tracker"].as_integer().unwrap() as u64,
            cap_key: value["settings"]["cap_key"].as_str().unwrap().to_string(),
            delay_action: (
                value["settings"]["delay_action"][0].as_integer().unwrap() as u64,
                value["settings"]["delay_action"][1].as_integer().unwrap() as u64,
            ),

            sepolia_eth_min: value["settings"]["sepolia_eth_min"].as_float().unwrap(),
            sepolia_eth_max: value["settings"]["sepolia_eth_max"].as_float().unwrap(),
            sepolia_eth_decimal: value["settings"]["sepolia_eth_decimal"].as_integer().unwrap() as i32,

            deposit_from_sepolia_to_scroll: value["settings"]["deposit_from_sepolia_to_scroll"].as_float().unwrap(),
            fees: value["settings"]["fees"].as_float().unwrap(),

            value_swap_min: value["settings"]["value_swap_min"].as_float().unwrap(),
            value_swap_max: value["settings"]["value_swap_max"].as_float().unwrap(),
            value_swap_decimal: value["settings"]["value_swap_decimal"].as_integer().unwrap() as i32,

            execute_get_faucet: value["settings"]["execute_get_faucet"].as_bool().unwrap(),
            execute_get_gas_sepolia: value["settings"]["execute_get_gas_sepolia"].as_bool().unwrap(),
            execute_bridge_from_sepolia_to_scroll: value["settings"]["execute_bridge_from_sepolia_to_scroll"].as_bool().unwrap(),
            execute_swap_eth_for_token: value["settings"]["execute_swap_eth_for_token"].as_bool().unwrap(),
            execute_swap_token_for_eth: value["settings"]["execute_swap_token_for_eth"].as_bool().unwrap(),
            execute_add_liquidity: value["settings"]["execute_add_liquidity"].as_bool().unwrap(),

            swap_eth_for_token_reps: {
                let arr = value["settings"]["swap_token_for_eth_reps"].as_array().unwrap();
                (arr[0].as_integer().unwrap() as usize, arr[1].as_integer().unwrap() as usize)
            },
            swap_token_for_eth_reps: {
                let arr = value["settings"]["swap_token_for_eth_reps"].as_array().unwrap();
                (arr[0].as_integer().unwrap() as usize, arr[1].as_integer().unwrap() as usize)
            },
            add_liquidity_reps: {
                let arr = value["settings"]["add_liquidity_reps"].as_array().unwrap();
                (arr[0].as_integer().unwrap() as usize, arr[1].as_integer().unwrap() as usize)
            },
        },
    })
}


