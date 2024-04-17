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

def segwit_txn_data(txn_id):
    file_path = os.path.join("mempool", f"{txn_id}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # Version
            ver = f"{_little_endian(data['version'], 4)}"

            # HASH256 (txid + vout) || HASH256 (sequece)
            serialized_txid_vout = ""
            serialized_sequese= ""
            for iN in data["vin"]:
                serialized_txid_vout += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
                serialized_txid_vout += f"{_little_endian(iN['vout'], 4)}"
                serialized_sequese += f"{_little_endian(iN['sequence'], 4)}"
            
            # Outputs
            serialized_output= ""
            for out in data["vout"]:
                serialized_output += f"{_little_endian(out['value'], 8)}"
                serialized_output += f"{_to_compact_size(len(out['scriptpubkey'])//2)}"
                serialized_output += f"{out['scriptpubkey']}"

            ###############################################################################
            # TXN Specific #
            pkh = f"{data['vin'][0]['prevout']['scriptpubkey'][6:-4]}" 
            scriptcode = f"1976a914{pkh}88ac"

            in_amt = f"{_little_endian(data['vin'][0]['prevout']['value'], 8)}"

            sequence_txn = f"{_little_endian(data['vin'][0]['sequence'], 4)}"
            ###############################################################################

            locktime = f"{_little_endian(data['locktime'], 4)}"
            print(serialized_txid_vout)
            print(serialized_sequese)
            print(serialized_output)
            print(sequence_txn)
            hash256_stv = hashlib.sha256(hashlib.sha256(bytes.fromhex(serialized_txid_vout)).digest()).digest().hex()
            hash256_seq = hashlib.sha256(hashlib.sha256(bytes.fromhex(serialized_sequese)).digest()).digest().hex()
            hash256_out = hashlib.sha256(hashlib.sha256(bytes.fromhex(serialized_output)).digest()).digest().hex()
            # serialized_txid_vout


            print(f"hash256 (txid + vout)::> {hash256_stv}")
            print(f"hash256 (sequesnce)  ::> {hash256_seq}")
            print(f"hash256 (output)     ::> {hash256_out}")

            # preimage = version + hash256(inputs) + hash256(sequences) + input + scriptcode + amount + sequence + hash256(outputs) + locktime
            preimage = ver + hash256_stv + hash256_seq + serialized_txid_vout + scriptcode + in_amt + sequence_txn + hash256_out + locktime
            preimage += "01000000"
            print(f"preimage ::> {preimage}")
    return hashlib.sha256(bytes.fromhex(preimage)).digest().hex()


def validate_p2wpkh_txn(witness, wit_scriptpubkey_asm):
    wit_sig, wit_pubkey = witness[0], witness[1]
    print(wit_sig, wit_pubkey)
    scriptpubkey_asm = ["OP_DUP", "OP_HASH160", "OP_PUSHBYTES_20", "pkh", "OP_EQUALVERIFY", "OP_CHECKSIG"]
    scriptpubkey_asm[3] = wit_scriptpubkey_asm[-1]
    print(wit_scriptpubkey_asm[-1])
    print(scriptpubkey_asm)
    return p2pkh.validate_p2pkh_txn(wit_sig, wit_pubkey, scriptpubkey_asm)


