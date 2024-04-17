import hashlib
import json
import os
import p2pkh

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

def segwit_txn_data1(txn_id):
    file_path = os.path.join("mempool", f"{txn_id}.json")
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
            hash256_in = hashlib.sha256(hashlib.sha256(bytes.fromhex(serialized_txid_vout)).digest()).digest().hex()
            
            ## (sequense)
            serialized_sequense= ""
            for iN in data["vin"]:
                serialized_sequense += f"{_little_endian(iN['sequence'], 4)}"
            ## HASH256 (sequense)
            hash256_seq = hashlib.sha256(hashlib.sha256(bytes.fromhex(serialized_sequense)).digest()).digest().hex()
            
            ###############################################################################
            # TXN Specific #
            ## TXID and VOUT for the REQUIRED_input
            ser_tx_vout_sp = f"{bytes.fromhex(data['vin'][1]['txid'])[::-1].hex()}{_little_endian(data['vin'][1]['vout'], 4)}"
            print(ser_tx_vout_sp)
            ## Scriptcode
            pkh = f"{data['vin'][1]['prevout']['scriptpubkey'][4:]}" 
            scriptcode = f"1976a914{pkh}88ac"
            ## Input amount
            in_amt = f"{_little_endian(data['vin'][1]['prevout']['value'], 8)}"
            ## SEQUENCE for the REQUIRED_input
            sequence_txn = f"{_little_endian(data['vin'][1]['sequence'], 4)}"
            ###############################################################################

            # Outputs
            serialized_output= ""
            for out in data["vout"]:
                serialized_output += f"{_little_endian(out['value'], 8)}"
                serialized_output += f"{_to_compact_size(len(out['scriptpubkey'])//2)}"
                serialized_output += f"{out['scriptpubkey']}"
            ## HASH256 (output)
            hash256_out = hashlib.sha256(hashlib.sha256(bytes.fromhex(serialized_output)).digest()).digest().hex()

            ## locktime
            locktime = f"{_little_endian(data['locktime'], 4)}"


            # print(serialized_txid_vout)
            # print(serialized_sequense)
            # print(serialized_output)
            # print(sequence_txn)
            # # serialized_txid_vout


            # print(f"hash256 11111 (txid + vout)::> {hash256_stv}")
            # print(f"hash256 11111 (sequesnce)  ::> {hash256_seq}")
            # print(f"hash256 11111 (output)     ::> {hash256_out}")

            # preimage = version + hash256(inputs) + hash256(sequences) + input + scriptcode + amount + sequence + hash256(outputs) + locktime
            preimage = ver + hash256_in + hash256_seq + ser_tx_vout_sp + scriptcode + in_amt + sequence_txn + hash256_out + locktime
            preimage += "01000000"
            print(f"preimage 11111 ::> {preimage}")
    return hashlib.sha256(bytes.fromhex(preimage)).digest().hex()


def validate_p2wpkh_txn(witness, wit_scriptpubkey_asm):
    wit_sig, wit_pubkey = witness[0], witness[1]
    print(wit_sig, wit_pubkey)
    scriptpubkey_asm = ["OP_DUP", "OP_HASH160", "OP_PUSHBYTES_20", "pkh", "OP_EQUALVERIFY", "OP_CHECKSIG"]
    scriptpubkey_asm[3] = wit_scriptpubkey_asm[-1]
    print(wit_scriptpubkey_asm[-1])
    print(scriptpubkey_asm)
    return p2pkh.validate_p2pkh_txn(wit_sig, wit_pubkey, scriptpubkey_asm)


print(segwit_txn_data1("1ccd927e58ef5395ddef40eee347ded55d2e201034bc763bfb8a263d66b99e5e"))

"""
02000000f81369411d3fba4eb8575cc858ead8a859ef74b94e160a036b8c1c5b023a6fae957879fdce4d8ab885e32ff307d54e75884da52522cc53d3c4fdb60edb69a098659a6eaf8d943ad2ff01ec8c79aaa7cb4f57002d49d9b8cf3c9a7974c5bd3608060000002cbc395e5c16b1204f1ced9c0d1699abf5abbbb6b2eee64425c55252131df6c4000000001976a9147db10cfe69dae5e67b85d7b59616056e68b3512288acf1a2010000000000fdffffff0f38c28e7d8b977cd40352d825270bd20bcef66ceac3317f2b2274d26f973f0f0000000001000000
02000000f81369411d3fba4eb8575cc858ead8a859ef74b94e160a036b8c1c5b023a6fae957879fdce4d8ab885e32ff307d54e75884da52522cc53d3c4fdb60edb69a098659a6eaf8d943ad2ff01ec8c79aaa7cb4f57002d49d9b8cf3c9a7974c5bd3608060000002cbc395e5c16b1204f1ced9c0d1699abf5abbbb6b2eee64425c55252131df6c4000000001976a9146dee3ed7e9a03ad379f2f78d13138f9141c794ed88acf306020000000000fdffffff0f38c28e7d8b977cd40352d825270bd20bcef66ceac3317f2b2274d26f973f0f0000000001000000
"""