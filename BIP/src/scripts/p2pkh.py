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

def to_hash160(hex_input):
    sha = hashlib.sha256(bytes.fromhex(hex_input)).hexdigest()
    hash_160 = RIPEMD160.new()
    hash_160.update(bytes.fromhex(sha))
    return hash_160.hexdigest()

def segwit_txn_data(txn_file):
    """
    Generate SegWit transaction data based on the provided transaction file.

    @param txn_file: The filename of the transaction data file (without extension).
    @type  txn_file: str
    @return        : The preimage required for signing the SegWit transaction.
    @rtype         : str
    """
    file_path = os.path.join("mempool", f"{txn_file}.json")
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
            # hash256_in = convert.to_hash256(serialized_txid_vout)
            hash256_in = hashlib.sha256(hashlib.sha256(bytes.fromhex(serialized_txid_vout)).digest()).digest().hex()
            
            ## (sequense)
            serialized_sequense= ""
            for iN in data["vin"]:
                serialized_sequense += f"{_little_endian(iN['sequence'], 4)}"
            ## HASH256 (sequense)
            # hash256_seq = convert.to_hash256(serialized_sequense)
            hash256_seq = hashlib.sha256(hashlib.sha256(bytes.fromhex(serialized_sequense)).digest()).digest().hex()
            
            ###############################################################################
            # TXN Specific #
            ## TXID and VOUT for the REQUIRED_input
            ser_tx_vout_sp = f"{bytes.fromhex(data['vin'][0]['txid'])[::-1].hex()}{_little_endian(data['vin'][0]['vout'], 4)}"
            print(ser_tx_vout_sp)
            ## Scriptcode
            pkh = f"{data['vin'][0]['prevout']['scriptpubkey'][6:-4]}" 
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
            # hash256_out = convert.to_hash256(serialized_output)
            hash256_out = hashlib.sha256(hashlib.sha256(bytes.fromhex(serialized_output)).digest()).digest().hex()

            ## locktime
            locktime = f"{_little_endian(data['locktime'], 4)}"

            # preimage = version + hash256(inputs) + hash256(sequences) + input + scriptcode + amount + sequence + hash256(outputs) + locktime
            preimage = ver + hash256_in + hash256_seq + ser_tx_vout_sp + scriptcode + in_amt + sequence_txn + hash256_out + locktime
    return preimage

