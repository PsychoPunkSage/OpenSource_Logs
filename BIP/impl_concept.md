## ToDo: Simulate Minig of Blocks

>> Write a code that will process the txns, mines them, validates them and put them in a block.

> **INPUT:**
> Folders with JSON Files

> **OUTPUT:** (Block - output.txt)
> - First line: The block header.
> - Second line: The serialized coinbase transaction.
> - Following lines: The transaction IDs (txids) of the transactions mined in the block, in order. The first txid should be that of the coinbase transaction

### Things to remember:

>> **Block Mining:**

> **Step 1: Transaction Selection**

`Mempool Overview`: The mempool is a collection of all pending transactions waiting to be included in a block.<br>
`Transaction Prioritization`: Transactions may be prioritized based on factors like transaction fee, transaction size, etc.<br>
`Transaction Sorting`: Sort transactions based on priority, with higher fee transactions typically given priority.<br>
`Transaction Filtering`: Remove any transactions deemed invalid or conflicting.

> **Step 2: Transaction Validation**

`Input Validation`: Check if each transaction's inputs are valid and exist in the UTXO (Unspent Transaction Output) set.<br>
`Double Spending Check`: Ensure no inputs are spent more than once.<br>
`Script Validation`: Validate the transaction scripts, including signature verification.<br>
`Transaction Fee Check`: Verify that the transaction fee is sufficient according to current network standards.<br>
`Consensus Rules Compliance`: Ensure all transactions adhere to Bitcoin's consensus rules.<br>

> **Step 3: Block Header Construction**

`Version`: Define the block version number.<br>
`Previous Block Hash`: Include the hash of the previous block in the blockchain to maintain the chain's continuity.<br>
`Merkle Root`: Calculate the Merkle root hash of all valid transactions included in the block.<br>
`Timestamp`: Assign a timestamp to the block, indicating when the block was created.<br>
`Target Difficulty`: Determine the target difficulty for mining the block.<br>
`Nonce`: Initialize the nonce value, which miners will increment during mining to find a valid block hash.<br>

> **Step 4: Mining**

`Proof-of-Work`: Start the mining process by selecting a nonce value and combining it with the block header.<br>
`Block Hash Calculation`: Hash the block header using the SHA-256 cryptographic hash function.<br>
`Difficulty Adjustment`: Compare the resulting hash with the target difficulty. If the hash meets the difficulty criteria (has a sufficient number of leading zeros), the block is considered mined.<br>
`Nonce Incrementation`: If the hash does not meet the difficulty criteria, increment the nonce value and repeat the hashing process.<br>
`Block Submission`: Once a valid block hash is found, broadcast the block to the network for validation and inclusion in the blockchain.<br>

> Step 5: Block Validation (Performed by Network Nodes)
Consensus Verification: Network nodes verify the validity of the block, including the transactions it contains and its adherence to consensus rules.
Chain Longest-Valid Rule: Nodes accept the block only if it extends the longest valid chain and is considered valid according to network consensus.
Block Propagation: Validated blocks are propagated to other nodes in the network for further verification and propagation.