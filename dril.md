| what    | how much |
|---------|----------|
| salary  |     4000 |
| food    |     -200 |
| rent    |     -800 |
| candles |    -3600 |
| utility |     -150 |
: total = (t, r, i) => parseInt(r["how much"]) + ((i === 0)? 0 : parseInt(t[i - 1]["total"]))

