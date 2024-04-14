import os
import json


def validate_p2pkh_txn(txnId):
    file_path = os.path.join('mempool', f'{txnId}.json') # file path

    if os.path.exists(file_path):
        # Read the JSON data from the file
        with open(file_path, 'r') as file:
            txn_data = json.load(file) # JSON Object
    
    # scriptpubkey = txn_data["vin"]["prevout"]["scriptpubkey"]
    for i in txn_data["vin"]:
        scriptpubkey_asm = i["prevout"]["scriptpubkey_asm"].split(" ")
        print(scriptpubkey_asm)

validate_p2pkh_txn("0a3c3139b32f021a35ac9a7bef4d59d4abba9ee0160910ac94b4bcefb294f196")