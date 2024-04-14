import os
import json
import hashlib


def validate_p2pkh_txn(scriptsig_asm, scriptsig, scriptpubkey_asm):
    stack = []
    signature = ""
    pubkey = ""

    stack.append(signature)
    stack.append(pubkey)

    for i in scriptpubkey_asm:
        if i == "OP_DUP":
            stack.append(stack[-1])
        if i == "OP_HASH160":
            ripemd160_hash = hashlib.new('ripemd160', bytes.fromhex(stack[-1])).digest()
            stack.pop(stack[-1])
            stack.append(ripemd160_hash.hex())
        if i == "OP_EQUALVERIFY":
            if stack[-1] != stack[-2]:
                return False
            continue
        if i == "OP_CHECKSIG":
            pass

        else:
            stack.append(i)

# validate_p2pkh_txn("1ccd927e58ef5395ddef40eee347ded55d2e201034bc763bfb8a263d66b99e5e")