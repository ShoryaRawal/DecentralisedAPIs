# Instructions to run

Make sure that you have enabled the wasm32 target with:

```
rustup target wasm32-unknown-unknown 
```

run the api using

```
dfx start --clean --background
dfx deploy (for local networks)
```

Use the endpoints using the commands:

```
curl "htpps://<canister_id>.raw.localhost:[port]/"
```
