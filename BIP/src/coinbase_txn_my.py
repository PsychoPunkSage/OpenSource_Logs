import hashlib
import json
import os
import helper.converter as convert
import helper.merkle_root as merkle
import helper.txn_info as txinfo

WTXID_COINBASE = bytes(32).hex()

def fees(txnId):
    file_path = os.path.join('mempool', f'{txnId}.json') # file path
    if os.path.exists(file_path):
        # Read the JSON data from the file
        with open(file_path, 'r') as file:
            txn_data = json.load(file)

    amt_vin = sum([vin["prevout"]["value"] for vin in txn_data["vin"]])
    amt_vout = sum([vout["value"] for vout in txn_data["vout"]])

    return amt_vin - amt_vout

def calculate_witness_commitment(txn_files):
    """
    Calculate the witness commitment of the transactions in the block.
    """
    wtxids = [WTXID_COINBASE]
    for tx in txn_files:
        w_txid = convert.to_hash256(txinfo.wtxid(tx))
        wtxids.append(w_txid)
    # wtxids.insert(0, "0000000000000000000000000000000000000000000000000000000000000000")
    witness_root = merkle.generate_merkle_root(wtxids)

    # Convert the WITNESS_RESERVED_VALUE to hex string
    WITNESS_RESERVED_VALUE_HEX = '0000000000000000000000000000000000000000000000000000000000000000'

    # WITNESS_RESERVED_VALUE_BYTES = bytes.fromhex(WITNESS_RESERVED_VALUE_HEX)
    witness_reserved_value_hex = WITNESS_RESERVED_VALUE_HEX
    # print(witness_reserved_value_hex)
    # print(witness_root)

    # Concatenate the witness root and the witness reserved value
    combined_data = witness_root + witness_reserved_value_hex

    # Calculate the hash (assuming hash256 is a function that hashes data with SHA-256 twice)
    witness_commitment = hashlib.sha256(hashlib.sha256(combined_data.encode()).digest()).digest()

    return witness_commitment.hex()

def _make_coinbase_raw_data_segwit(witness_commitment): # txn_files ::> (List) of valide .json files
    # txn_files.insert(0, "0000000000000000000000000000000000000000000000000000000000000000")
    raw_data = ""
    # reward = 0

    #  SHOULD I USE ``BLOCK_SUBSIDY``
    # for txnId in txn_files:
    #     reward += fees(txnId)

    # VERSION #
    ver = 1
    raw_data += f"{convert.to_little_endian(ver, 4)}"

    # MARKER + FLAG #
    marker = "00"
    flag = "01"
    raw_data += f"{marker}{flag}"

    ###########
    ## INPUT ##
    ###########
    # INPUT_COUNT #
    i_count = "01"
    raw_data += f"{i_count}"

    # INPUT_TX_ID #
    tx_id = "0000000000000000000000000000000000000000000000000000000000000000"
    raw_data += f"{tx_id}"

    # V_OUT #
    v_out = "ffffffff"
    raw_data += f"{v_out}"

    # SCRIPTSIZE #
    scriptsig = "03233708184d696e656420627920416e74506f6f6c373946205b8160a4256c0000946e0100" # RANDOM
    # SCRIPTSIG_SIZE #
    scriptsig_size = f"{convert.to_compact_size(len(scriptsig)//2)}"

    # SEQUENCE #
    sequence = "ffffffff"
    raw_data += f"{sequence}"

    ############
    ## OUTPUT ##
    ############
    # OUTPUT_COUNT #
    o_count = "02" # segwit
    raw_data += f"{o_count}"

    # OUTPUT_AMOUNT 1 #
    # o_amount = f"{to_little_endian(reward, 8)}"
    o_amount = f"f595814a00000000"
    raw_data += f"{o_amount}"

    script_public_key = "76a914edf10a7fac6b32e24daa5305c723f3de58db1bc888ac"

    # SCRIPT_PUBLIC_SIZE 1 #
    raw_data += f"{convert.to_compact_size(len(script_public_key)//2)}"

    # SCRIPT_PUBLIC_KEY 1 #
    raw_data += f"{script_public_key}"
    
    # OUTPUT_AMOUNT 2 #
    o_amount2 = "0000000000000000"
    raw_data += f"{o_amount2}"

    script_public_key2 = f"6a24aa21a9ed{witness_commitment}"
    # SCRIPT_PUBLIC_SIZE 2 #
    # print(f"SCRIPT_PUBLIC_SIZE2 (witness:: OUTPUT2) {convert.to_compact_size(len(script_public_key2)//2)}") # 26
    raw_data += f"{convert.to_compact_size(len(script_public_key2)//2)}"

    # SCRIPT_PUBLIC_KEY 2 #
    raw_data += f"{script_public_key2}"

    ## witness ##
    # STACK_ITEMS
    stack_items = "01"
    raw_data += f"{stack_items}"

    # SIZE
    size = "20"
    raw_data += f"{size}"

    # ITEM
    item = "0000000000000000000000000000000000000000000000000000000000000000"
    raw_data += f"{item}"

    # LOCKTIME #
    locktime = "00000000"
    raw_data += locktime

    return raw_data

