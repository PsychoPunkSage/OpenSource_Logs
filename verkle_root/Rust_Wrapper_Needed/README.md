# Required RUST wrapper of Methods for Verkle

## Elliptic Curve API:

> Need to define a Elliptic curve `𝐸` over a base field `𝐹𝑝` with a scalar field `𝐹𝑟`<br>
> The group exposed by 𝐸(𝐹𝑝) must have prime order. This is so that the verkle trie logic does not need to worry about subgroup attack vectors.

#### Algorithms Required:

> **Serialize**: This algorithm takes in a group element as input and returns a unique encoding of the group element as a byte string.
> **MapToFieldBytes**: This algorithm takes in a group element as input and maps the group element to the base field 𝐹𝑝. The output is a unique encoding of the field element in 𝐹𝑝 as a byte string.