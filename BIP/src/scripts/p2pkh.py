import os
import json
import hashlib

def ripemd160_hex(hex_input):
    byte_input = bytes.fromhex(hex_input)

    ripemd160_hash = hashlib.new('ripemd160')
    ripemd160_hash.update(byte_input)    
    
    hex_hash = ripemd160_hash.hexdigest()

    return hex_hash

def validate_p2pkh_txn(signature, pubkey, scriptpubkey_asm):
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
            ripemd160_hash = ripemd160_hex(stack[-1])
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
                stack.pop(stack[-1])
                print(stack)
                stack.pop(stack[-2])
                print(stack)

        if i == "OP_CHECKSIG":
            pass

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
scriptsig = txn_data["vin"][0]["scriptsig"]
scriptpubkey_asm = txn_data["vin"][0]["prevout"]["scriptpubkey_asm"].split(" ")
validate_p2pkh_txn(scriptsig_asm[1], scriptsig_asm[3], scriptpubkey_asm)