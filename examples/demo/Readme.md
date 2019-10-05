Develop with:
```shell script
cargo watch -s "wasm-pack build --target web && rollup ./main.js --format iife --file ./pkg/bundle.js && python -m SimpleHTTPServer 8080"
```

Build with:
```shell script
wasm-pack build --release --target web && rollup ./main.js --format iife --file ./pkg/bundle.js 
```
