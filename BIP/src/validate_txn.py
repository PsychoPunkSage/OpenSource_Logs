import os
import json


def validate(txnId):

    ###############
    ## READ TXNS ##
    ###############
    file_path = os.path.join('mempool', f'{txnId}.json') # file path

    if os.path.exists(file_path):
        # Read the JSON data from the file
        with open(file_path, 'r') as file:
            txn_data = json.load(file) # JSON Object
    else:
        print(f"ERROR::> Transaction with ID {txnId} not found.")
        return None
    
    ##################
    ## BASIC CHECKS ##
    ##################
    required_fields = ['version', 'locktime', 'vin', 'vout']
    for field in required_fields:
        if field not in txn_data:
            print(f"ERROR::> Transaction is missing the required field: {field}")
            return False
    # version check
    if txn_data["version"] > 2 or txn_data["version"] < 0:
        print(f"ERROR::> Possible Transaction versions :: 1 and 2")
        return False
    # vin and vout check - Empty or not
    if len(txn_data["vin"]) < 1 or len(txn_data["vout"]) < 1:
        print(f"ERROR::> Vin or Vout fields can't be empty")
        return False
    return True

print(validate("0a3c3139b32f021a35ac9a7bef4d59d4abba9ee0160910ac94b4bcefb294f196"))
# print(validate("0a3fd98f8b3d89d2080489d75029ebaed0c8c631d061c2e9e90957a40e99eb4c"))