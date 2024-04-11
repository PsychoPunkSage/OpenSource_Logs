import os
import json
import time
import hashlib

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


#########################
# merkel root formation #
#########################
def compute_merkle_root(txn_hashes, include_witness_commit=False):
    # if no. transactions is odd
    if len(txn_hashes) % 2 == 1:
        txn_hashes.append(txn_hashes[-1])

    tree = [_double_sha256(hash) for hash in txn_hashes]

    while len(tree) > 1:
        pairs = [(tree[i], tree[i+1]) for i in range(0, len(tree), 2)]
        tree = [_double_sha256(pair[0] + pair[1]) for pair in pairs]

    merkle_root = tree[0]

    if include_witness_commit:
        # Assuming the witness commitment is concatenated with the Merkle root
        # Here, you would include the witness commitment of the coinbase transaction
        # If the transactions are SegWit
        witness_commitment = get_witness_commitment()
        if witness_commitment:
            merkle_root += witness_commitment

    return merkle_root

def get_witness_commitment():
    # Implement the logic to extract witness commitment from the coinbase transaction
    # If it exists
    pass

def _double_sha256(data):
    return hashlib.sha256(hashlib.sha256(data).digest()).digest()

def _extract_txn_hashes_from_folder(folder_path, txn_list):
    txn_hashes = []
    # Iterate over JSON txns in the folder
    for filename in txn_list:
        file_path = os.path.join(folder_path, filename)
        if os.path.exists(file_path) and filename.endswith(".json"):
            with open(file_path, 'r') as f:
                data = json.load(f)
                txn_hash = hashlib.sha256(json.dumps(data).encode()).digest()
                txn_hashes.append(txn_hash)
    return txn_hashes

# coinbase txn init
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
"""