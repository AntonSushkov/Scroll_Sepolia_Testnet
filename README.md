<h1 align="center">Scroll Sepolia Testnet Activity</h1>
<p align="left">
</p>

<p align="center">
A utility for automating activities on the Scroll Sepolia test network.
</p>

## Table of Contents
- [Overview](#overview)
- [Pre-launch Setup](#pre-launch-setup)
- [Configuration Parameters](#configuration-parameters)
- [Installation](#installation)
- [Usage](#usage)
- [Donation](#donation)

## Overview
This program facilitates automated activities on the Scroll Sepolia test network. By adjusting the parameters in the `Config.toml` file, users can customize various operations such as gas refueling, deposits, swaps, and more.

## Pre-launch Setup
Before launching the program, ensure the following files are correctly filled:

1. **FILEs/proxy.txt**: Fill in the proxies in the format `IP:PORT:USER:PASS`. Each proxy should be on a new line.
2. **FILEs/address_private_key.txt**: Fill in the `Address:PrivateKey` format. Each wallet should be on a new line.
 Ensure that the number of `Address:PrivateKey` pairs matches the number of proxies listed in the `proxy.txt` file.
3. **2Captcha API Key**: To use the `bwarelabs_faucet` function, specify your API keys from [2Captcha](https://2captcha.com/) in the `Config.toml` file.



## Configuration Parameters

### RPC URLs for Different Chains
- **Optimism (Upcoming)**: Planned for the next update.
- **Arbitrum**: `https://arbitrum-one.publicnode.com`
- **Sepolia**: `https://ethereum-sepolia.blockpi.network/v1/rpc/public`
- **Scroll Sepolia**: `https://sepolia-rpc.scroll.io`

### Thread Configurations
- **number_of_threads**: Total number of concurrent threads to be executed.
- **delay_between_threads**: Delay (in seconds) between the start of each thread, chosen randomly from the range.

### Global Settings
- **gas_tracker**: Threshold of gas amount below which swaps from Arbitrum to Sepolia won't be performed.
- **cap_key**: Necessary to enter API key from [2Captcha](https://2captcha.com/) service.
- **delay_action**: Delay (in seconds) between actions, chosen randomly from the range.

### Gas Refuel Settings (Arbitrum to Sepolia)
- **sepolia_eth_min**: Minimum ETH amount to be received to Sepolia.
- **sepolia_eth_max**: Maximum ETH amount to be received to Sepolia.
- **sepolia_eth_decimal**: Decimal precision for ETH amounts.

### Deposit Settings (Sepolia to Scroll Sepolia)
- **deposit_from_sepolia_to_scroll**: Percentage of balance in Sepolia to be bridged to Scroll Sepolia. It's not recommended to set this above 0.9 (90%).
- **fees**: Advised to set slightly above the fee on [Scroll Bridge](https://scroll.io/bridge).

### Uniswap Swap Settings
- **value_swap_min**: Minimum ETH amount for swapping to GHO tokens.
- **value_swap_max**: Maximum ETH amount for swapping to GHO tokens.
- **value_swap_decimal**: Decimal precision for ETH swap amounts.


### Module Execution Settings
Set the corresponding module to `true` to enable or `false` to disable.
- **execute_get_faucet**: Enable/Disable ETH fetching from [Scroll Sepolia faucet](https://bwarelabs.com/faucets/scroll-testnet) (once every 24 hours).
- **execute_get_gas_sepolia**: Enable/Disable sending from Arbitrum to Sepolia.
- **execute_bridge_from_sepolia_to_scroll**: Enable/Disable bridge from Sepolia to Scroll Sepolia.
- **execute_swap_eth_for_token**: Enable/Disable swapping of ETH for GHO tokens and the number of repetitions.
- **swap_eth_for_token_reps**: Number of repetitions for ETH-GHO token swap.
- **execute_swap_token_for_eth**: Enable/Disable swapping of GHO tokens for ETH and the number of repetitions.
- **swap_token_for_eth_reps**: Number of repetitions for GHO-ETH token swap.
- - **Swap GHO to ETH**: Swap will be from 20-60% of GHO token balance.
- **execute_add_liquidity**: Enable/Disable adding liquidity for the ETH-GHO pair on Uniswap and the number of repetitions.
- **add_liquidity_reps**: Number of repetitions to add liquidity.
- - **Pool Creation and Liquidity**: Creation and addition of liquidity will be for 10-30% of GHO tokens on the balance.

## Installation
1. Install Rust and Cargo using the instructions provided [here](https://www.rust-lang.org/learn/get-started).

## Usage
To launch the program, navigate to the project directory and run:
```bash
cargo run --release
```

## Donation:
```bash
0x0000002b721da5723238369e69e4c7cf48ca5f0c
```
- _only EVM_
