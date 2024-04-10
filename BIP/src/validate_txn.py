import os
import json


def validate(txnId) -> bool:
    file_path = os.path.join('mempool', f'{txnId}.json')

    if os.path.exists(file_path):
        # Read the JSON data from the file
        with open(file_path, 'r') as file:
            txn_data = json.load(file)
    else:
        print(f"Transaction with ID {txnId} not found.")
    
    return True

# print(validate("0a3c3139b32f021a35ac9a7bef4d59d4abba9ee0160910ac94b4bcefb294f196"))
# print(validate("0a3fd98f8b3d89d2080489d75029ebaed0c8c631d061c2e9e90957a40e99eb4c"))