# Node integration tests

The main purpose of these tests is to test if every exported function
successfully return correct value and if it throws in case of an error.
We do not need to test actual functionality as it is tested in the Rust code.

## Run tests

* Build isomorphic NPM package with `scripts/build-wasm.sh` script
* Install dependencies with `npm install`
* Run tests with `npm test`
