import os
import time
import hashlib
import validate_txn
import coinbase_txn_my as coinbase
import helper.converter as convert
import helper.merkle_root as merkle 
import helper.txn_info as txinfo

OUTPUT_FILE = "output.txt"
DIFFICULTY = "0000ffff00000000000000000000000000000000000000000000000000000000"
BLOCK_VERSION = 4
MEMPOOL_DIR = "mempool"

def mine_block(transaction_files):
    """
    Mine a block with the given transactions.
    """
    nonce = 0
    txids = [txinfo.txid(tx) for tx in transaction_files]
    # print(f"txids::> {txids}")

    # Create a coinbase transaction with no inputs and two outputs: one for the block reward and one for the witness commitment
    witness_commitment = coinbase.calculate_witness_commitment(transaction_files)
    print("witneness commitment:", witness_commitment)

    coinbase_hex, coinbase_txid = coinbase.serialize_coinbase_transaction(witness_commitment=witness_commitment)

    # Calculate the Merkle root of the transactions
    merkle_root = merkle.generate_merkle_root([coinbase_txid]+txids)

    # Construct the block header
    block_version_bytes = BLOCK_VERSION.to_bytes(4, "little")
    prev_block_hash_bytes = bytes.fromhex(
        "0000000000000000000000000000000000000000000000000000000000000000"
    )
    merkle_root_bytes = bytes.fromhex(merkle_root)
    timestamp_bytes = int(time.time()).to_bytes(4, "little")
    bits_bytes = (0x1F00FFFF).to_bytes(4, "little")
    nonce_bytes = nonce.to_bytes(4, "little")

    # Combine the header parts
    block_header = (
        block_version_bytes
        + prev_block_hash_bytes
        + merkle_root_bytes
        + timestamp_bytes
        + bits_bytes
        + nonce_bytes
    )

    # Attempt to find a nonce that results in a hash below the difficulty target
    target = int(DIFFICULTY, 16)
    print("target:", target)
    while True:
        block_hash = hashlib.sha256(hashlib.sha256(block_header).digest()).digest()
        reversed_hash = block_hash[::-1]
        if int.from_bytes(reversed_hash, "big") <= target:
            break
        nonce += 1
        nonce_bytes = nonce.to_bytes(4, "little")
        block_header = block_header[:-4] + nonce_bytes  # Update the nonce in the header
        # Validate nonce range within the mining loop
        if nonce < 0x0 or nonce > 0xFFFFFFFF:
            raise ValueError("Invalid nonce")

    block_header_hex = block_header.hex()

    return block_header_hex, txids, nonce, coinbase_hex, coinbase_txid

'''
Critical comments::>

* CONBASE
if (coinbaseTx.outs.length !== 2) {
    throw new Error(
      'Coinbase transaction must have exactly 2 outputs. One for the block reward and one for the witness commitment',
    )
  }

* MERKLE:
  let level = txids.map((txid) => Buffer.from(txid, 'hex').reverse().toString('hex')) ### IMP LINE
'''

def read_transactions():
    txn_files = []
    mempool_dir = "mempool"
    try:
        for filename in os.listdir(mempool_dir):
            with open(os.path.join(mempool_dir, filename), "r") as file:
                # locktime ka locha #
                txn_files.append(filename[:-5])
        # print(txn_files[:])
        # return txn_files[:5]
        return ["7cd041411276a4b9d0ea004e6dd149f42cb09bd02ca5dda6851b3df068749b2d", "c990d29bd10828ba40991b687362f532df79903424647dd1f9a5e2ace3edabca", "119604185a31e515e86ba0aec70559e7169600eab5adf943039b0a8b794b40df", "c3576a146165bdd8ecbfc79f18c54c8c51abd46bc0d093b01e640b6692372a93", "9fbc187e552b9e93406df86a4ebac8b67ccc0c4c321d0297edd8ffb87d4f5a45"]
    except Exception as e:
        print("Error:", e)
        return None

def main():
    # Read transaction files
    transactions = read_transactions()


    print(f"Total transactions: {len(transactions)}")

    if not any(transactions):
        raise ValueError("No valid transactions to include in the block")

    # Mine the block
    block_header, txids, nonce, coinbase_tx_hex, coinbase_txid = mine_block(transactions)
    # Corrected writing to output file
    with open(OUTPUT_FILE, "w") as file:
        file.write(f"{block_header}\n{coinbase_tx_hex}\n{coinbase_txid}\n")
        file.writelines(f"{txid}\n" for txid in txids)



if __name__ == "__main__":
    main()
