## No calls? No problem!
Rust framework for virtualizing calls and rets and encrypting their target offsets in a binary.
The build currently replaces all `0xcc` chains with random valid instructions, overwrites calls with `INT3` and rets with `ICEBP`. This leads to a binary that functions but breaks decompilers without further tooling.
This build creates TLS callback that registers our custom VEH, but you could just overwrite rust's default VEH by overwriting the target address to hide the jump handler.