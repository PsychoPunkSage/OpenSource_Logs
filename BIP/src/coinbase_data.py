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

    @param txn_files: A list of transaction files to include in the calculation.
    @type  txn_files: list

    @return         : The witness commitment calculated for the given transactions.
    @rtype          : str
    """
    wtxids = [WTXID_COINBASE] # must begin with wtxid of Coinbase txn

    # Calculate wtxid of list of transactions
    for tx in txn_files:
        w_txid = txinfo.wtxid(tx)
        wtxids.append(w_txid)

    # Get merkle root of wtxids
    witness_root = merkle.merkle_root_calculator(wtxids)
    print(f"witness root::> {witness_root}")

    # Append witness reserved value at the end.
    witness_reserved_value_hex = WITNESS_RESERVED_VALUE_HEX
    combined_data = witness_root + witness_reserved_value_hex

    # Calculate the hash256 to get witness commitment
    witness_commitment = convert.to_hash256(combined_data)
    return witness_commitment