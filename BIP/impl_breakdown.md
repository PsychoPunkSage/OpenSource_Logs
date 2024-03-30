## Code Breakdown

#### Fn to `Read txn` from mempool

> **AIM:** Go through the list of txns in `mempool` folder and then parse each json file an **return the list of Transactions**.

<details>
<summary>Template</summary>

```python
def read_transactions():
    transactions = []
    mempool_dir = "mempool"
    for filename in os.listdir(mempool_dir):
        with open(os.path.join(mempool_dir, filename), "r") as file:
            transaction_data = json.load(file)
            transactions.append(transaction_data)
    return transactions
```

</details><br>

#### Fn to `Validate txn` 

> **AIM:** Parse the list of Transactions returned by `Read txn fn` and then **return the list of valid Transactions** .

<details>
<summary>Template</summary>

```python
def validate_transactions(transactions):
    valid_transactions = []
    for transaction in transactions:
        # Add validation logic here
        valid_transactions.append(transaction)
    return valid_transactions
```

</details><br>

<details>
<summary>Pointers</summary>

>> **Vin**

> **p2pkh**

* **Validation**<br>
`OP_DUP`: Duplicates the top stack item.<br>
`OP_HASH160`: Hashes the top stack item using SHA-256 followed by RIPEMD-160.<br>
`OP_PUSHBYTES_20`: Pushes 20 bytes onto the stack.<br>
`OP_EQUALVERIFY`: Checks if the top two stack items are equal, then removes them from the stack.<br>
`OP_CHECKSIG`: Verifies the signature of the transaction input.<br>

> **v0_p2wpkh**

* **Validation**<br>
`OP_DUP`: Duplicates the top stack item.<br>
`OP_HASH160`: Hashes the top stack item using SHA-256 followed by RIPEMD-160.<br>
`OP_PUSHBYTES_20`: Pushes 20 bytes onto the stack.<br>
`OP_EQUALVERIFY`: Checks if the top two stack items are equal, then removes them from the stack.<br>
`OP_CHECKSIG`: Verifies the signature of the transaction input.<br>

* **Field Significance**
`txid`: The transaction ID uniquely identifies the transaction on the blockchain. To validate this field, one would typically check if the transaction ID is unique and corresponds to the transaction data provided.

`vout`: This field specifies the index of the output being spent by the input. It indicates which output of the previous transaction (specified by txid) is being spent. To validate, ensure that the referenced output exists in the previous transaction and has not already been spent.

`prevout`: This object contains information about the output being spent, including the script public key, its type, address, and value. To validate, ensure that the referenced output is valid, unspent, and matches the provided details.

`scriptsig`: This field contains the signature script for the input, which is used to unlock the output being spent. To validate, verify that the signature script is correctly formatted and can unlock the referenced output.

`scriptsig_asm`: This field provides the human-readable representation of the signature script. To validate, ensure that the signature script corresponds to the expected unlocking conditions for the referenced output.

`sequence`: This field specifies the relative locktime of the input. It determines when the transaction can be included in a block based on its age or block height. To validate, ensure that the sequence number meets the requirements set by the transaction's locktime.

>> **Vout**

`scriptpubkey`: This field contains the locking script, which defines the conditions under which the funds can be spent. It typically includes an output address or public key hash (for P2PKH or P2WPKH scripts), allowing the owner of the corresponding private key to unlock and spend the funds.

`scriptpubkey_asm`: This is the assembly representation of the locking script. It shows the individual operations (OP_CODES) performed by the script, such as OP_DUP, OP_HASH160, and OP_EQUALVERIFY, along with any associated data (e.g., public key hashes).

`scriptpubkey_type`: Indicates the type of locking script used. In the provided examples, "p2pkh" denotes Pay-to-Public-Key-Hash, indicating that the funds are locked using a public key hash.

`scriptpubkey_address`: This field represents the Bitcoin address associated with the locking script. It is a human-readable format derived from the locking script's hash, making it easier for users to send funds to and identify the recipient of the transaction.

