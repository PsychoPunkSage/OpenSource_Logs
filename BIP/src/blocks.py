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
























##############
# Block init #
##############
class Block:
    def __init__(self, previous_hash, transactions, merkle_root):
        self.previous_hash = previous_hash
        self.transactions = transactions
        self.merkle_root = merkle_root
        self.timestamp = time.time()
        self.nonce = 0

    def compute_hash(self):
        block_header = str(self.previous_hash) + str(self.merkle_root) + str(self.timestamp) + str(self.nonce)
        return hashlib.sha256(block_header.encode()).hexdigest()
    
    def mine_block(self):
        target = '0' * DIFFICULTY
        while self.compute_hash()[:DIFFICULTY] != target:
            self.nonce += 1
        print("Block mined:", self.compute_hash())


def get_witness_commitment():
    # Implement the logic to extract witness commitment from the coinbase transaction
    # If it exists
    pass

"""
COINBASE TXN::>

:> Coinbase:
    @> https://learnmeabitcoin.com/technical/transaction/input/#coinbase
A coinbase is a special type of input found in coinbase transactions.
The input for a coinbase transaction doesn't need to reference any previous outputs, as a coinbase transaction is simply used to collect the block reward. Therefore, the TXID is set to all zeros, the VOUT is set to the maximum value, and a miner is free to put any data they like inside the ScriptSig.
For example, this is the coinbase transaction for block
"""

##################
## Coinbase txn ##
##################

def create_coinbase_txn_data(txn_list):
    reward = 0
    for txnId in txn_list:
        reward += validate_txn.fees(txnId)
    
    version = "01000000"
    in_count = "01"
    in_txnId = "0000000000000000000000000000000000000000000000000000000000000000"
    vout = "ffffffff"
    scriptsig_size = "07"
    scriptsig = "0453ec131c0108" # RANDOM
    sequence = "ffffffff"

    out_count = "01"
    out_amt = f"{validate_txn.to_little_endian(reward, 8)}"

    return version+in_count+in_txnId+vout+scriptsig_size+scriptsig+sequence+out_count+out_amt

def get_coinbase_txn_id(txn_list):
    data = create_coinbase_txn_data(txn_list)

    bytes_data = bytes.fromhex(data)
    txn_id_bytes = hashlib.sha256(hashlib.sha256(bytes_data).digest()).digest()
    txid = txn_id_bytes.hex()

    return txid[::-1]


"""
def create_witness_commitment(txn_ids):
    # Compute the Merkle root of the transaction IDs
    merkle_root = compute_merkle_root(txn_ids)

    # Construct the witness commitment script
    witness_commitment_script = bytearray.fromhex('6a')  # OP_RETURN
    witness_commitment_script.extend(len(merkle_root).to_bytes(1, byteorder='big'))  # Push the Merkle root size
    witness_commitment_script.extend(merkle_root)  # Push the Merkle root

    return bytes(witness_commitment_script)
"""

# witness calculation


"""
ISSUES::
- Do I need to serialize the transaction before calculating `merkle root`?

- mempool/0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240.json
=> Serialized Hash::> <02000000><01><25c9f7c56ab4b9c358cb159175de542b41c7d38bf862a045fa5da51979e37ffb><01000000><6b><4830450221008f619822a97841ffd26eee942d41c1c4704022af2dd42600f006336ce686353a0220659476204210b21d605baab00bef7005ff30e878e911dc99413edb6c1e022acd012102c371793f2e19d1652408efef67704a2e9953a43a9dd54360d56fc93277a5667d><ffffffff><02><54e805>00000000001976a9141ef7874d338d24ecf6577e6eadeeee6cd579c67188acc8910000000000001976a9142e391b6c47778d35586b1f4154cbc6b06dc9840c88ac00000000


hash256 your transaction to get the txid
and reverse it and sha256 once again to verify if it matches your file name

OBJECTIVE::
- Also, can anyone confirm if i understand the process correctly as of now:

- There are multiple JSON files. And each file is a whole transaction. I have to verify the signatures and scripts of each object in the "vin" array. If I can verify all the objects in vin array, I will say that this particular JSON file is a valid transaction. While, if even one of them cant be verified I will say its not a valid transaction, and disregard the ENTIRE file. Correct?
After that, I will have the set of verified, but unconfirmed transactions, and will proceed to mine the block. 

- you'll have to parse and serialise the entire transaction to validate the signature from vins

- assume validity of locktime based on block height. You need to validate locktime which are UNIX timestamps

- Claculate weight of the transaction
"""