def coinbase_txn_id(witness_commitment):
    raw_data = _make_coinbase_raw_data_segwit(witness_commitment)
    coinbase_hash = convert.to_hash256(raw_data)
    # reversed_bytes = convert.to_reverse_bytes_string(coinbase_hash)
    # txnId = convert.to_sha256(reversed_bytes)
    return raw_data, convert.to_reverse_bytes_string(coinbase_hash)

def serialize_coinbase_transaction(witness_commitment):
    tx_dict = {
        "version": "01000000",
        "marker": "00",
        "flag": "01",
        "inputcount": "01",
        "inputs": [
            {
                "txid": "0000000000000000000000000000000000000000000000000000000000000000",
                "vout": "ffffffff",
                "scriptsigsize": "25",
                "scriptsig": "03233708184d696e656420627920416e74506f6f6c373946205b8160a4256c0000946e0100",
                "sequence": "ffffffff",
            }
        ],
        "outputcount": "02",
        "outputs": [
            {
                "amount": "f595814a00000000",
                "scriptpubkeysize": "19",
                "scriptpubkey": "76a914edf10a7fac6b32e24daa5305c723f3de58db1bc888ac",
            },
            {
                "amount": "0000000000000000",
                "scriptpubkeysize": "26",
                "scriptpubkey": f"6a24aa21a9ed{witness_commitment}",
            },
        ],
        "witness": [
            {
                "stackitems": "01",
                "0": {
                    "size": "20",
                    "item": "0000000000000000000000000000000000000000000000000000000000000000",
                },
            }
        ],
        "locktime": "00000000",
    }
    tx_dict_modified = {
        "version": 1,
        "marker": "00",
        "flag": "01",
        "inputcount": "01",
        "vin": [
            {
                "txid": "0000000000000000000000000000000000000000000000000000000000000000",
                "vout": int("ffffffff", 16),
                "scriptsigsize": 37,
                "scriptsig": "03233708184d696e656420627920416e74506f6f6c373946205b8160a4256c0000946e0100",
                "sequence": int("ffffffff", 16),
            }
        ],
        "outputcount": "02",
        "vout": [
            {
                "value": 2753059167,
                "scriptpubkeysize": "19",
                "scriptpubkey": "76a914edf10a7fac6b32e24daa5305c723f3de58db1bc888ac",
            },
            {
                "value": 0,
                "scriptpubkeysize": "26",
                "scriptpubkey": f"6a24aa21a9ed{witness_commitment}",
            },
        ],
        "witness": [
            {
                "stackitems": "01",
                "0": {
                    "size": "20",
                    "item": "0000000000000000000000000000000000000000000000000000000000000000",
                },
            }
        ],
        "locktime": 0,
    }
    # Version
    serialized_tx = tx_dict["version"]

    # Marker and Flag
    serialized_tx += tx_dict["marker"] + tx_dict["flag"]

    # Input Count
    serialized_tx += tx_dict["inputcount"]

    # Input
    input_data = tx_dict["inputs"][0]
    serialized_tx += input_data["txid"]
    serialized_tx += input_data["vout"]
    serialized_tx += input_data["scriptsigsize"].zfill(2)
    serialized_tx += input_data["scriptsig"]
    serialized_tx += input_data["sequence"]

    # Output Count
    serialized_tx += tx_dict["outputcount"]

    # Outputs
    for output in tx_dict["outputs"]:
        serialized_tx += output["amount"].zfill(16)
        serialized_tx += output["scriptpubkeysize"].zfill(2)
        serialized_tx += output["scriptpubkey"]

    # Witness
    witness_data = tx_dict["witness"][0]
    serialized_tx += witness_data["stackitems"]
    serialized_tx += witness_data["0"]["size"].zfill(2)
    serialized_tx += witness_data["0"]["item"]

    # Locktime
    serialized_tx += tx_dict["locktime"]

    # print(serialize_txn(tx_dict_modified))
    return serialized_tx, convert.to_reverse_bytes_string(convert.to_hash256(txinfo.txid_dict(tx_dict_modified)))
