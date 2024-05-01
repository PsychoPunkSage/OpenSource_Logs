import os
import json
import helper.converter as convert

def create_raw_txn_data_min(txn_file):
    """
    Create raw transaction data from a transaction ID.
    
    @param txn_file: The ID of the transaction for which raw data is to be created.
    @type  txn_file: str

    @return        : The raw transaction data as a hexadecimal string.
    @rtype         : str
    """

    txn_data = ""

    file_path = os.path.join("mempool", f"{txn_file}.json")
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
                txn_data += f"{convert.to_compact_size(len(iN['scriptsig'])//2)}"
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
    else:
        ValueError(f"ERROR (txn_info.py)::> File {txn_file}.json Does not exist in MEMPOOL")
    return txn_data

def create_raw_txn_data_full(txn_file):
    """
    Create raw transaction data from a transaction ID.
    
    @param txn_file: The ID of the transaction for which raw data is to be created.
    @type  txn_file: str

    @return        : The raw transaction data in hexadecimal format. Includes MARKER + FLAGS + WITNESS_DATA
    @rtype         : str
    """

    txn_hash = ""

    file_path = os.path.join("mempool", f"{txn_file}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # Version
            txn_hash += f"{convert.to_little_endian(data['version'], 4)}"
            # Marker+flags (if any `vin` has empty scriptsig)
            if any(i.get("scriptsig") == "" for i in data["vin"]):
                txn_hash += "0001"
            # No. of inputs:
            txn_hash += f"{str(convert.to_compact_size(len(data['vin'])))}"
            # Inputs
            for iN in data["vin"]:
                txn_hash += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
                txn_hash += f"{convert.to_little_endian(iN['vout'], 4)}"
                txn_hash += f"{convert.to_compact_size(len(iN['scriptsig'])//2)}"
                txn_hash += f"{iN['scriptsig']}"
                txn_hash += f"{convert.to_little_endian(iN['sequence'], 4)}"

            # No. of outputs
            txn_hash += f"{str(convert.to_compact_size(len(data['vout'])))}"

            # Outputs
            for out in data["vout"]:
                txn_hash += f"{convert.to_little_endian(out['value'], 8)}"
                txn_hash += f"{convert.to_compact_size(len(out['scriptpubkey'])//2)}"
                txn_hash += f"{out['scriptpubkey']}"

            # witness
            for i in data["vin"]:
                if "witness" in i and i["witness"]:
                    txn_hash += f"{convert.to_compact_size(len(i['witness']))}"
                    for j in i["witness"]:
                        txn_hash += f"{convert.to_compact_size(len(j) // 2)}"
                        txn_hash += f"{j}"

            # Locktime
            txn_hash += f"{convert.to_little_endian(data['locktime'], 4)}"
    else:
        ValueError(f"ERROR (txn_info.py)::> File {txn_file}.json Does not exist in MEMPOOL")
    return txn_hash

def txid(txn_file):
    """
    Calculate the transaction ID (Txn_ID) based on the contents of a transaction file.
    
    @param txn_file: The name of the transaction file (without extension) located in the 'mempool' directory.
    @type  txn_file: str

    @return        : The transaction ID (Txn_ID) of the transaction.
    @rtype         : str
    """
    txn_hash = ""

    file_path = os.path.join("mempool", f"{txn_file}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # Version
            txn_hash += f"{convert.to_little_endian(data['version'], 4)}"

            # No. of inputs:
            txn_hash += f"{str(convert.to_compact_size(len(data['vin'])))}"
            # Inputs
            for iN in data["vin"]:
                txn_hash += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
                txn_hash += f"{convert.to_little_endian(iN['vout'], 4)}"
                txn_hash += f"{convert.to_compact_size(len(iN['scriptsig'])//2)}"
                txn_hash += f"{iN['scriptsig']}"
                txn_hash += f"{convert.to_little_endian(iN['sequence'], 4)}"

            # No. of outputs
            txn_hash += f"{str(convert.to_compact_size(len(data['vout'])))}"

            # Outputs
            for out in data["vout"]:
                txn_hash += f"{convert.to_little_endian(out['value'], 8)}"
                txn_hash += f"{convert.to_compact_size(len(out['scriptpubkey'])//2)}"
                txn_hash += f"{out['scriptpubkey']}"

            # Locktime
            txn_hash += f"{convert.to_little_endian(data['locktime'], 4)}"
    return convert.to_reverse_bytes_string(convert.to_hash256(txn_hash))

def wtxid(txn_file):
    """
    Calculate the WTXID (Witness Transaction ID) for a given transaction file.
    
    @param txn_file: The filename of the transaction file (without extension).
    @type  txn_file: str

    @return        : The WTXID (Witness Transaction ID) of the transaction.
    @rtype         : str
    """

    txn_hash = ""

    file_path = os.path.join("mempool", f"{txn_file}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # Version
            txn_hash += f"{convert.to_little_endian(data['version'], 4)}"
            # Marker+flags (if any `vin` has empty scriptsig)
            if any((i.get("witness") and i.get("witness") != "") for i in data["vin"]):
                txn_hash += "0001"
            # No. of inputs:
            txn_hash += f"{str(convert.to_compact_size(len(data['vin'])))}"
            # Inputs
            for iN in data["vin"]:
                txn_hash += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
                txn_hash += f"{convert.to_little_endian(iN['vout'], 4)}"
                txn_hash += f"{convert.to_compact_size(len(iN['scriptsig'])//2)}"
                txn_hash += f"{iN['scriptsig']}"
                txn_hash += f"{convert.to_little_endian(iN['sequence'], 4)}"

            # No. of outputs
            txn_hash += f"{str(convert.to_compact_size(len(data['vout'])))}"

            # Outputs
            for out in data["vout"]:
                txn_hash += f"{convert.to_little_endian(out['value'], 8)}"
                txn_hash += f"{convert.to_compact_size(len(out['scriptpubkey'])//2)}"
                txn_hash += f"{out['scriptpubkey']}"

            # witness
            for i in data["vin"]:
                if "witness" in i and i["witness"]:
                    txn_hash += f"{convert.to_compact_size(len(i['witness']))}"
                    for j in i["witness"]:
                        txn_hash += f"{convert.to_compact_size(len(j) // 2)}"
                        txn_hash += f"{j}"

            # Locktime
            txn_hash += f"{convert.to_little_endian(data['locktime'], 4)}"
    return convert.to_reverse_bytes_string(convert.to_hash256(txn_hash))

def coinbase_txn_id(txn_dict):
    """
    Calculate the transaction ID (Txn_ID) of a given transaction dictionary - especially made for coinbase transaction.

    @param txn_dict: The transaction dictionary containing transaction details.
    @type  txn_dict: dict

    @return        : The transaction ID as a hexadecimal string.
    @rtype         : str
    """
    txn_hash = ""
    data = txn_dict
    # Version
    txn_hash += f"{convert.to_little_endian(data['version'], 4)}"

    # No. of inputs:
    txn_hash += f"{str(convert.to_compact_size(len(data['vin'])))}"
    # Inputs
    for iN in data["vin"]:
        txn_hash += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
        txn_hash += f"{convert.to_little_endian(iN['vout'], 4)}"
        txn_hash += f"{convert.to_compact_size(len(iN['scriptsig'])//2)}"
        txn_hash += f"{iN['scriptsig']}"
        txn_hash += f"{convert.to_little_endian(iN['sequence'], 4)}"

    # No. of outputs
    txn_hash += f"{str(convert.to_compact_size(len(data['vout'])))}"

    # Outputs
    for out in data["vout"]:
        txn_hash += f"{convert.to_little_endian(out['value'], 8)}"
        txn_hash += f"{convert.to_compact_size(len(out['scriptpubkey'])//2)}"
        txn_hash += f"{out['scriptpubkey']}"

    # Locktime
    txn_hash += f"{convert.to_little_endian(data['locktime'], 4)}"
    # print(f"coinbase_txn_hash(dict)  ::> {txn_hash}")
    return convert.to_reverse_bytes_string(convert.to_hash256(txn_hash))