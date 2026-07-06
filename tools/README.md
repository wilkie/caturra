# tools

`FloatCorpus.java` regenerates the `Float.toString` reference corpus used by
the env-gated test in `crates/caturra-vm/src/floatdec.rs`:

```sh
javac FloatCorpus.java && java FloatCorpus > corpus.txt
CATURRA_FLOAT_CORPUS=$PWD/corpus.txt cargo test -p caturra-vm floatdec
```

The renderer matches OpenJDK 11 on 99.94% of the corpus; the residue is
exotic subnormal bit patterns documented in the test.
