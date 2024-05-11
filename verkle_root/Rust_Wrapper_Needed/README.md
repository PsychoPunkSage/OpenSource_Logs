# Required RUST wrapper of Methods for Verkle

## Elliptic Curve API:

> Need to define a Elliptic curve `𝐸` over a base field `𝐹𝑝` with a scalar field `𝐹𝑟`<br>
> The group exposed by 𝐸(𝐹𝑝) must have prime order. This is so that the verkle trie logic does not need to worry about subgroup attack vectors.