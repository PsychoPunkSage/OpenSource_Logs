import json
import os


def to_compact_size(value):
    if value < 0xfd:
        return value.to_bytes(1, byteorder='little').hex()
    elif value <= 0xffff:
        return (0xfd).to_bytes(1, byteorder='little').hex() + value.to_bytes(2, byteorder='little').hex()
    elif value <= 0xffffffff:
        return (0xfe).to_bytes(1, byteorder='little').hex() + value.to_bytes(4, byteorder='little').hex()
    else:
        return (0xff).to_bytes(1, byteorder='little').hex() + value.to_bytes(8, byteorder='little').hex()

def to_little_endian(num, size):
    return num.to_bytes(size, byteorder='little').hex()

def fees(txnId):
    file_path = os.path.join('mempool', f'{txnId}.json') # file path
    if os.path.exists(file_path):
        # Read the JSON data from the file
        with open(file_path, 'r') as file:
            txn_data = json.load(file)

    amt_vin = sum([vin["prevout"]["value"] for vin in txn_data["vin"]])
    amt_vout = sum([vout["value"] for vout in txn_data["vout"]])

    return amt_vin - amt_vout

def create_raw_txn_data_full(txn_id):
    txn_hash = ""

    file_path = os.path.join("mempool", f"{txn_id}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # Version
            txn_hash += f"{to_little_endian(data['version'], 4)}"
            # Marker+flags (if any `vin` has empty scriptsig)
            # if any(i.get("scriptsig") == "" for i in data["vin"]):
            #     txn_hash += "0001"
            if any((i.get("witness") and i["witness"] != []) for i in data["vin"]):
                txn_hash += "0001"
            # No. of inputs:
            txn_hash += f"{str(to_compact_size(len(data['vin'])))}"
            # Inputs
            for iN in data["vin"]:
                txn_hash += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
                txn_hash += f"{to_little_endian(iN['vout'], 4)}"
                txn_hash += f"{to_compact_size(len(iN['scriptsig'])//2)}"
                txn_hash += f"{iN['scriptsig']}"
                txn_hash += f"{to_little_endian(iN['sequence'], 4)}"

            # No. of outputs
            txn_hash += f"{str(to_compact_size(len(data['vout'])))}"

            # Outputs
            for out in data["vout"]:
                txn_hash += f"{to_little_endian(out['value'], 8)}"
                txn_hash += f"{to_compact_size(len(out['scriptpubkey'])//2)}"
                txn_hash += f"{out['scriptpubkey']}"

            # witness
            for i in data["vin"]:
                if "witness" in i and i["witness"]:
                    txn_hash += f"{to_compact_size(len(i['witness']))}"
                    for j in i["witness"]:
                        txn_hash += f"{to_compact_size(len(j) // 2)}"
                        txn_hash += f"{j}"

            # Locktime
            txn_hash += f"{to_little_endian(data['locktime'], 4)}"
    return txn_hash


def _is_segwit(txn_id):
    txn_data = create_raw_txn_data_full(txn_id) # get raw txn_data
    # print(txn_data) # print
    # print(txn_data[8:12])
    if txn_data[8:12] == "0001":
        return True
    return False
'''
wTXID(Legacy) == TXID(Legacy) ===> reverse_bytes(SHA256(txn_data))

wTXID Commitment === HASH256(merkle root for all of the wTXIDs <witness_root_hash>  | witness_reserved_value)
        --> Must have `COINBASE_TXN` at the begining
'''


def make_coinbase_raw_data_segwit(txn_files): # txn_files ::> (List) of valide .json files
    # txn_files.insert(0, "0000000000000000000000000000000000000000000000000000000000000000")
    raw_data = ""
    reward = 0

    #  SHOULD I USE ``BLOCK_SUBSIDY``
    for txnId in txn_files:
        reward += fees(txnId)

    # VERSION #
    ver = 4
    raw_data += f"{to_little_endian(ver, 4)}"

    # MARKER + FLAG #
    marker = "00"
    flag = "01"
    raw_data += f"{marker}{flag}"

    # INPUT_COUNT #
    i_count = "01"
    raw_data += f"{i_count}"

    # INPUT_TX_ID #
    tx_id = "0000000000000000000000000000000000000000000000000000000000000000"
    raw_data += f"{tx_id}"

    # V_OUT #
    v_out = "00000000"
    raw_data += f"{v_out}"

    # SCRIPTSIZE #
    scriptsig = "4d6164652062792050737963686f50756e6b53616765" # RANDOM
    # SCRIPTSIG_SIZE #
    scriptsig_size = f"{to_compact_size(len(scriptsig)//2)}"

    # SEQUENCE #
    sequence = "ffffffff"
    raw_data += f"{sequence}"

    # OUTPUT_COUNT #
    o_count = "02" # segwit
    raw_data += f"{o_count}"

    # OUTPUT_AMOUNT 1 #
    o_amount = f"{to_little_endian(reward, 8)}"
    raw_data += f"{o_amount}"

    script_public_key = "76a914edf10a7fac6b32e24daa5305c723f3de58db1bc888ac"

    # SCRIPT_PUBLIC_SIZE 1 #
    raw_data += f"{to_compact_size(len(script_public_key)//2)}"

    # SCRIPT_PUBLIC_KEY 1 #
    raw_data += f"{script_public_key}"
    
    # OUTPUT_AMOUNT 2 #
    o_amount2 = "0000000000000000"
    raw_data += f"{o_amount2}"

    ##### GET WTXID of all the transactions #####
    wTXIDs = []
    for i in txn_files:
        if i.is_segwit():
            pass
        else:
            pass


    script_public_key2 = ""
    # SCRIPT_PUBLIC_SIZE 2 #
    raw_data += f"{to_compact_size(len(script_public_key2)//2)}"

    # SCRIPT_PUBLIC_KEY 2 #
    raw_data += f"{script_public_key2}"

    ## witness?? ##

    # LOCKTIME #
    locktime: "00000000"
    raw_data += locktime

    return raw_data