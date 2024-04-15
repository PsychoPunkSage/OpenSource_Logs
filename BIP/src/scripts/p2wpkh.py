import p2pkh

def validate_p2wpkh_txn(witness, wit_scriptpubkey_asm):
    wit_sig, wit_pubkey = witness[0], witness[1]
    print(wit_sig, wit_pubkey)
    scriptpubkey_asm = ["OP_DUP", "OP_HASH160", "OP_PUSHBYTES_20", "pkh", "OP_EQUALVERIFY", "OP_CHECKSIG"]
    scriptpubkey_asm[3] = wit_scriptpubkey_asm[-1]
    return p2pkh.validate_p2pkh_txn(wit_sig, wit_pubkey, scriptpubkey_asm)


