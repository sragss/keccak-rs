# keccak-rs
Created to learn keccak. Currently about half the speed of `tiny_keccak`.

Does not include most of the sponge construction as we only allow a 32byte output.

# notes
- Ethereum's keccak256 does not match sha3_256. The only difference being the delimited suffix for 256 is 0x01 rather than 0x06

# resources
- [Solidity impl](https://github.com/SmartPool/contracts/blob/develop/contracts/Ethash.sol)
- [C impl](https://github.com/mjosaarinen/tiny_sha3)
- [Python impl](https://github.com/XKCP/XKCP/blob/master/Standalone/CompactFIPS202/Python/CompactFIPS202.py) (included in `./keccak.py`)
- [Bellman impl](https://github.com/zatoichi-labs/bellman-keccak256/blob/master/src/lib.rs)
- [Spec summary](https://keccak.team/keccak_specs_summary.html)

# future possibilities
- unroll loops
- compile to wasm, replace keccak256 npm package (buffer-less)

# cmds
- `cargo build`
- `cargo test`
- `cargo bench`
- `cargo flamegraph --bench speed -- --bench`