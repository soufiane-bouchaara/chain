 cargo run --release --features=runtime-benchmarks -- benchmark --chain dev  --execution=wasm --extrinsic="*" --pallet=ternoa_altvr --steps=50 --repeat=20 --heap-pages=4096 --output .
