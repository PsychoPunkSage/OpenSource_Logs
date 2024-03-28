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

#### Fn to `Validate txn` from mempool

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

#### Fn to `Create Coinbase txn` from mempool

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

#### Fn to `Compute merkle root` from mempool

> **AIM:** Return the merkel root of the txn that is being put in the given block.

<details>
<summary>Template</summary>

```python
def compute_merkle_root(transactions):
    merkle_root = hashlib.sha256(b"".join(sorted([hashlib.sha256(json.dumps(tx).encode()).digest() for tx in transactions]))).hexdigest()
    return merkle_root
```

</details><br>

#### Fn to `Mine Block` from mempool

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


#### Fn to `Format output` from mempool

> **AIM:** Mines the block by finding a hash that meets the difficulty target.

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

