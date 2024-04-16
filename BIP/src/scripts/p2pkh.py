import os
import json
import ecdsa
import hashlib
# from src.helper import pubKey_uncompressor as xy

import ecdsa.util
from ecdsa.util import sigdecode_der

def compressed_pubkey_to_uncompressed(compressed):
    prefix = compressed[:2]
    x = int(compressed[2:], 16)

    p = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f

    y_sq = (x**3 + 7) % p  # everything is modulo p

    y = pow(y_sq, (p+1)//4, p)  # use modular exponentiation
    if prefix == "02" and y % 2 != 0:  # if prefix is 02 and y isn't even, use other y value
        y = (p - y) % p
    if prefix == "03" and y % 2 == 0:  # if prefix is 03 and y is even, use other y value
        y = (p - y) % p

    return (x, y)

"""
Example::> 
Pubkey: 03ab996ad23c7930cee68f950e739fa067aa70a0e63786572b864900985879c4c4
            |_> x: 77616561961719797560395316518092500847148122687187451311177913683967720670404
            |_> y: 69589226102335479499252171152551221549584577390107066575000126767261437088529
"""
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

# def verify_sig(signature, pubKey, msg):
#     msg_bytes = bytes.fromhex(msg)
#     print("entering VK")
#     vk = ecdsa.VerifyingKey.from_string(bytes.fromhex(pubKey), curve=ecdsa.SECP256k1) # the default is sha1
#     try:
#         if vk.verify(signature, msg_bytes, hashfunc=hashlib.sha256, sigdecode=ecdsa.util.sigdecode_der):
#             return True
#         else:
#             print("SHIFTER")
#             return False
#     except Exception as e:
#         print("ERROR (Signature verification)::> ", e)
#         return False
# def verify_signature(signature, public_key, message):
#     # Convert the public key to a VerifyingKey object
#     vk = ecdsa.VerifyingKey.from_string(bytes.fromhex(public_key), curve=ecdsa.SECP256k1, hashfunc=hashlib.sha256)
    
#     # Decode the DER-encoded signature
#     r, s = sigdecode_der(bytes.fromhex(signature))
    
#     # Verify the signature
#     return vk.verify(ecdsa.util.sigencode_der(r, s), bytes.fromhex(message))




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
                pubkey_xy = compressed_pubkey_to_uncompressed(pubkey)
                # print(f"msg: : {msg}")
                # print(f"msg_hash: : {msg_hash}")
                print(f"pubkey_xy: : {pubkey_xy}")
                # verify_sig(der_sig, pubkey, bytes.fromhex(msg_hash))
                # print(verify_signature(der_sig, pubkey, msg))
                print(verify_sig(der_sig, pubkey, msg))
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
        print(txn_data)
scriptsig_asm = txn_data["vin"][0]["scriptsig_asm"].split(" ")
# scriptsig = txn_data["vin"][0]["scriptsig"]
scriptpubkey_asm = txn_data["vin"][0]["prevout"]["scriptpubkey_asm"].split(" ")
print(create_raw_txn_hash("0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240"))
# print(create_raw_txn_hash("0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240")+"01000000")
# 0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240
print(validate_p2pkh_txn(scriptsig_asm[1], scriptsig_asm[3], scriptpubkey_asm, create_raw_txn_hash("0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240")))



"""
30450221008f619822a97841ffd26eee942d41c1c4704022af2dd42600f006336ce686353a0220659476204210b21d605baab00bef7005ff30e878e911dc99413edb6c1e022acd01

02000000 01 25c9f7c56ab4b9c358cb159175de542b41c7d38bf862a045fa5da51979e37ffb 01000000 19 76a914286eb663201959fb12eff504329080e4c56ae28788ac ffffffff 02 54e8050000000000 19 76a9141ef7874d338d24ecf6577e6eadeeee6cd579c67188ac c891000000000000 19 76a9142e391b6c47778d35586b1f4154cbc6b06dc9840c88ac 00000000 01000000
01000000 01 b7994a0db2f373a29227e1d90da883c6ce1cb0dd2d6812e4558041ebbbcfa54b 00000000 19 76a9144299ff317fcd12ef19047df66d72454691797bfc88ac ffffffff 01 983a000000000000 19 76a914b3e2819b6262e0b1f19fc7229d75677f347c91ac88ac 00000000 01000000       
"""