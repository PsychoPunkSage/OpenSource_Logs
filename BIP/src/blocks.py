import os
import json
import time
import hashlib
import validate_txn
import helper.converter as convert

DIFFICULTY = "0000ffff00000000000000000000000000000000000000000000000000000000"

def raw_block_data(txn_ids, nonce):
    block_header = ""

    ## Version : 4 ##
    block_header += "02000000"

    # Previous block :32 ## here = 0000000000000000000000000000000000000000000000000000000000000000
    prev_block_hash = "0000000000000000000000000000000000000000000000000000000000000000"
    block_header += f"{prev_block_hash}"

    ## Merkle root :32 ##

    ## time :4 ##
    uinx_timestamap = int(time.time())
    block_header += f"{convert.to_little_endian(uinx_timestamap, 4)}"

    ## bits :4 ##
    bits = "1f00ffff" # related to difficulty
    block_header += f"{bits}"

    ## Nonce :4 ##
    block_header += f"{convert.to_little_endian(nonce, 4)}"

    ## Transaction Count ##
    txn_count = len(txn_ids)
    block_header += f"{convert.to_little_endian(txn_count, 4)}"

    ## Transaction IDs ##
    for txn_id in txn_ids:
        block_header += f"{convert.to_compact_size(txn_id)}"

    return block_header

    ## Transactions ##
    # Coinbase transaction

    # Regular Transactions

"""
Block Hash::> - double-SHA256'ing the block header
              - the block hash is in reverse byte order when searching for a block in a block explorer.
              - block hash must get below the current target for the block to get added on to the blockchain.
"""


def mine(txn_ids):
    nonce = 0
    while raw_block_data(txn_ids, nonce) > DIFFICULTY:
        nonce += 1
    return raw_block_data(txn_ids, nonce)

































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

import json

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

