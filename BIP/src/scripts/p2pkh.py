import os
import json
import hashlib
import ecdsa
import binascii

def parse_signature(signature_hex):
    signature = binascii.unhexlify(signature_hex)
    der_signature = ecdsa.util.sigdecode_der(signature, curve=ecdsa.SECP256k1.curve)

    R, S = der_signature

    R_hex = hex(R)
    S_hex = hex(S)
    print(f"R::> {R_hex}")
    print(f"S::> {S_hex}")
    return R_hex, S_hex


def hash160(hex_input):
    print(hex_input)
    sha = hashlib.sha256(bytes.fromhex(hex_input)).hexdigest()
    hash_160 = hashlib.new('ripemd160')
    hash_160.update(bytes.fromhex(sha))

    return hash_160.hexdigest()

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
            parse_signature(stack[0])
            return True

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
validate_p2pkh_txn(scriptsig_asm[1], scriptsig_asm[3], scriptpubkey_asm)