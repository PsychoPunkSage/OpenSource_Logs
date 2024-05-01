import os
import json
import hashlib
import coincurve
from Crypto.Hash import RIPEMD160

def validate_signature(signature, message, publicKey):
    """
    Validate a signature against a message using a public key.

    @param signature : The signature to be validated.
    @type  signature : str
    @param message   : The message that was signed.
    @type  message   : str
    @param publicKey : The public key corresponding to the private key used for signing.
    @type  publicKey : str

    @return          : True if the signature is valid, False otherwise.
    @rtype           : bool
    """

    b_sig = bytes.fromhex(signature)
    b_msg = bytes.fromhex(message)
    b_pub = bytes.fromhex(publicKey)
    return coincurve.verify_signature(b_sig, b_msg, b_pub)

def _to_compact_size(value):
    if value < 0xfd:
        return value.to_bytes(1, byteorder='little').hex()
    elif value <= 0xffff:
        return (0xfd).to_bytes(1, byteorder='little').hex() + value.to_bytes(2, byteorder='little').hex()
    elif value <= 0xffffffff:
        return (0xfe).to_bytes(1, byteorder='little').hex() + value.to_bytes(4, byteorder='little').hex()
    else:
        return (0xff).to_bytes(1, byteorder='little').hex() + value.to_bytes(8, byteorder='little').hex()

def _little_endian(num, size):
    return num.to_bytes(size, byteorder='little').hex()

def segwit_txn_data(txn_id):
    file_path = os.path.join("mempool", f"{txn_id}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            ## Version
            ver = f"{_little_endian(data['version'], 4)}"

            ## (txid + vout)
            serialized_txid_vout = ""
            for iN in data["vin"]:
                serialized_txid_vout += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
                serialized_txid_vout += f"{_little_endian(iN['vout'], 4)}"
            # HASH256 (txid + vout)
            hash256_in = hashlib.sha256(hashlib.sha256(bytes.fromhex(serialized_txid_vout)).digest()).digest().hex()
            
            ## (sequense)
            serialized_sequense= ""
            for iN in data["vin"]:
                serialized_sequense += f"{_little_endian(iN['sequence'], 4)}"
            ## HASH256 (sequense)
            hash256_seq = hashlib.sha256(hashlib.sha256(bytes.fromhex(serialized_sequense)).digest()).digest().hex()
            
            ###############################################################################
            # TXN Specific #
            ## TXID and VOUT for the REQUIRED_input
            ser_tx_vout_sp = f"{bytes.fromhex(data['vin'][0]['txid'])[::-1].hex()}{_little_endian(data['vin'][0]['vout'], 4)}"
            print(ser_tx_vout_sp)
            ## Scriptcode
            pkh = f"{data['vin'][0]['prevout']['scriptpubkey'][4:]}" 
            scriptcode = f"1976a914{pkh}88ac"
            ## Input amount
            in_amt = f"{_little_endian(data['vin'][0]['prevout']['value'], 8)}"
            ## SEQUENCE for the REQUIRED_input
            sequence_txn = f"{_little_endian(data['vin'][0]['sequence'], 4)}"
            ###############################################################################

            # Outputs
            serialized_output= ""
            for out in data["vout"]:
                serialized_output += f"{_little_endian(out['value'], 8)}"
                serialized_output += f"{_to_compact_size(len(out['scriptpubkey'])//2)}"
                serialized_output += f"{out['scriptpubkey']}"
            ## HASH256 (output)
            hash256_out = hashlib.sha256(hashlib.sha256(bytes.fromhex(serialized_output)).digest()).digest().hex()

            ## locktime
            locktime = f"{_little_endian(data['locktime'], 4)}"

            # preimage = version + hash256(inputs) + hash256(sequences) + input + scriptcode + amount + sequence + hash256(outputs) + locktime
            preimage = ver + hash256_in + hash256_seq + ser_tx_vout_sp + scriptcode + in_amt + sequence_txn + hash256_out + locktime
    return preimage

def to_hash160(hex_input):
    sha = hashlib.sha256(bytes.fromhex(hex_input)).hexdigest()
    hash_160 = RIPEMD160.new()
    hash_160.update(bytes.fromhex(sha))
    return hash_160.hexdigest()

def _validate_p2wpkh_txn(signature, pubkey, scriptpubkey_asm, txn_data):
    """
    Validate a Pay-to-Witness-Public-Key-Hash (P2WPKH) transaction.

    @param signature        : The signature of the transaction.
    @type  signature        : str
    @param pubkey           : The public key used for the signature.
    @type  pubkey           : str
    @param scriptpubkey_asm : The assembly script of the script pubkey.
    @type  scriptpubkey_asm : list of str
    @param txn_data         : The transaction data.
    @type  txn_data         : str

    @return                 : A boolean indicating whether the transaction is valid.
    @rtype                  : bool
    """
    stack = []

    stack.append(signature)
    stack.append(pubkey)

    for i in scriptpubkey_asm:
        if i == "OP_DUP":
            stack.append(stack[-1])
            # print("===========")
            # print("OP_DUP")
            # print(stack)

        if i == "OP_HASH160":
            # print("===========")
            # print("OP_HASH160")
            hash_160 = to_hash160(stack[-1])

            stack.pop(-1)
            # print(stack)
            stack.append(hash_160)
            # print(stack)

        if i == "OP_EQUALVERIFY":
            # print("===========")
            # print("OP_EQUALVERIFY")
            if stack[-1] != stack[-2]:
                return False
            else:
                stack.pop(-1)
                # print(stack)
                stack.pop(-1)
                # print(stack)

        if i == "OP_CHECKSIG":
            # print("===========")
            # print("OP_CHECKSIG")
            if signature[-2:] == "01": # SIGHASH_ALL ONLY
                der_sig = signature[:-2]
                msg = txn_data + "01000000"
                msg_hash = hashlib.sha256(bytes.fromhex(msg)).digest().hex()
                return validate_signature(der_sig, msg_hash, pubkey)
                # return True

        if i == "OP_PUSHBYTES_20":
            # print("===========")
            # print("OP_PUSHBYTES_20")
            stack.append(scriptpubkey_asm[scriptpubkey_asm.index("OP_PUSHBYTES_20") + 1])
            # print(stack)

def validate_p2wpkh_txn(witness, wit_scriptpubkey_asm, txn_data):
    wit_sig, wit_pubkey = witness[0], witness[1]

    pkh = wit_scriptpubkey_asm.split(" ")[-1]
    scriptpubkey_asm = ["OP_DUP", "OP_HASH160", "OP_PUSHBYTES_20", pkh, "OP_EQUALVERIFY", "OP_CHECKSIG"]

    return _validate_p2wpkh_txn(wit_sig, wit_pubkey,scriptpubkey_asm, txn_data)


### TEST SCRIPT ###

# filename = "0a3fd98f8b3d89d2080489d75029ebaed0c8c631d061c2e9e90957a40e99eb4c"
# filename = "dcd45100f59948d0ba3031a55be2c131db24ab92daccb7a58696f3abccdcacca"
# file_path = os.path.join('mempool', f"{filename}.json") # file path
# if os.path.exists(file_path):
#     with open(file_path, 'r') as file: 
#         txn_data = json.load(file)
# else:
#     print(f"{filename}.json DOES NOT exist")

# wit = txn_data["vin"][0]["witness"]
# wit_asm = txn_data["vin"][0]["prevout"]["scriptpubkey_asm"]
# txn_data = segwit_txn_data(filename)

# print(f"p2wpkh::> {validate_p2wpkh_txn(wit, wit_asm, txn_data)}")