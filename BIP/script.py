import json
import os

'''

'''
def read_transactions():
    txn_ids = []
    mempool_dir = "mempool"
    try:
        for filename in os.listdir(mempool_dir):
            with open(os.path.join(mempool_dir, filename), "r") as file:
                txn_ids.append(filename[:-5])
        return txn_ids
    except Exception as e:
        print("Error:", e)
        return None

# print(read_transactions())
