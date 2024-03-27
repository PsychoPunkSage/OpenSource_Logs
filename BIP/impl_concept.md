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