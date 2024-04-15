import os
import json
import hashlib


def validate_p2pkh_txn(scriptsig_asm, scriptsig, scriptpubkey_asm):
    stack = []
    signature = scriptsig_asm[1]
    pubkey = scriptsig_asm[3]

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
            # last = stack[-1]
            # print(last)
            ripemd160_hash = hashlib.new('ripemd160', bytes.fromhex(stack[-1])).digest().hex()
            # ripemd160_hash = hashlib.new('ripemd160', stack[-1].encode()).digest()
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

file_path = os.path.join('mempool', "1ccd927e58ef5395ddef40eee347ded55d2e201034bc763bfb8a263d66b99e5e.json") # file path
if os.path.exists(file_path):
    with open(file_path, 'r') as file:
        txn_data = json.load(file)

scriptsig_asm = txn_data["vin"][0]["scriptsig_asm"].split(" ")
scriptsig = txn_data["vin"][0]["scriptsig"]
scriptpubkey_asm = txn_data["vin"][0]["prevout"]["scriptpubkey_asm"].split(" ")
validate_p2pkh_txn(scriptsig_asm, scriptsig, scriptpubkey_asm)