`value`: Denotes the amount of Bitcoin (in satoshis) associated with each output. It represents the quantity of funds being transferred to the corresponding locking script/address.



</details><br>

> **Valid Txn details**

- no. of transactions::> `no limit` BUT the allowable size of block is 1MB. 

**1. Transaction Structure:**

* `Version`: Must be within a supported range (currently 1 or 2).
* `Locktime`: Specifies a block height or timestamp for transaction execution. Here this should be `0` as we are creating 1 block and we are not waiting for other block creation.
* `Inputs (vin)`:<br>
Transaction ID (txid): References previous transaction's output(s) being spent.<br>
Output Index (vout): Identifies which specific output from the previous transaction is being used.<br>
Unlocking Script (scriptSig or witness): Provides data to spend the referenced output, proving ownership of funds.<br>
Sequence Number: Typically set to maximum value (4294967295), indicating default priority.<br>
* `Outputs (vout)`:<br>
Value: Specifies the amount of Bitcoin being sent to each output.<br>
Locking Script (scriptPubkey): Defines conditions for spending the output in future transactions.<br>

**2. Input Validation:**

* `Unspent Output`: Each input must reference an unspent transaction output (UTXO) not yet spent.
* `Signature Validation`: The unlocking script must provide a valid digital signature that matches the output's locking script conditions.

**3. Output Validation:**

* `Value Total`: The sum of all output values cannot exceed the total value of inputs, conserving funds.
* `Locking Script Formats`: Outputs must use recognized locking script formats (e.g., p2pkh, p2wpkh).

**4. Transaction Size:**

* `Block Limit`: The transaction's size in bytes cannot exceed the current block size limit for inclusion in a block.

**5. Fees:**

* `Miner Fee`: A transaction typically includes a fee paid to miners as an incentive to include it in a block.
* `Fee Calculation`: The fee is the difference between the total input value and the total output value.



#### Fn to `Create Coinbase txn`

> **AIM:** From the selected transactions (from list of valid txns), get the coinbase txn and then return it .

<details>
<summary>Template</summary>

```python
def create_coinbase_transaction():
    coinbase_transaction = {
        # Add coinbase transaction details here
        "txid": "coinbase_txid",
        # Add other fields as needed
    }
    return coinbase_transaction
```

</details><br>

#### Fn to `Compute merkle root`

> **AIM:** Return the merkel root of the txn that is being put in the given block.

<details>
<summary>Template</summary>

```python
def compute_merkle_root(transactions):
    merkle_root = hashlib.sha256(b"".join(sorted([hashlib.sha256(json.dumps(tx).encode()).digest() for tx in transactions]))).hexdigest()
    return merkle_root
```

</details><br>

#### Fn to `Mine Block`

> **AIM:** Mines the block by finding a hash that meets the difficulty target.

<details>
<summary>Template</summary>

```python
def mine_block(transactions, coinbase_transaction):
    block_header = {
        "version": "1",
        "prev_block_hash": "previous_block_hash",
        "merkle_root": compute_merkle_root(transactions),
        "timestamp": int(time.time()),
        "nonce": 0
    }
    while True:
        block_header_hash = hashlib.sha256(json.dumps(block_header).encode()).hexdigest()
        if block_header_hash < DIFFICULTY_TARGET:
            break
        block_header["nonce"] += 1
    return block_header, coinbase_transaction
```

</details><br>


#### Fn to `Format output`

> **AIM:** This will create `output.txt` in desired format.

<details>
<summary>Template</summary>

```python
def format_output(block_header, coinbase_transaction, valid_transactions):
    with open("output.txt", "w") as file:
        file.write(json.dumps(block_header) + "\n")
        file.write(json.dumps(coinbase_transaction) + "\n")
        for transaction in valid_transactions:
            file.write(transaction["txid"] + "\n")
```

</details><br>

