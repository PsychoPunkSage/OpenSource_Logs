import os
import json
import hashlib

# Block init

#########################
# merkel root formation #
#########################
def double_sha256(data):
    return hashlib.sha256(hashlib.sha256(data).digest()).digest()

def compute_merkle_root(txn_hashes):
    # if no. transactions is odd
    if len(txn_hashes) % 2 == 1:
        txn_hashes.append(txn_hashes[-1])

    tree = [double_sha256(hash) for hash in txn_hashes]

    while len(tree) > 1:
        pairs = [(tree[i], tree[i+1]) for i in range(0, len(tree), 2)]
        tree = [double_sha256(pair[0] + pair[1]) for pair in pairs]

    return tree[0]

# def extract_txn_hashes_from_folder(folder_path):
#     txn_hashes = []

#     # Iterate over files in the folder
#     for filename in os.listdir(folder_path):
#         if filename.endswith(".json"):
#             file_path = os.path.join(folder_path, filename)
#             # Read JSON file
#             with open(file_path, 'r') as f:
#                 data = json.load(f)
#                 # Extract transaction hash from JSON data
#                 txn_hash = hashlib.sha256(json.dumps(data).encode()).digest()
#                 txn_hashes.append(txn_hash)

#     return txn_hashes

# coinbase txn init

# witness calculation