def legacy_txn_data(txn_file):
    """
    Generate legacy transaction data from a transaction file.

    @param txn_file: The name of the transaction file.
    @type  txn_file: str
    @return        : The legacy transaction data as a hexadecimal string.
    @rtype         : str
    """
    txn_hash = ""

    file_path = os.path.join("mempool", f"{txn_file}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # Version
            txn_hash += f"{_little_endian(data['version'], 4)}"
            # No. of inputs:
            txn_hash += f"{str(_to_compact_size(len(data['vin'])))}"
            # Inputs
            for iN in data["vin"]:
                txn_hash += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
                txn_hash += f"{_little_endian(iN['vout'], 4)}"
                txn_hash += f"{_to_compact_size(len(iN['prevout']['scriptpubkey'])//2)}" # FLAG@> maybe not divided by 2
                txn_hash += f"{iN['prevout']['scriptpubkey']}"
                txn_hash += f"{_little_endian(iN['sequence'], 4)}"

            # No. of outputs
            txn_hash += f"{str(_to_compact_size(len(data['vout'])))}"

            # Outputs
            for out in data["vout"]:
                txn_hash += f"{_little_endian(out['value'], 8)}"
                txn_hash += f"{_to_compact_size(len(out['scriptpubkey'])//2)}"
                txn_hash += f"{out['scriptpubkey']}"

            # Locktime
            txn_hash += f"{_little_endian(data['locktime'], 4)}"
    return txn_hash

##########
## MAIN ##
##########
def validate_p2pkh_txn(signature, pubkey, scriptpubkey_asm, txn_data):
    """
    Validate a pay-to-public-key-hash (P2PKH) transaction.

    @param signature        : The signature of the transaction.
    @type  signature        : str
    @param pubkey           : The public key associated with the signature.
    @type  pubkey           : str
    @param scriptpubkey_asm : The assembly representation of the script pubkey.
    @type  scriptpubkey_asm : list of str
    @param txn_data         : The transaction data.
    @type  txn_data         : str
 
    @return                 : A boolean value indicating whether the transaction is valid.
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



### TEST SCRIPT ###

# filename = "0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240"
# file_path = os.path.join('mempool', f"{filename}.json") # file path
# if os.path.exists(file_path):
#     with open(file_path, 'r') as file: 
#         txn_data = json.load(file)
#         # print(f"txn_data: {txn_data}")
# else:
#     print(f"file not found: {file_path}")
# signature = txn_data['vin'][0]["scriptsig_asm"].split(" ")[1]
# pubkey = txn_data['vin'][0]["scriptsig_asm"].split(" ")[3]
# scriptpubkey_asm = txn_data['vin'][0]["prevout"]["scriptpubkey_asm"].split(" ")
# raw_txn_data = legacy_txn_data(filename)
# print(raw_txn_data)

# print(f"p2pkh::> {validate_p2pkh_txn(signature, pubkey, scriptpubkey_asm, raw_txn_data)}")






















# def rs(signature):
#     r, s = sigdecode_der(bytes.fromhex(signature), secp256k1_generator.order)
#     print(f"r: {r}, s: {s}")
#     return (r, s)




# def legacy_p2pkh_txn_validation(signature, pubkey, scriptpubkey_asm, )
###<INJECTION>###
# filename = "1ccd927e58ef5395ddef40eee347ded55d2e201034bc763bfb8a263d66b99e5e"
# filename = "0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240"
# file_path = os.path.join('mempool', f"{filename}.json") # file path
# if os.path.exists(file_path):
#     with open(file_path, 'r') as file: 
#         txn_data = json.load(file)
# scriptsig_asm = txn_data["vin"][0]["scriptsig_asm"].split(" ")
# scriptpubkey_asm = txn_data["vin"][0]["prevout"]["scriptpubkey_asm"].split(" ")
# print(legacy_txn_data(filename))
# print(f"p2pkh::> {validate_p2pkh_txn(scriptsig_asm[1], scriptsig_asm[3], scriptpubkey_asm, legacy_txn_data(filename))}")

"""
STEPS::>
* serialize the TXID+VOUT for the specific input we want to create a signature for.
* sequence field for the input we're creating the signature for.

02000000 f81369411d3fba4eb8575cc858ead8a859ef74b94e160a036b8c1c5b023a6fae 957879fdce4d8ab885e32ff307d54e75884da52522cc53d3c4fdb60edb69a098 659a6eaf8d943ad2ff01ec8c79aaa7cb4f57002d49d9b8cf3c9a7974c5bd3608:06000000 1976a9147db10cfe69dae5e67b85d7b59616056e68b3512288ac f1a2010000000000 fdffffff 0f38c28e7d8b977cd40352d825270bd20bcef66ceac3317f2b2274d26f973f0f 00000000 01000000

02000000 f81369411d3fba4eb8575cc858ead8a859ef74b94e160a036b8c1c5b023a6fae 957879fdce4d8ab885e32ff307d54e75884da52522cc53d3c4fdb60edb69a098 659a6eaf8d943ad2ff01ec8c79aaa7cb4f57002d49d9b8cf3c9a7974c5bd3608:06000000 1976a9147db10cfe69dae5e67b85d7b59616056e68b3512288ac f1a2010000000000 fdffffff 0f38c28e7d8b977cd40352d825270bd20bcef66ceac3317f2b2274d26f973f0f 00000000 01000000
"""