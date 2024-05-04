import os
import json
import hashlib
import helper.txn_info as txinfo

##############
## Txn Data ##
##############

#################
## TxnId Check ##
#################
def _get_txn_id(txn_id):
    txn_data = txinfo.create_raw_txn_data_full(txn_id) # get raw txn_data
    if txn_data[8:12] == "0001":
        txn_data = txinfo.create_raw_txn_data_min(txn_id)
    txn_hash = hashlib.sha256(hashlib.sha256(bytes.fromhex(txn_data)).digest()).digest().hex() # 2xSHA256
    reversed_bytes = bytes.fromhex(txn_hash)[::-1].hex() # bytes reversal
    txnId = hashlib.sha256(bytes.fromhex(reversed_bytes)).digest().hex() # last sha256
    return txnId
# print(f"txinfo::> {_get_txn_id('0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240')}")
#######################
## Segwit/Non-Segwit ##
#######################
def _is_segwit(txn_id):
    txn_data = txinfo.create_raw_txn_data_full(txn_id) # get raw txn_data
    # print(txn_data) # print
    # print(txn_data[8:12])
    if txn_data[8:12] == "0001":
        return True
    return False

# print(f"_is_segwit::> {_is_segwit('0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240')}")

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
    # Amount Consistency <vin >= vout> as coinbase txn are not present in mempool
    if sum([vin["prevout"]["value"] for vin in txn_data["vin"]]) < sum([vout["value"] for vout in txn_data["vout"]]):
        print("ERROR::> value_Vin shouldn't be less than value_Vout")
        return False

    ########################
    ## TXN CONTENT CHECKS ##
    ########################
    if txnId != _get_txn_id(txnId):
        return False
    
    ############################
    ## TXN INPUT VERIFICATION ##    
    ############################