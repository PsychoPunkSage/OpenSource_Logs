import os
import json
import helper.converter as convert

def _create_raw_txn_data(txn_id):
    txn_data = ""

    file_path = os.path.join("mempool", f"{txn_id}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # Version
            txn_data += f"{convert.to_little_endian(data['version'], 4)}"
            # No. of inputs:
            txn_data += f"{str(convert.to_compact_size(len(data['vin'])))}"
            # Inputs
            for iN in data["vin"]:
                txn_data += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
                txn_data += f"{convert.to_little_endian(iN['vout'], 4)}"
                txn_data += f"{convert.to_compact_size(len(iN['scriptsig'])//2)}" # FLAG@> maybe not divided by 2
                txn_data += f"{iN['scriptsig']}"
                txn_data += f"{convert.to_little_endian(iN['sequence'], 4)}"

            # No. of outputs
            txn_data += f"{str(convert.to_compact_size(len(data['vout'])))}"

            # Outputs
            for out in data["vout"]:
                txn_data += f"{convert.to_little_endian(out['value'], 8)}"
                txn_data += f"{convert.to_compact_size(len(out['scriptpubkey'])//2)}"
                txn_data += f"{out['scriptpubkey']}"

            # Locktime
            txn_data += f"{convert.to_little_endian(data['locktime'], 4)}"
    return txn_data

def get_txn_id(txn_id):
    txn_data = _create_raw_txn_data(txn_id)
    return convert.to_hash256(txn_data)