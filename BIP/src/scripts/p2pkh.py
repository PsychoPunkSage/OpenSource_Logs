import os
import json
import ecdsa
import hashlib

def create_raw_txn_hash(txn_id):
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
                txn_hash += f"{_to_compact_size(len(iN['prevout']["scriptpubkey"])//2)}" # FLAG@> maybe not divided by 2
                txn_hash += f"{iN['scriptsig']['prevout']["scriptpubkey"]}"
                txn_hash += f"{_little_endian(iN['sequence'], 4)}"

            # No. of outputs
            txn_hash += f"{str(_to_compact_size(len(data['vout'])))}"

            # Outputs
            for out in data["vout"]:
                txn_hash += f"{_little_endian(out['value'], 8)}"
                txn_hash += f"{_to_compact_size(len(out['scriptpubkey'])//2)}"
                txn_hash += f"{out['scriptpubkey']}"

            # # witness
            # for i in data["vin"]:
            #     if "witness" in i and i["witness"]:
            #         txn_hash += f"{_to_compact_size(len(i['witness']))}"
            #         for j in i["witness"]:
            #             txn_hash += f"{_to_compact_size(len(j) // 2)}"
            #             txn_hash += f"{j}"

            # Locktime
            txn_hash += f"{_little_endian(data['locktime'], 4)}"
            # print(f"txn_hash: {txn_hash}")
    return txn_hash

def create_raw_txn_hash_wo_witness(txn_id):
    txn_hash = ""

    file_path = os.path.join("mempool", f"{txn_id}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # print(f"data: {data}")
            # Version
            txn_hash += f"{_little_endian(data['version'], 4)}"
            # No. of inputs:
            txn_hash += f"{str(_to_compact_size(len(data['vin'])))}"
            # Inputs
            for iN in data["vin"]:
                txn_hash += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
                txn_hash += f"{_little_endian(iN['vout'], 4)}"
                txn_hash += f"{_to_compact_size(len(iN['scriptsig'])//2)}" # FLAG@> maybe not divided by 2
                txn_hash += f"{iN['scriptsig']}"
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

def verify_sig(signature, pubKey, msg_bytes):
    message = msg_bytes
    public_key = pubKey
    sig = signature
    print("entering VK")
    vk = ecdsa.VerifyingKey.from_string(bytes.fromhex(public_key), curve=ecdsa.SECP256k1, hashfunc=hashlib.sha256) # the default is sha1
    print(f"VK ::> {vk}")
    return vk.verify(bytes.fromhex(sig), bytes.fromhex(message))


def hash160(hex_input):
    print(hex_input)
    sha = hashlib.sha256(bytes.fromhex(hex_input)).hexdigest()
    hash_160 = hashlib.new('ripemd160')
    hash_160.update(bytes.fromhex(sha))

    return hash_160.hexdigest()

def validate_p2pkh_txn(signature, pubkey, scriptpubkey_asm, txn_data):
    stack = []

    stack.append(signature)
    stack.append(pubkey)

    print(stack)

    for i in scriptpubkey_asm:
        if i == "OP_DUP":
            stack.append(stack[-1])
            print("===========")
            print("OP_DUP")
            print(stack)

        if i == "OP_HASH160":
            print("===========")
            print("OP_HASH160")
            ripemd160_hash = hash160(stack[-1])
            stack.pop(-1)
            print(stack)
            stack.append(ripemd160_hash)
            print(stack)

        if i == "OP_EQUALVERIFY":
            print("===========")
            print("OP_EQUALVERIFY")
            if stack[-1] != stack[-2]:
                return False
            else:
                stack.pop(-1)
                print(stack)
                stack.pop(-2)
                print(stack)

        if i == "OP_CHECKSIG":
            print("===========")
            print("OP_CHECKSIG")
            if signature[-2:] == "01":
                der_sig = signature[:-2]
                msg = txn_data + "01000000"
                msg_hash = hashlib.sha256(hashlib.sha256(bytes.fromhex(msg)).digest()).digest().hex()
                print(f"msg: : {msg}")
                print(f"msg_hash: : {msg_hash}")
                verify_sig(der_sig, pubkey, msg_hash)
            # return verify_sig(stack[0], stack[1], bytes.fromhex(txn_data))

        if i == "OP_PUSHBYTES_20":
            print("===========")
            print("OP_PUSHBYTES_20")
            stack.append(scriptpubkey_asm[scriptpubkey_asm.index("OP_PUSHBYTES_20") + 1])
            print(stack)

# file_path = os.path.join('mempool', "1ccd927e58ef5395ddef40eee347ded55d2e201034bc763bfb8a263d66b99e5e.json") # file path
file_path = os.path.join('mempool', "0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240.json") # file path
if os.path.exists(file_path):
    with open(file_path, 'r') as file:
        txn_data = json.load(file)

scriptsig_asm = txn_data["vin"][0]["scriptsig_asm"].split(" ")
# scriptsig = txn_data["vin"][0]["scriptsig"]
scriptpubkey_asm = txn_data["vin"][0]["prevout"]["scriptpubkey_asm"].split(" ")
print(create_raw_txn_hash("0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240"))
# print(create_raw_txn_hash("0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240")+"01000000")
# 0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240
print(validate_p2pkh_txn(scriptsig_asm[1], scriptsig_asm[3], scriptpubkey_asm, create_raw_txn_hash("0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240")))



"""
30450221008f619822a97841ffd26eee942d41c1c4704022af2dd42600f006336ce686353a0220659476204210b21d605baab00bef7005ff30e878e911dc99413edb6c1e022acd01

02000000 01 25c9f7c56ab4b9c358cb159175de542b41c7d38bf862a045fa5da51979e37ffb 01000000 6b4830450221008f619822a97841ffd26eee942d41c1c4704022af2dd42600f006336ce686353a0220659476204210b21d605baab00bef7005ff30e878e911dc99413edb6c1e022acd012102c371793f2e19d1652408efef67704a2e9953a43a9dd54360d56fc93277a5667dffffffff0254e80500000000001976a9141ef7874d338d24ecf6577e6eadeeee6cd579c67188acc8910000000000001976a9142e391b6c47778d35586b1f4154cbc6b06dc9840c88ac0000000001000000
01000000 01 b7994a0db2f373a29227e1d90da883c6ce1cb0dd2d6812e4558041ebbbcfa54b 00000000 19 76a9144299ff317fcd12ef19047df66d72454691797bfc88ac ffffffff 01 983a000000000000 19 76a914b3e2819b6262e0b1f19fc7229d75677f347c91ac88ac 00000000 01000000
                                                                                         76a914286eb663201959fb12eff504329080e4c56ae28788ac
"""