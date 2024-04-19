import os
import json
import hashlib
import scripts.p2pkh
import helper.get_txn_id as tx_id  

################
## Txn Weight ##
################
def _txn_weight(txnId):
    tx_bytes = len(tx_id.create_raw_txn_data_full(txnId))//2
    tx_weight = 4*(len(tx_id.create_raw_txn_data_min(txnId))//2) + (tx_bytes - len(tx_id.create_raw_txn_data_min(txnId))//2)
    tx_virtual_weight = tx_weight/4

    return [tx_bytes, tx_weight, tx_virtual_weight]

##########
## FEES ##
##########
def fees(txnId):
    file_path = os.path.join('mempool', f'{txnId}.json') # file path
    if os.path.exists(file_path):
        # Read the JSON data from the file
        with open(file_path, 'r') as file:
            txn_data = json.load(file)

    amt_vin = sum([vin["prevout"]["value"] for vin in txn_data["vin"]])
    amt_vout = sum([vout["value"] for vout in txn_data["vout"]])

    return amt_vin - amt_vout

##############
## Txn Data ##
##############

#################
## TxnId Check ##
#################
def _get_txn_id(txn_id):
    txn_data = tx_id.create_raw_txn_data_full(txn_id) # get raw txn_data
    if txn_data[8:12] == "0001":
        txn_data = tx_id.create_raw_txn_data_min(txn_id)
    txn_hash = hashlib.sha256(hashlib.sha256(bytes.fromhex(txn_data)).digest()).digest().hex() # 2xSHA256
    reversed_bytes = bytes.fromhex(txn_hash)[::-1].hex() # bytes reversal
    txnId = hashlib.sha256(bytes.fromhex(reversed_bytes)).digest().hex() # last sha256
    return txnId

#######################
## Segwit/Non-Segwit ##
#######################
def _is_segwit(txn_id):
    txn_data = tx_id.create_raw_txn_data_full(txn_id) # get raw txn_data
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

    if not _is_segwit(txnId):
    ## P2PKH:
        for i in txn_data["vin"]:
            if i['prevout']['scriptpubkey_type'] != "p2pkh":
                return False
            signature = i["scriptsig_asm"].split(" ")[1]
            pubkey = i["scriptsig_asm"].split(" ")[3]
            scriptpubkey_asm = i["prevout"]["scriptpubkey_asm"].split(" ")
            raw_txn_data = scripts.p2pkh.legacy_txn_data(txnId)
            return scripts.p2pkh.validate_p2pkh_txn(signature, pubkey, scriptpubkey_asm, raw_txn_data)

    # return True

print("\nOUTPUT::>\n")
# print(validate("0a3c3139b32f021a35ac9a7bef4d59d4abba9ee0160910ac94b4bcefb294f196"))
# print(validate("ff0717b6f0d2b2518cfb85eed7ccea44c3a3822e2a0ce6e753feecf68df94a7f"))
print(validate("0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240")) # - p2pkh only
# print(validate("1ccd927e58ef5395ddef40eee347ded55d2e201034bc763bfb8a263d66b99e5e"))
# print(validate("0a5d6ddc87a9246297c1038d873eec419f04301197d67b9854fa2679dbe3bd65"))

"""
>> 02000000 01 25c9f7c56ab4b9c358cb159175de542b41c7d38bf862a045fa5da51979e37ffb 01000000 1976a914286eb663201959fb12eff504329080e4c56ae28788acffffffff0254e80500000000001976a9141ef7874d338d24ecf6577e6eadeeee6cd579c67188acc8910000000000001976a9142e391b6c47778d35586b1f4154cbc6b06dc9840c88ac00000000
   02000000 01 25c9f7c56ab4b9c358cb159175de542b41c7d38bf862a045fa5da51979e37ffb 01000000 6b4830450221008f619822a97841ffd26eee942d41c1c4704022af2dd42600f006336ce686353a0220659476204210b21d605baab00bef7005ff30e878e911dc99413edb6c1e022acd012102c371793f2e19d1652408efef67704a2e9953a43a9dd54360d56fc93277a5667dffffffff0254e80500000000001976a9141ef7874d338d24ecf6577e6eadeeee6cd579c67188acc8910000000000001976a9142e391b6c47778d35586b1f4154cbc6b06dc9840c88ac0000000001000000
"""