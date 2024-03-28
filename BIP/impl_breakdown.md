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

> Valid Txn details

**1. Transaction Structure:**

* `Version`: Must be within a supported range (currently 1 or 2).
*`Locktime`: Specifies a block height or timestamp for transaction execution.
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

