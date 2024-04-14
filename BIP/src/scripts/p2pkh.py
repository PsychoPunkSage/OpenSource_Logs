import os
import json
import hashlib


def validate_p2pkh_txn(scriptsig_asm, scriptsig, scriptpubkey_asm):
    stack = []
    signature = ""
    pubkey = ""

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
            ripemd160_hash = hashlib.new('ripemd160', bytes.fromhex(stack[-1])).digest()
            stack.pop(stack[-1])
            print(stack)
            stack.append(ripemd160_hash.hex())
            print(stack)

        if i == "OP_EQUALVERIFY":
            print("===========")
            print("OP_EQUALVERIFY")
            if stack[-1] != stack[-2]:
                return False
            else:
                stack.pop(stack[-1])
                stack.pop(stack[-2])

        if i == "OP_CHECKSIG":
            pass

        else:
            stack.append(i)

# validate_p2pkh_txn("1ccd927e58ef5395ddef40eee347ded55d2e201034bc763bfb8a263d66b99e5e")