import os
import json
import time
import hashlib

DIFFICULTY = "0000ffff00000000000000000000000000000000000000000000000000000000"

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

# def serialize_json_file(input_file_path):
#     with open(input_file_path, 'r') as file:
#         data = json.load(file)

#     serialized_data = json.dumps(data, indent=4)  # indent for pretty printing, optional
#     print(serialized_data)

# Example usage:
# input_file_path = 'input.json'
# output_file_path = 'output.json'
# serialize_json_file("mempool/0a3c3139b32f021a35ac9a7bef4d59d4abba9ee0160910ac94b4bcefb294f196.json")

def calculate_txn_id(transaction_data):
    serialized_data = json.dumps(transaction_data, separators=(',', ':')).encode()

    hash_bytes = hashlib.sha256(hashlib.sha256(serialized_data).digest()).digest()
    reversed_hash = hash_bytes[::-1]
    txn_id = reversed_hash.hex()
    return txn_id

def calculate_txn_id_from_file(json_file_path):
    with open(json_file_path, "r") as f:
        transaction_data = json.load(f)

    serialized_data = json.dumps(transaction_data, separators=(',', ':')).encode()
    serialized_bytes = bytes(serialized_data)
    hash_bytes = hashlib.sha256(hashlib.sha256(serialized_bytes).digest()).digest()
    reversed_hash = hash_bytes[::-1]
    txn_id = reversed_hash.hex()

    print(f"\n{bytes(serialized_data)}\n")

    return txn_id

# Example usage:
# file_location = "mempool/0a3c3139b32f021a35ac9a7bef4d59d4abba9ee0160910ac94b4bcefb294f196.json"
file_location = "mempool/fff4ebbd7325f2ec9a53347d063225266f324bb178134c5590bde23d83ba8f31.json"
txn_id = calculate_txn_id_from_file(file_location)
print("Transaction ID:", txn_id)
# coinbase txn init















"""
def create_witness_commitment(txn_ids):
"""

# witness calculation


"""
ISSUES::
- Do I need to serialize the transaction before calculating `merkle root`?

- mempool/0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240.json
=> Serialized Hash::> 020000000125c9f7c56ab4b9c358cb159175de542b41c7d38bf862a045fa5da51979e37ffb010000006b4830450221008f619822a97841ffd26eee942d41c1c4704022af2dd42600f006336ce686353a0220659476204210b21d605baab00bef7005ff30e878e911dc99413edb6c1e022acd012102c371793f2e19d1652408efef67704a2e9953a43a9dd54360d56fc93277a5667dffffffff0254e80500000000001976a9141ef7874d338d24ecf6577e6eadeeee6cd579c67188acc8910000000000001976a9142e391b6c47778d35586b1f4154cbc6b06dc9840c88ac00000000


hash256 your transaction to get the txid
and reverse it and sha256 once again to verify if it matches your file name

OBJECTIVE::
- Also, can anyone confirm if i understand the process correctly as of now:

- There are multiple JSON files. And each file is a whole transaction. I have to verify the signatures and scripts of each object in the "vin" array. If I can verify all the objects in vin array, I will say that this particular JSON file is a valid transaction. While, if even one of them cant be verified I will say its not a valid transaction, and disregard the ENTIRE file. Correct?
After that, I will have the set of verified, but unconfirmed transactions, and will proceed to mine the block. 

- you'll have to parse and serialise the entire transaction to validate the signature from vins

- assume validity of locktime based on block height. You need to validate locktime which are UNIX timestamps
"""

"""
SEGWIT Txn::



010000000001013c735f81c1a0115af2e735554fb271ace18c32a3faf443f9db40cb9a11ca63110000000000ffffffff02b113030000000000160014689a681c462536ad7d735b497511e527e9f59245cf120000000000001600148859f1e9ef3ba438e2ec317f8524ed41f8f06c6a024730440220424772d4ad659960d4f1b541fd853f7da62e8cf505c2f16585dc7c8cf643fe9a02207fbc63b9cf317fc41402b2e7f6fdc1b01f1b43c5456cf9b547fe9645a16dcb150121032533cb19cf37842556dd2168b1c7b6f3a70cff25a6ff4d4b76f2889d2c88a3f200000000


"""