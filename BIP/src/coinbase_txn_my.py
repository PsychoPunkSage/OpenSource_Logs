import binascii
import hashlib
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

def to_hash256(hex_input):
    return hashlib.sha256(hashlib.sha256(bytes.fromhex(hex_input)).digest()).digest().hex()

def to_sha256(hex_input):
    return hashlib.sha256(bytes.fromhex(hex_input)).digest().hex()

def to_reverse_bytes_string(hex_input):
    return bytes.fromhex(hex_input)[::-1].hex()

def fees(txnId):
    file_path = os.path.join('mempool', f'{txnId}.json') # file path
    if os.path.exists(file_path):
        # Read the JSON data from the file
        with open(file_path, 'r') as file:
            txn_data = json.load(file)

    amt_vin = sum([vin["prevout"]["value"] for vin in txn_data["vin"]])
    amt_vout = sum([vout["value"] for vout in txn_data["vout"]])

    return amt_vin - amt_vout

# FOR ONLY NON-SEGWIT
def txid(txn_id):
    txn_hash = ""

    file_path = os.path.join("mempool", f"{txn_id}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # Version
            txn_hash += f"{to_little_endian(data['version'], 4)}"

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

            # Locktime
            txn_hash += f"{to_little_endian(data['locktime'], 4)}"
    return txn_hash

def wtxid(txn_id):
    txn_hash = ""

    file_path = os.path.join("mempool", f"{txn_id}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # Version
            txn_hash += f"{to_little_endian(data['version'], 4)}"
            # Marker+flags (if any `vin` has empty scriptsig)
            if any(i.get("scriptsig") == "" for i in data["vin"]):
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

# txid = to_reverse_bytes_string(to_hash256(create_raw_txn_data_full("e4020c97eb2eb68055362d577e7068a128ceb887a33260062bb3ba2820b9bd30")))
# print(f"txid::> {txid}")
# filename = "e4020c97eb2eb68055362d577e7068a128ceb887a33260062bb3ba2820b9bd30"
filename = "c1b27a173feead93944952612148c8972e5837d4d564dda8b96639561402ad2e"
# filename = "0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240"
print(f"txn_hash = {wtxid(filename)}\n")
tx_id = (to_hash256(wtxid(filename)))
print(f"txid::> {to_reverse_bytes_string(tx_id)}")
'''
NON_SEGWIT
txid = to_hash256(create_raw_txn_data_full("0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240"))

896aeeb4d8af739da468ad05932455c639073fa3763d3256ff3a2c86122bda4e >> Actual txn_id
4eda2b12862c3aff56323d76a33f0739c655249305ad68a49d73afd8b4ee6a89.json >> present in valid_mempool


SEGWIT::
tx_id = (to_hash256(txid("e4020c97eb2eb68055362d577e7068a128ceb887a33260062bb3ba2820b9bd30")))

0a3fa2941f316cbf05d7a708f180a4f7cd8034f33ccfea77091252354da41e61.json >> present in valid_mempool
'''


def _is_segwit(txn_id):
    txn_data = txid(txn_id) # get raw txn_data
    # print(txn_data) # print
    # print(txn_data[8:12])
    if txn_data[8:12] == "0001":
        return True
    return False
'''
wTXID(Legacy) == TXID(Legacy) ===> reverse_bytes(SHA256(txn_data))

wTXID Commitment === HASH256(merkle root for all of the wTXIDs <witness_root_hash>  | witness_reserved_value)
        --> Must have `COINBASE_TXN` at the begining


p2pkh ::> 0a331187bb44a28b342bd2fdfd2ff58147f0e4e43444b5efd89c71f3176caea6.json :: 0a331187bb44a28b342bd2fdfd2ff58147f0e4e43444b5efd89c71f3176caea6
p2wpkh::> 0a3fa2941f316cbf05d7a708f180a4f7cd8034f33ccfea77091252354da41e61.json :: 0a3fa2941f316cbf05d7a708f180a4f7cd8034f33ccfea77091252354da41e61
'''

###########################################################################
# MERKLE ROOT
def hashIt(firstTxHash, secondTxHash):
    unhex_reverse_first = binascii.unhexlify(firstTxHash)[::-1]
    unhex_reverse_second = binascii.unhexlify(secondTxHash)[::-1]

    concat_inputs = unhex_reverse_first+unhex_reverse_second
    first_hash_inputs = hashlib.sha256(concat_inputs).digest()
    final_hash_inputs = hashlib.sha256(first_hash_inputs).digest()
    return binascii.hexlify(final_hash_inputs[::-1])
 
def merkleCalculator(hashList):
    if len(hashList) == 1:
        return hashList[0]
    newHashList = []
    for i in range(0, len(hashList)-1, 2):
        newHashList.append(hashIt(hashList[i], hashList[i+1]))
    if len(hashList) % 2 == 1: # odd, hash last item twice
        newHashList.append(hashIt(hashList[-1], hashList[-1]))
    return merkleCalculator(newHashList)
###########################################################################

def calculate_witness_commitment(txn_files):
    """
    Calculate the witness commitment of the transactions in the block.
    """
    wtxids = []
    for tx in txn_files:
        w_txid = to_hash256(wtxid(tx))
        wtxids.append(w_txid)
    wtxids.insert(0, "0000000000000000000000000000000000000000000000000000000000000000")
    witness_root = merkleCalculator(wtxids)

    # Convert the WITNESS_RESERVED_VALUE to hex string
    WITNESS_RESERVED_VALUE_HEX = '0000000000000000000000000000000000000000000000000000000000000000'

    # Convert the hexadecimal string to bytes

    # WITNESS_RESERVED_VALUE_BYTES = bytes.fromhex(WITNESS_RESERVED_VALUE_HEX)
    witness_reserved_value_hex = WITNESS_RESERVED_VALUE_HEX

    # Concatenate the witness root and the witness reserved value
    combined_data = witness_root + witness_reserved_value_hex

    # Calculate the hash (assuming hash256 is a function that hashes data with SHA-256 twice)
    witness_commitment = hashlib.sha256(hashlib.sha256(combined_data.encode()).digest()).digest()

    return witness_commitment.hex()

def _make_coinbase_raw_data_segwit(txn_files): # txn_files ::> (List) of valide .json files
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
    scriptsig_size = f"{to_compact_size(len(scriptsig)//2)}"

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

    w_commit = calculate_witness_commitment(txn_files)


    script_public_key2 = f"scriptpubkey{w_commit}"
    # SCRIPT_PUBLIC_SIZE 2 #
    print(f"SCRIPT_PUBLIC_SIZE2 (witness:: OUTPUT2) {to_compact_size(len(script_public_key2)//2)}") # 26
    raw_data += f"{to_compact_size(len(script_public_key2)//2)}"

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
    locktime: "00000000"
    raw_data += locktime

    return raw_data

def coinbase_txn_id(txn_files):
    raw_data = _make_coinbase_raw_data_segwit(txn_files)
    coinbase_hash = to_hash256(raw_data)
    # reversed_bytes = to_reverse_bytes_string(coinbase_hash)
    # txnId = to_sha256(reversed_bytes)
    return coinbase_hash