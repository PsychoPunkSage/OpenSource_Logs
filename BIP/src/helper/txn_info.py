import os
import json
import helper.converter as convert

def create_raw_txn_data_min(txn_id):
    txn_data = ""

    file_path = os.path.join("mempool", f"{txn_id}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # Version
            txn_data += f"{convert.to_little_endian(data['version'], 4)}"
            # No. of inputs:
            txn_data += f"{str(convert.to_compact_size(len(data['vin'])))}"
            # Inputs
            for iN in data["vin"]:
                txn_data += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
                txn_data += f"{convert.to_little_endian(iN['vout'], 4)}"
                txn_data += f"{convert.to_compact_size(len(iN['scriptsig'])//2)}"
                txn_data += f"{iN['scriptsig']}"
                txn_data += f"{convert.to_little_endian(iN['sequence'], 4)}"

            # No. of outputs
            txn_data += f"{str(convert.to_compact_size(len(data['vout'])))}"

            # Outputs
            for out in data["vout"]:
                txn_data += f"{convert.to_little_endian(out['value'], 8)}"
                txn_data += f"{convert.to_compact_size(len(out['scriptpubkey'])//2)}"
                txn_data += f"{out['scriptpubkey']}"

            # Locktime
            txn_data += f"{convert.to_little_endian(data['locktime'], 4)}"
    return txn_data

def create_raw_txn_data_full(txn_id):
    txn_hash = ""

    file_path = os.path.join("mempool", f"{txn_id}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # Version
            txn_hash += f"{convert.to_little_endian(data['version'], 4)}"
            # Marker+flags (if any `vin` has empty scriptsig)
            if any(i.get("scriptsig") == "" for i in data["vin"]):
                txn_hash += "0001"
            # No. of inputs:
            txn_hash += f"{str(convert.to_compact_size(len(data['vin'])))}"
            # Inputs
            for iN in data["vin"]:
                txn_hash += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
                txn_hash += f"{convert.to_little_endian(iN['vout'], 4)}"
                txn_hash += f"{convert.to_compact_size(len(iN['scriptsig'])//2)}"
                txn_hash += f"{iN['scriptsig']}"
                txn_hash += f"{convert.to_little_endian(iN['sequence'], 4)}"

            # No. of outputs
            txn_hash += f"{str(convert.to_compact_size(len(data['vout'])))}"

            # Outputs
            for out in data["vout"]:
                txn_hash += f"{convert.to_little_endian(out['value'], 8)}"
                txn_hash += f"{convert.to_compact_size(len(out['scriptpubkey'])//2)}"
                txn_hash += f"{out['scriptpubkey']}"

            # witness
            for i in data["vin"]:
                if "witness" in i and i["witness"]:
                    txn_hash += f"{convert.to_compact_size(len(i['witness']))}"
                    for j in i["witness"]:
                        txn_hash += f"{convert.to_compact_size(len(j) // 2)}"
                        txn_hash += f"{j}"

            # Locktime
            txn_hash += f"{convert.to_little_endian(data['locktime'], 4)}"
    return txn_hash

def get_txn_id(txn_id):
    txn_data = create_raw_txn_data_min(txn_id)
    return convert.to_hash256(txn_data)

'''
wTXID(Legacy) == TXID(Legacy) ===> reverse_bytes(SHA256(txn_data))

wTXID Commitment === HASH256(merkle root for all of the wTXIDs <witness_root_hash>  | witness_reserved_value)
        --> Must have `COINBASE_TXN` at the begining


p2pkh ::> 0a331187bb44a28b342bd2fdfd2ff58147f0e4e43444b5efd89c71f3176caea6.json :: 0a331187bb44a28b342bd2fdfd2ff58147f0e4e43444b5efd89c71f3176caea6
p2wpkh::> 0a3fa2941f316cbf05d7a708f180a4f7cd8034f33ccfea77091252354da41e61.json :: 0a3fa2941f316cbf05d7a708f180a4f7cd8034f33ccfea77091252354da41e61
'''

def txid(txn_id):
    txn_hash = ""

    file_path = os.path.join("mempool", f"{txn_id}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # Version
            txn_hash += f"{convert.to_little_endian(data['version'], 4)}"

            # No. of inputs:
            txn_hash += f"{str(convert.to_compact_size(len(data['vin'])))}"
            # Inputs
            for iN in data["vin"]:
                txn_hash += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
                txn_hash += f"{convert.to_little_endian(iN['vout'], 4)}"
                txn_hash += f"{convert.to_compact_size(len(iN['scriptsig'])//2)}"
                txn_hash += f"{iN['scriptsig']}"
                txn_hash += f"{convert.to_little_endian(iN['sequence'], 4)}"

            # No. of outputs
            txn_hash += f"{str(convert.to_compact_size(len(data['vout'])))}"

            # Outputs
            for out in data["vout"]:
                txn_hash += f"{convert.to_little_endian(out['value'], 8)}"
                txn_hash += f"{convert.to_compact_size(len(out['scriptpubkey'])//2)}"
                txn_hash += f"{out['scriptpubkey']}"

            # Locktime
            txn_hash += f"{convert.to_little_endian(data['locktime'], 4)}"
    return convert.to_reverse_bytes_string(convert.to_hash256(txn_hash))

def wtxid(txn_id):
    txn_hash = ""

    file_path = os.path.join("mempool", f"{txn_id}.json")
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            data = json.load(f)
            # Version
            txn_hash += f"{convert.to_little_endian(data['version'], 4)}"
            # Marker+flags (if any `vin` has empty scriptsig)
            if any(i.get("scriptsig") == "" for i in data["vin"]):
                txn_hash += "0001"
            # No. of inputs:
            txn_hash += f"{str(convert.to_compact_size(len(data['vin'])))}"
            # Inputs
            for iN in data["vin"]:
                txn_hash += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
                txn_hash += f"{convert.to_little_endian(iN['vout'], 4)}"
                txn_hash += f"{convert.to_compact_size(len(iN['scriptsig'])//2)}"
                txn_hash += f"{iN['scriptsig']}"
                txn_hash += f"{convert.to_little_endian(iN['sequence'], 4)}"

            # No. of outputs
            txn_hash += f"{str(convert.to_compact_size(len(data['vout'])))}"

            # Outputs
            for out in data["vout"]:
                txn_hash += f"{convert.to_little_endian(out['value'], 8)}"
                txn_hash += f"{convert.to_compact_size(len(out['scriptpubkey'])//2)}"
                txn_hash += f"{out['scriptpubkey']}"

            # witness
            for i in data["vin"]:
                if "witness" in i and i["witness"]:
                    txn_hash += f"{convert.to_compact_size(len(i['witness']))}"
                    for j in i["witness"]:
                        txn_hash += f"{convert.to_compact_size(len(j) // 2)}"
                        txn_hash += f"{j}"

            # Locktime
            txn_hash += f"{convert.to_little_endian(data['locktime'], 4)}"
    return txn_hash

def txid_dict(txn_dict):
    txn_hash = ""
    data = txn_dict
    # Version
    txn_hash += f"{convert.to_little_endian(data['version'], 4)}"

    # No. of inputs:
    txn_hash += f"{str(convert.to_compact_size(len(data['vin'])))}"
    # Inputs
    for iN in data["vin"]:
        txn_hash += f"{bytes.fromhex(iN['txid'])[::-1].hex()}"
        txn_hash += f"{convert.to_little_endian(iN['vout'], 4)}"
        txn_hash += f"{convert.to_compact_size(len(iN['scriptsig'])//2)}"
        txn_hash += f"{iN['scriptsig']}"
        txn_hash += f"{convert.to_little_endian(iN['sequence'], 4)}"

    # No. of outputs
    txn_hash += f"{str(convert.to_compact_size(len(data['vout'])))}"

    # Outputs
    for out in data["vout"]:
        txn_hash += f"{convert.to_little_endian(out['value'], 8)}"
        txn_hash += f"{convert.to_compact_size(len(out['scriptpubkey'])//2)}"
        txn_hash += f"{out['scriptpubkey']}"

    # Locktime
    txn_hash += f"{convert.to_little_endian(data['locktime'], 4)}"
    return txn_hash

# filename = "e4020c97eb2eb68055362d577e7068a128ceb887a33260062bb3ba2820b9bd30"
# filename = "c1b27a173feead93944952612148c8972e5837d4d564dda8b96639561402ad2e"
# filename = "0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240"
# print(f"txn_hash = {wtxid(filename)}\n")
# tx_id = (convert.to_hash256(wtxid(filename)))
# print(f"txid::> {convert.to_reverse_bytes_string(tx_id)}")
'''
NON_SEGWIT
txid = convert.to_hash256(create_raw_txn_data_full("0a8b21af1cfcc26774df1f513a72cd362a14f5a598ec39d915323078efb5a240"))

896aeeb4d8af739da468ad05932455c639073fa3763d3256ff3a2c86122bda4e >> Actual txn_id
4eda2b12862c3aff56323d76a33f0739c655249305ad68a49d73afd8b4ee6a89.json >> present in valid_mempool


SEGWIT::
tx_id = (convert.to_hash256(txid("e4020c97eb2eb68055362d577e7068a128ceb887a33260062bb3ba2820b9bd30")))

0a3fa2941f316cbf05d7a708f180a4f7cd8034f33ccfea77091252354da41e61.json >> present in valid_mempool
'''

"""
"01000000000101b689a040d64727dc044de15994ea05b00d8f67c850dc55063e11899caafa3649010000000000000000023334140000000000160014e622478beaeef0370269ab49d7858d12225742e4d00a950c000000001600144e885d4290508124d3a151876771097472c20b4902473044022011fd827385911acbf05a834a3b38ea62086054b7e122d337ead1dbea6bbdf6b602204f7f9dadee670684802bf3c935f6e341cf7785e0ce89a0d5072b078740f767e1012103ed74445e27c8d9527f0f642368b8082c35b1b6eaeeadf1973023c4590c0b620300000000"
"01000000000101be70c3da68a5c19c1428610c36409457ba007db7278e1bc30447722d87aee5490200000000ffffffff14ec470100000000001600147c948956e090a54e16c72d4d332c22c54dd17263e84900000000000017a914f68e91e73c0597e917dbea85398df152e617513787981d03000000000017a914ea872350d6389942d81620079f01e1a9b7e8303887c0c62d000000000016001466d1dc307e64dee33786cc88f64ceeb267ab6bf034f101000000000016001443904ede162e4d87e586528dd8510e2f0e2b7586c0230d0000000000160014a916426a91d13cfbf85b64111fd1e458f3913e40602160000000000017a9145f6f65b0eb55311f9bec04c7d61911adc53d1cab8739160100000000001976a9148e0887a66d03b595b90965dad236c3ef4ccb5a0088ac1a4b1b0000000000160014ee608e0afe2dc5bf425d56a8d19c1221fef18787733900000000000016001454c0630e502b32a67e4b433a4fe862cb8534eb5c29350000000000001976a914290c63cf2ec46bd799dbe8d41d9976311fe214c588ac373e01000000000017a914cac78dd82fd32df44fc91b9ae7b2dda665e3b27187026d00000000000017a9148fe1b87ec2c0058fb51c523da6ccc97569f2a78987a3d400000000000017a914c84e4d1a90f408beb79deb57a464516ea51ba0be87e02a2501000000001600140687f01fb5f214ab87cda086561fb46248ff0c35bc550500000000001976a9142e0795e029dd3641ec6bbc5a7913422a1f1b149288ace1aa020000000000160014937a331982b2c4f7dffdfeb68f758ee7e25e9c67c34f000000000000160014ab60143550dbea1e8b1ec3618dfbb772968519d83a0e0400000000001976a9147c25aad0244cd9ce10952f745694d618ad3c29ae88acd40c040000000000160014d7d93b0092672329d9ed5c2828e73e97df5b0c0702483045022100c37d540af42edfd1571f2a7f93ac927c69d97b9d317be3168a755f430298b5e3022046c5fce88a6a2248a77bfd4612d0ebf36c4648a2505ac25b413485b5deeca8f3012102de3cd080021c2a0eb2a6693a3aef0282ea2e7af74852d6ef92a6f0bff4bb80b900000000"
"02000000000101d6e3b57d6e9863b888aa71c05a8bf1de7425ac0ee55f2b597ab8e975329397020200000000ffffffff012202000000000000160014060e51cb98209df71b6eeb7eec1005a6ae398ed20340fa2b8396041b724d4e7fe4ecca5e28b4ecebffcd046f7ef13ce9ce3b188d6f81ac4596f8f161933609c0f492081412e8a6643222a0c59269b69b9466cefbf39efd0c012060008877fcfdf2cff418dcc9571a1240ae643a8ad3d3132c386a62d2aa2955c9ac0063036f726401011e6170706c69636174696f6e2f6a736f6e3b636861727365743d7574662d38004cbf7b2270223a22706f772d3230222c226f70223a226d696e74222c227469636b223a2250455045222c22616d74223a2231303030222c22736f6c7574696f6e223a22504550453a6263317171633839726a7563797a776c77786d7761646c7763797139353668726e726b6a6833637361723a303030303030303030303030303030303030303139393431613561653132383937363539383134343239323533333030353762306461393666336465613163353a3638343237303638353638227d6821c0ebc23c5cb62efa16bafbb201dcb9689a89d3be5df33e83929d6fd8751a90bb4200000000"
"010000000001010bd543ec2097c9711bee84bdb4a7a71715de0e8c1733dc063e4db7c4b6a4e6f30100000000ffffffff02f8240100000000001600149ad65fdbf649321a77ead284fe9e07d4fc59bc7dba0d420000000000160014c8e74e772f568d0cde369a4415b6dc5fd442227802473044022062a763cbdf5045bcbdf9c70c2e286ad6da68f07067ce4b1768fe634ea6cbc21902201341b46e3b8684400194df8f369be7864463a0530f5e8035696c18544f721c10012103aba8226eaddb95be2d2a621bde91071f86189fce989bc669b8a83cfa16cd1c9600000000"
"02000000000101af4737b3cd2a5684061015a97ace866916f2726a1ed47dd65e5164f6f4a3e8810600000000fdffffff01c1bdcc0300000000160014a93300870c8d1d91852760c3235b89361f45339302473044022074aabd3dee2092ad0dbc623468fd0811ff3a613058b69fa4b22b0fe299f69a2e02200d244de85cb89ee9865f404b8b485521fc53e5e5972f57f8ebe718e1a3fe55920121039e201100c8cca000c9ac73112bf7ffc4f4023028589734b22ce4fa0f48e549c000000000"
"""
print(len(wtxid))
l = ["02d3bb4e96de666d9309dc5e7715c4c1b10cdac8a02eb231aba56c102e166258", "83a5a0bc26c4c7f04e2db47ff11549161a72d1a70cc3908dc5d6fc7b4c0135e8", "73a06cfbd5ffd328278d97756265c27c32fd28b32985d07815ebda93d5c464e9", "556e80b02f18b0399305fb594a7fa7f7f6b98099f17c179a893202bb74f1386f", "4a90044907c7608b8859eebc617e05a24190a1b3961e303d262e791edc461665"]
