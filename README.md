# Paper Mario Randomizer

#### Running

`paper-mario-randomizer` is a command line program. For more usage directions use the `--help` flag.

For example on Windows:
```
paper-mario-randomizer.exe --help
``` 

____

#### Building

To build your own copy of the executable you'll need to [install rust](https://doc.rust-lang.org/book/ch01-01-installation.html).

Once that's done running `cargo build --release` in the project root will download some more packages and then produce an executable in the `/target/release` directory.  
____

This project began as a port of https://github.com/MrCheeze/paper-mario-randomizer

This project also owes a debt to the information collected together in [the paper mario hacking archive](http://papermariohackingarchive.com/)

In particular [the hex editing page](http://papermariohackingarchive.com/downloads/PM64_Hex.txt) by CrashingThunder was referenced several times.

Released under the WTFPL see [LICENSE](./LICENSE).