# Valkyrie Contracts
This mono repository contains the source code for the smart contracts implementing Valkyrie Protocol on the Terra blockchain.

You can find information about the architecture, usage, and function of the smart contracts on the official Valkyrie Protocol documentation [site](https://docs.valkyrieprotocol.com).

### Dependencies
Valkyrie Protocol depends on Terraswap and uses its implementation of the CW20 token specification.

## Contracts
| Contract                                            | Reference                                              | Description                                                                                                                        |
| --------------------------------------------------- | ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------- |
| [`valkyrie_campaign`](./contracts/campaign)  | [doc](https://docs.valkyrieprotocol.com/campaign) | Implementation of Valkyrie Protocol campaign                                             |
| [`valkyrie_campaign_manager`](../contracts/campaign_manager) | [doc](https://docs.valkyrieprotocol.com/campaign-manager) | Managing global configuration for campaign and creating campaign                                                                                                   |
| [`valkyrie_community`](./contracts/community)      | [doc](https://docs.valkyrieprotocol.com/community)   | Manages the commuinty pool fund                                                       |
| [`valkyrie_distributor`](./contracts/distributor)              |  | Manages the governance staking reward fund |
| [`valkyrie_governance`](./contracts/governance)            | [doc](https://docs.valkyrieprotocol.com/governance)      | Allows other Valkyrie contracts to be controlled by decentralized governance, distributes VKR received from Distributor to VKR stakers                                                                                   |
| [`valkyrie_lp_staking`](./contracts/lp_staking)        | [doc](https://docs.valkyrieprotocol.com/staking)    | Distributes VKR rewards from block reward to LP stakers                                                                   |

## Development

### Environment Setup

- Rust v1.53.1+
- `wasm32-unknown-unknown` target
- Docker

1. Install `rustup` via https://rustup.rs/

2. Run the following:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

3. Make sure [Docker](https://www.docker.com/) is installed

### Unit / Integration Tests

Each contract contains Rust unit tests embedded within the contract source directories. You can run:

```sh
cargo unit-test
```

### Compiling

After making sure tests pass, you can compile each contract with the following:

```sh
RUSTFLAGS='-C link-arg=-s' cargo wasm
cp ../../target/wasm32-unknown-unknown/release/cw1_subkeys.wasm .
ls -l cw1_subkeys.wasm
sha256sum cw1_subkeys.wasm
```

#### Production

For production builds, run the following:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.11.5
```

This performs several optimizations which can significantly reduce the final size of the contract binaries, which will be available inside the `artifacts/` directory.

## License

Copyright 2021 Valkyrie Protocol

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0. Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

See the License for the specific language governing permissions and limitations under the License.