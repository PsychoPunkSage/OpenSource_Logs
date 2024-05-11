# Required RUST wrapper of Methods for Verkle

## Elliptic Curve API:

>> Need to define a Elliptic curve `𝐸` over a base field `𝐹𝑝` with a scalar field `𝐹𝑟`<br>
>> The group exposed by 𝐸(𝐹𝑝) must have prime order. This is so that the verkle trie logic does not need to worry about subgroup attack vectors.

### Algorithms Required:

**Serialize**: This algorithm takes in a group element as input and returns a unique encoding of the group element as a byte string.

**MapToFieldBytes**: This algorithm takes in a group element as input and maps the group element to the base field 𝐹𝑝. The output is a unique encoding of the field element in 𝐹𝑝 as a byte string.

> MapToFieldBytes returns a byte string so that the verkle trie library does not need to be concerned with 𝐹𝑝, only 𝐹𝑟 is exposed through the API.

## MultiPoint Scheme API:

### Algorithms Required:

**Prove**: This algorithm takes in as input a list of tuples of the form (𝐶𝑖,𝑓𝑖(𝑋),𝑧𝑖,𝑦𝑖) and produces a proof 𝜋 that each 𝑓𝑖(𝑧𝑖)=𝑦𝑖.

𝐶𝑖 is the commitment to the function 𝑓𝑖(𝑋) and is produced by using the Commit algorithm.

**Commit**: This algorithm takes as input a function 𝑓(𝑋) and produces a commitment to that function 𝐶.

**Verify**: This algorithm takes as input a proof 𝜋, and a list of tuples 𝐶𝑖,𝑧𝑖,𝑦𝑖. The output is true, if the proof can attest to the fact that, for all 𝑖, 𝐶𝑖 commits to a function 𝑓𝑖(𝑋) using Commit and 𝑓𝑖(𝑧𝑖)=𝑦𝑖.