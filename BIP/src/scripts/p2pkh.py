import os
import json
import hashlib
import coincurve

def validate_signature(signature, message, publicKey):
    b_sig = bytes.fromhex(signature)
    b_msg = bytes.fromhex(message)
    b_pub = bytes.fromhex(publicKey)
    return coincurve.verify_signature(b_sig, b_msg, b_pub)

def segwit_txn_data(txn_id):
    txn_hash = ""

    file_path = os.path.join("mempool", f"{txn_id}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # Version
            txn_hash += f"{_little_endian(data['version'], 4)}"

            serialized_txid_vout = ""
            for iN in data["vin"]:
                serialized_txid_vout += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
                serialized_txid_vout += f"{_little_endian(iN['vout'], 4)}"
                # serialized_txid_vout += " "
            print(serialized_txid_vout)
            hash256_stv = hashlib.sha256(hashlib.sha256(bytes.fromhex(serialized_txid_vout)).digest()).digest().hex()
            print(f"hash256::> {hash256_stv}")
    return txn_hash
            # # MArker + flag
            # txn_hash += "0001"
            # # No. of inputs:
            # txn_hash += f"{str(_to_compact_size(len(data['vin'])))}"
            # # Inputs
            # for iN in data["vin"]:
            #     txn_hash += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
            #     txn_hash += f"{_little_endian(iN['vout'], 4)}"
            #     txn_hash += f"{_to_compact_size(len(iN['prevout']['scriptpubkey'])//2)}" # FLAG@> maybe not divided by 2
            #     txn_hash += f"{iN['prevout']['scriptpubkey']}"
            #     txn_hash += f"{_little_endian(iN['sequence'], 4)}"

            # # No. of outputs
            # txn_hash += f"{str(_to_compact_size(len(data['vout'])))}"

            # # Outputs
            # for out in data["vout"]:
            #     txn_hash += f"{_little_endian(out['value'], 8)}"
            #     txn_hash += f"{_to_compact_size(len(out['scriptpubkey'])//2)}"
            #     txn_hash += f"{out['scriptpubkey']}"

            # # # witness
            # # for i in data["vin"]:
            # #     if "witness" in i and i["witness"]:
            # #         txn_hash += f"{_to_compact_size(len(i['witness']))}"
            # #         for j in i["witness"]:
            # #             txn_hash += f"{_to_compact_size(len(j) // 2)}"
            # #             txn_hash += f"{j}"

            # # Locktime
            # txn_hash += f"{_little_endian(data['locktime'], 4)}"
            # # print(f"txn_hash: {txn_hash}")


def legacy_txn_data(txn_id):
    txn_hash = ""

    file_path = os.path.join("mempool", f"{txn_id}.json")
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

print(segwit_txn_data("1ccd927e58ef5395ddef40eee347ded55d2e201034bc763bfb8a263d66b99e5e"))
# def rs(signature):
#     r, s = sigdecode_der(bytes.fromhex(signature), secp256k1_generator.order)
#     print(f"r: {r}, s: {s}")
#     return (r, s)

def hash160(hex_input):
    # print(hex_input)
    sha = hashlib.sha256(bytes.fromhex(hex_input)).hexdigest()
    hash_160 = hashlib.new('ripemd160')
    hash_160.update(bytes.fromhex(sha))

    return hash_160.hexdigest()

##########
## MAIN ##
##########
def validate_p2pkh_txn(signature, pubkey, scriptpubkey_asm, txn_data):
    stack = []

    stack.append(signature)
    stack.append(pubkey)

    # print(stack)

    for i in scriptpubkey_asm:
        if i == "OP_DUP":
            stack.append(stack[-1])
            print("===========")
            print("OP_DUP")
            # print(stack)

        if i == "OP_HASH160":
            print("===========")
            print("OP_HASH160")
            ripemd160_hash = hash160(stack[-1])
            stack.pop(-1)
            # print(stack)
            stack.append(ripemd160_hash)
            # print(stack)

        if i == "OP_EQUALVERIFY":
            print("===========")
            print("OP_EQUALVERIFY")
            if stack[-1] != stack[-2]:
                return False
            else:
                stack.pop(-1)
                # print(stack)
                stack.pop(-2)
                # print(stack)

        if i == "OP_CHECKSIG":
            print("===========")
            print("OP_CHECKSIG")
            if signature[-2:] == "01": # SIGHASH_ALL
                der_sig = signature[:-2]
                msg = txn_data + "01000000"
                msg_hash = hashlib.sha256(bytes.fromhex(msg)).digest().hex()
                return validate_signature(der_sig, msg_hash, pubkey)

        if i == "OP_PUSHBYTES_20":
            print("===========")
            print("OP_PUSHBYTES_20")
            stack.append(scriptpubkey_asm[scriptpubkey_asm.index("OP_PUSHBYTES_20") + 1])
            # print(stack)

###<INJECTION>###
# file_path = os.path.join('mempool', "0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240.json") # file path
# filename = "1ccd927e58ef5395ddef40eee347ded55d2e201034bc763bfb8a263d66b99e5e"
filename = "0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240"
file_path = os.path.join('mempool', f"{filename}.json") # file path
if os.path.exists(file_path):
    with open(file_path, 'r') as file: 
        txn_data = json.load(file)
scriptsig_asm = txn_data["vin"][0]["scriptsig_asm"].split(" ")
scriptpubkey_asm = txn_data["vin"][0]["prevout"]["scriptpubkey_asm"].split(" ")
print(legacy_txn_data(filename))
print(validate_p2pkh_txn(scriptsig_asm[1], scriptsig_asm[3], scriptpubkey_asm, legacy_txn_data(filename)))

"""
02000000 0001 02 659a6eaf8d943ad2ff01ec8c79aaa7cb4f57002d49d9b8cf3c9a7974c5bd3608 06000000 19 76a9147db10cfe69dae5e67b85d7b59616056e68b3512288ac fdffffff 2cbc395e5c16b1204f1ced9c0d1699abf5abbbb6b2eee64425c55252131df6c4 00000000 16 00146dee3ed7e9a03ad379f2f78d13138f9141c794ed fdffffff 01 878a03000000000017a914f043430ec4acf2cc3233309bbd1e43ae5efc81748700000000
020000000125c9f7c56ab4b9c358cb159175de542b41c7d38bf862a045fa5da51979e37ffb010000001976a914286eb663201959fb12eff504329080e4c56ae28788acffffffff0254e80500000000001976a9141ef7874d338d24ecf6577e6eadeeee6cd579c67188acc8910000000000001976a9142e391b6c47778d35586b1f4154cbc6b06dc9840c88ac00000000
"""