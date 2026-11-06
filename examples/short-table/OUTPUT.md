# The short table, measured

Written by `cargo run --manifest-path examples/short-table/Cargo.toml`, from this checkout of
float-abel. The neighbours are the versions this package's Cargo.lock pins: bitrep 0.5.1,
xsum 0.1.6, fsum 0.1.4, accurate 0.4.1, num-rational 0.4.2, bigdecimal 0.4.11 and fixed 1.31.0.

| | float-abel | bitrep | xsum | fsum | accurate | num-rational | bigdecimal | fixed |
|---|---|---|---|---|---|---|---|---|
| holds every f64 exactly | yes | yes | yes | yes | yes | yes | yes | no |

## Evidence

### holds every f64 exactly

The weight carries the column's value, so a carrier that loses any of it makes the `SUM` wrong.
Seven edge doubles, 0.1, 1e-20, 1e30, 1e300, f64::MAX, f64::MIN_POSITIVE, 5e-324: each is read back
alone, and each is entered with the double just below it retracted, which must leave exactly their
gap, one ulp and itself a double. The second check catches a carrier that rounds on entry and rounds
back the same way on read, which the first alone cannot see. Every comparison is of bits. Zero stays
out, because the group's zero is +0.0 and the sign of zero is not a value; NaN and ±∞ have their own
row.

| column | cell | evidence |
|---|---|---|
| float-abel | yes | ran: all 7 values and their 7 one-ulp gaps read back bit for bit |
| bitrep | yes | ran: all 7 values and their 7 one-ulp gaps read back bit for bit |
| xsum | yes | ran: all 7 values and their 7 one-ulp gaps read back bit for bit |
| fsum | yes | ran: all 7 values and their 7 one-ulp gaps read back bit for bit |
| accurate | yes | ran: all 7 values and their 7 one-ulp gaps read back bit for bit |
| num-rational | yes | ran: all 7 values and their 7 one-ulp gaps read back bit for bit |
| bigdecimal | yes | ran: all 7 values and their 7 one-ulp gaps read back bit for bit |
| fixed | no | ran: +1e-20 reads 0.0; ran: +1e-20 −9.999999999999998e-21 reads 0.0, want 1.504632769052528e-36; refused: I64F64::checked_from_num(1e30) is None; refused: I64F64::checked_from_num(1e300) is None; refused: I64F64::checked_from_num(f64::MAX) is None; ran: +f64::MIN_POSITIVE reads 0.0; ran: +f64::MIN_POSITIVE −2.225073858507201e-308 reads 0.0, want 5e-324; ran: +5e-324 reads 0.0; ran: +5e-324 −0.0 reads 0.0, want 5e-324 |
