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
        scriptsig = i["scriptsig"]
        scriptsig_asm = i["scriptsig_asm"]
        print("scriptpubkey_asm:  ", scriptsig_asm, "\n\nscriptsig:  ", scriptsig, "\n\nscriptpubkey_asm:  ", scriptpubkey_asm)
        # print(scriptpubkey_asm)

validate_p2pkh_txn("1ccd927e58ef5395ddef40eee347ded55d2e201034bc763bfb8a263d66b99e5e")