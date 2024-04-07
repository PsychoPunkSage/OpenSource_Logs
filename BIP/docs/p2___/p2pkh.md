## p2pkh txn

### Requirements

> **Initial Input Check**
 
This ensures at least one of the essential pieces of information (`address`, `hash`, `output`, `pubkey`, or `input`) is provided.

```python

```

> **Type Checking**

This uses a type-checking library (typeforce) to verify that the provided data matches expected types (e.g., address should be a string, hash should be a 20-byte Buffer).

```python

```

> **Extended Validation**

in-depth validation is performed here.

`address`:
- Verifies the version against the network's pubKeyHash.
- Ensures the address length is correct (21 bytes).
- Extracts the hash from the address and stores it in the hash variable.

`hash`:
- Compares it with the hash extracted from the address (if available).
- Updates the hash variable for further checks.

`output`:
- Ensures the output script has the expected format (`OP_DUP`, `OP_HASH160`, `hash`, `OP_EQUALVERIFY`, `OP_CHECKSIG`) and correct lengths.
- Compares the extracted hash from the output script with the hash variable.

`pubkey`:
Calculates the public key hash (P2KH) from the provided pubkey and compares it with the hash variable.

`input`:
- Ensures the input script has a length of 2 (signature and pubkey).
- Uses helper functions to verify the signature format and pubkey validity.
- Compares the extracted signature and pubkey from the input with the provided signature and pubkey (if available).
- Recalculates the P2KH from the input pubkey and compares it with the hash variable.