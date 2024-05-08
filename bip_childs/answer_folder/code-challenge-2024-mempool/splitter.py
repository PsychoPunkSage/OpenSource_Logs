import json
import os
import shutil


def read_transactionsII():
    txn_ids = []
    mempool_dir = "valid-mempool"
    c_p2sh = 0
    c_p2tr = 0
    c_p2pkh = 0
    c_p2wsh = 0
    c_p2wpkh = 0
    try:
        for filename in os.listdir(mempool_dir):
            # print("oh yes")
            with open(os.path.join(mempool_dir, filename), "r") as file:
                txn_data = json.load(file)
                type_txn = txn_data["vin"][0]["prevout"]["scriptpubkey_type"]
                    
                if "p2pkh" in type_txn:
                    c_p2pkh += 1
                    p2pkh_dir = os.path.join(os.path.dirname(mempool_dir), "p2pkh_txn")
                    if not os.path.exists(p2pkh_dir):
                        os.makedirs(p2pkh_dir)
                    destination_file = os.path.join(p2pkh_dir, filename)
                    shutil.copyfile(os.path.join(mempool_dir, filename), destination_file)

                if "p2wpkh" in type_txn:
                    c_p2wpkh += 1
                    p2wpkh_dir = os.path.join(os.path.dirname(mempool_dir), "p2wpkh_txn")
                    if not os.path.exists(p2wpkh_dir):
                        os.makedirs(p2wpkh_dir)
                    destination_file = os.path.join(p2wpkh_dir, filename)
                    shutil.copyfile(os.path.join(mempool_dir, filename), destination_file)
                    
                if "p2sh" in type_txn:
                    c_p2sh += 1
                    p2sh_dir = os.path.join(os.path.dirname(mempool_dir), "p2sh_txn")
                    if not os.path.exists(p2sh_dir):
                        os.makedirs(p2sh_dir)
                    destination_file = os.path.join(p2sh_dir, filename)
                    shutil.copyfile(os.path.join(mempool_dir, filename), destination_file)
                    
                if "p2wsh" in type_txn:
                    c_p2wsh += 1
                    p2wsh_dir = os.path.join(os.path.dirname(mempool_dir), "p2wsh_txn")
                    if not os.path.exists(p2wsh_dir):
                        os.makedirs(p2wsh_dir)
                    destination_file = os.path.join(p2wsh_dir, filename)
                    shutil.copyfile(os.path.join(mempool_dir, filename), destination_file)
                    
                
                if "p2tr" in type_txn:
                    c_p2tr += 1
                    p2tr_dir = os.path.join(os.path.dirname(mempool_dir), "p2tr_txn")
                    if not os.path.exists(p2tr_dir):
                        os.makedirs(p2tr_dir)
                    destination_file = os.path.join(p2tr_dir, filename)
                    shutil.copyfile(os.path.join(mempool_dir, filename), destination_file)

    except Exception as e:
        print("Error:", e)
        return None
    
    print(f"p2sh::> {c_p2sh}")
    print(f"p2tr::> {c_p2tr}")
    print(f"p2ms::> {c_p2ms}")
    print(f"p2pk::> {c_p2pk}")
    print(f"p2pkh::> {c_p2pkh}")
    print(f"p2wpkh::> {c_p2wpkh}")
    print(f"p2wsh::> {c_p2wsh}")
read_transactionsII()