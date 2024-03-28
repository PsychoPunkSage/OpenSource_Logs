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

> **AIM:** From the **return the list of valid Transactions** .

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

