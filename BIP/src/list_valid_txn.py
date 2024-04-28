import os
import validate_txn
# import helper.merkle_root as merkle 
# import helper.converter as convert
# import helper.get_txn_id as tx_id
'''
@title read transaction form mempool
@notice parses each json object in `mempool` and append an internal list with all txn_ids
@return list of transaction ids
'''
def read_transactions():
    txn_ids = []
    mempool_dir = "mempool"
    try:
        for filename in os.listdir(mempool_dir):
            with open(os.path.join(mempool_dir, filename), "r") as file:
                # locktime ka locha #
                txn_ids.append(filename[:-5])
        return txn_ids
    except Exception as e:
        print("Error:", e)
        return None

def list_valid_txn():
    valid_txn_ids = []
    unchecked_txn_ids = read_transactions()
    for txn_id in unchecked_txn_ids:
        if validate_txn.validate(txn_id):
            valid_txn_ids.append(txn_id)


    return valid_txn_ids

print(len(list_valid_txn()))

# lst = list_valid_txn()
# a_lst = [tx_id.get_txn_id(i) for i in lst]
# with open('output.txt', 'w') as f:
#     for txn_id in a_lst:
#         f.write(txn_id + '\n')

# CalculatedMerkleRoot = str(merkle.merkleCalculator(a_lst), 'utf-8')
# print('Calculated MerkleRoot : ' + CalculatedMerkleRoot)