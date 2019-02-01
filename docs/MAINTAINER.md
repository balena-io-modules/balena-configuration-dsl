
## Hello and welcome to CDSL, maintainer !

This talks about how to run this project locally and how to make changes to it.
 
You can just run `./ci/install.sh` to get all dependencies going. This will install Rust and Node via NVM - please review the script before running, to see if this is what you want on your machine.

To make sure everything is working run `./ci/test.sh` - this will launch all the Rust tests and then package npm package and run node tests on it.

 
## Fuzzing
Fuzz tests are not included in the CI runs as they typically take a long time to run and the timing is quite unpredictable.  
We're using pretty standard [cargo fuzz] configuration with seeds here, based on [libFuzzer].  
To run the default fuzzer configuration do `./fuzz/run-fuzzer.sh`. 
Warning: running the fuzzer via this script will set your rustup environment override to nightly.
 
This will use seed jellyschema files from `fuzz/seeds` to jumpstart the process.  
You can cancel the fuzzing at any time by pressing `ctrl-c`. If you then run the fuzzer again, it will start where it left off - the temporary state is saved to `fuzz/corpus`  

### Fuzzing targets
It is only possible to fuzz one target at the time, the default target is `any_input` - that tests whether random input will crash the CDSL (make it panic).
TODO: add a fuzzing target where we make sure there is no `Err` when the yaml validates as jellyschema.

[cargo fuzz]: https://fuzz.rs/book/cargo-fuzz/guide.html
[libFuzzer]: https://llvm.org/docs/LibFuzzer.html