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

# def calculate_txn_id(transaction_data):
#     serialized_data = json.dumps(transaction_data, separators=(',', ':')).encode()

#     hash_bytes = hashlib.sha256(hashlib.sha256(serialized_data).digest()).digest()
#     reversed_hash = hash_bytes[::-1]
#     txn_id = reversed_hash.hex()
#     return txn_id

def create_raw_txn_hash(txn_id):
    txn_hash = ""

    file_path = os.path.join("mempool", f"{txn_id}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # print(f"data: {data}")
            # Version
            txn_hash += f"{_little_endian(data['version'], 4)}"
            # Marker+flags (if any `vin` has empty scriptsig)
            if any(i.get("scriptsig") == "" for i in data["vin"]):
                txn_hash += "0001"
            # No. of inputs:
            txn_hash += f"{str(_to_compact_size(len(data['vin'])))}"
            # Inputs
            for iN in data["vin"]:
                txn_hash += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
                txn_hash += f"{_little_endian(iN['vout'], 4)}"
                txn_hash += f"{_to_compact_size(len(iN['scriptsig'])//2)}" # FLAG@> maybe not divided by 2
                txn_hash += f"{iN['scriptsig']}"
                txn_hash += f"{_little_endian(iN['sequence'], 4)}"

            # No. of outputs
            txn_hash += f"{str(_to_compact_size(len(data['vout'])))}"

            # Outputs
            for out in data["vout"]:
                txn_hash += f"{_little_endian(out['value'], 8)}"
                txn_hash += f"{_to_compact_size(len(out['scriptpubkey'])//2)}"
                txn_hash += f"{out['scriptpubkey']}"

            # witness
            for i in data["vin"]:
                if "witness" in i and i["witness"]:
                    txn_hash += f"{_to_compact_size(len(i['witness']))}"
                    for j in i["witness"]:
                        txn_hash += f"{_to_compact_size(len(j) // 2)}"
                        txn_hash += f"{j}"

            # Locktime
            txn_hash += f"{_little_endian(data['locktime'], 4)}"
            # print(f"txn_hash: {txn_hash}")
    return txn_hash

def create_raw_txn_hash_wo(txn_id):
    txn_hash = ""

    file_path = os.path.join("mempool", f"{txn_id}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # print(f"data: {data}")
            # Version
            txn_hash += f"{_little_endian(data['version'], 4)}"
            # No. of inputs:
            txn_hash += f"{str(_to_compact_size(len(data['vin'])))}"
            # Inputs
            for iN in data["vin"]:
                txn_hash += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
                txn_hash += f"{_little_endian(iN['vout'], 4)}"
                txn_hash += f"{_to_compact_size(len(iN['scriptsig'])//2)}" # FLAG@> maybe not divided by 2
                txn_hash += f"{iN['scriptsig']}"
                txn_hash += f"{_little_endian(iN['sequence'], 4)}"

            # No. of outputs
            txn_hash += f"{str(_to_compact_size(len(data['vout'])))}"

            # Outputs
            for out in data["vout"]:
                txn_hash += f"{_little_endian(out['value'], 8)}"
                txn_hash += f"{_to_compact_size(len(out['scriptpubkey'])//2)}"
                txn_hash += f"{out['scriptpubkey']}"

            # Locktime
            txn_hash += f"{_little_endian(data['locktime'], 4)}"
    return txn_hash

def _to_compact_size(value):
    if value < 0xfd:
        return value.to_bytes(1, byteorder='little').hex()
    elif value <= 0xffff:
        return (0xfd).to_bytes(1, byteorder='little').hex() + value.to_bytes(2, byteorder='little').hex()
    elif value <= 0xffffffff:
        return (0xfe).to_bytes(1, byteorder='little').hex() + value.to_bytes(4, byteorder='little').hex()
    else:
        return (0xff).to_bytes(1, byteorder='little').hex() + value.to_bytes(8, byteorder='little').hex()

def _little_endian(num, size):
    return num.to_bytes(size, byteorder='little').hex()



# 0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240
# 896aeeb4d8af739da468ad05932455c639073fa3763d3256ff3a2c86122bda4e - HASH256 (2xSHA256)
h = create_raw_txn_hash("0a3c3139b32f021a35ac9a7bef4d59d4abba9ee0160910ac94b4bcefb294f196")
print(h + "\n")        
h = create_raw_txn_hash_wo("0a3c3139b32f021a35ac9a7bef4d59d4abba9ee0160910ac94b4bcefb294f196")
print(h + "\n")        

h = create_raw_txn_hash("0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240")
print(h + "\n")        
h = create_raw_txn_hash_wo("0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240")
print(h + "\n")        

h = create_raw_txn_hash("ff0717b6f0d2b2518cfb85eed7ccea44c3a3822e2a0ce6e753feecf68df94a7f")
print(h + "\n")        
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

"""
SEGWIT Txn::

"""