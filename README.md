# riscv-simulator-rust
RISC-V instruction set simulator written in Rust language.  
Currently only RV32I instructions are supported.  

## Test
All implemented instructions are tested by [riscv-tests](https://github.com/riscv/riscv-tests)  
```
git clone git@github.com:kura197/riscv-simulator-rust.git
cd riscv-simulator-rust
cargo build
TEST_DIR=/path/to/riscv-tests ./test.sh
```
