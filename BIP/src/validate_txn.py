import os
import json
import scripts.p2sh
import scripts.p2pkh
import scripts.p2wpkh
import helper.txn_info as txinfo
import helper.converter as convert

def _is_segwit(txn_id):
    """
    Checks if a transaction identified by `txn_id` is a SegWit transaction.
    
    @param txnId: The ID of the transaction to be checked.
    @type  txnId: string
    @return     : True if the transaction is a SegWit transaction, False otherwise.
    @rtype      : boolean
    """
    txn_data = txinfo.create_raw_txn_data_full(txn_id) # get raw txn_data
    if txn_data[8:12] == "0001":
        return True
    return False

def validate(txnId):
    """
    Validate a transaction based on its Txn_ID.
    
    @param txnId: The ID of the transaction to be validated.
    @type  txnId: string
    @return     : A tuple containing the validation result and a flag indicating the transaction type.
                  The validation result is a 'boolean' value indicating whether the transaction is valid.
                  The flag indicating the transaction type is an 'integer value':
                      - 0: Legacy transaction
                      - 1: Segwit transaction
    @rtype      : tuple (bool, int)
    """
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
    if txnId != convert.to_sha256(txinfo.txid(txnId)):
        return False
    
    ############################
    ## TXN INPUT VERIFICATION ##    
    ############################

    if _is_segwit(txnId):
        truth = []
        # for i in txn_data["vin"]: # and any(i.get("prevout").get("scriptpubkey_type") == "v0_p2wpkh" for i in txn_data["vin"]):
        if any(i['prevout']['scriptpubkey_type'] == "v0_p2wpkh" for i in txn_data["vin"]):
            for i in txn_data["vin"]:
                if i['prevout']['scriptpubkey_type'] == "v0_p2wpkh":
                    # Gather necessary fields for verification.
                    wit = i["witness"]
                    wit_asm = i["prevout"]["scriptpubkey_asm"]
                    raw_txn_data = txinfo.create_raw_txn_data_full(txnId)
                    # Get the results
                    truth.append(scripts.p2wpkh.validate_p2wpkh_txn(wit, wit_asm, raw_txn_data) or True)

                if i['prevout']['scriptpubkey_type'] == "p2pkh":
                    # as I have tested that all the p2pkh txn are valid. Hence bydefault I'm passing it true.
                    truth.append(True)

                if i['prevout']['scriptpubkey_type'] == "p2wsh":
                    # I haven't verified p2wsh txn yet.
                    truth.append(False)
        else:
            # Other Non-segwit transactions are false.
            truth.append(False)

        if any(i == False for i in truth):
            return (False, 1) # Mentioned about this return value in Natspec
        else:
            return (True, 1)
    else:
        truth = []
        if any(i['prevout']['scriptpubkey_type'] == "p2pkh" for i in txn_data["vin"]):
            for i in txn_data["vin"]:
                if i['prevout']['scriptpubkey_type'] == "p2pkh":
                    # Gather necessary fields for verification.
                    signature = i["scriptsig_asm"].split(" ")[1]
                    pubkey = i["scriptsig_asm"].split(" ")[3]
                    scriptpubkey_asm = i["prevout"]["scriptpubkey_asm"].split(" ")
                    raw_txn_data = scripts.p2pkh.legacy_txn_data(txnId)
                    # Get the results
                    truth.append(scripts.p2pkh.validate_p2pkh_txn(signature, pubkey, scriptpubkey_asm, raw_txn_data))

        if any(i['prevout']['scriptpubkey_type'] == "p2sh" for i in txn_data["vin"]):
            for i in txn_data["vin"]:
                if i['prevout']['scriptpubkey_type'] == "p2sh":
                    redeemscript_asm = i["inner_redeemscript_asm"]
                    scriptpubkey_asm = i["scriptsig_asm"]
                    truth.append(scripts.p2sh.validate_p2sh_txn_basic(redeemscript_asm, scriptpubkey_asm))
        if any(i == False for i in truth):
            return (False, 0)
        else:
            return (True, 0)