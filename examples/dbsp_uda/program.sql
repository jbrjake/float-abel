-- Feldera's sanctioned route to an exact SUM over doubles: a user-defined linear aggregate whose
-- accumulator is float_abel::Exact (main.rs beside this file holds the three Rust items udf.rs
-- needs). SQL's own SUM(DOUBLE) is fixed to f64 by the compiler and cannot be made linear from
-- outside, and `SUM` cannot be the name of a user-defined aggregate.
CREATE LINEAR AGGREGATE exact_sum(value DOUBLE) RETURNS DOUBLE;

CREATE TABLE ledger(account INT NOT NULL, amount DOUBLE NOT NULL);

CREATE MATERIALIZED VIEW totals AS
SELECT account, exact_sum(amount) AS total
FROM ledger
GROUP BY account;
