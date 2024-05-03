import os
import validate_txn

def read_transactions():
    """
    Reads transaction IDs from files in the mempool directory.

    @return : List of transaction IDs. Each transaction ID corresponds to a file name in the mempool directory.
    @rtype  : list
    """
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
    """
    Lists valid transactions from the mempool.

    @return : List of valid transaction files. Transactions are sorted with SEGWIT transactions at the beginning and NON-SEGWIT transactions at the end.
    @rtype  : list
    """
    valid_txn_files = []
    unchecked_txn_ids = read_transactions()
    for txn_file_name in unchecked_txn_ids:
        valid, is_segwit = validate_txn.validate(txn_file_name)
        if valid and is_segwit == 1:
            # Adding SEGWIT txn at begining to give it priority over NON-SEGWIT tx
            valid_txn_files.insert(0, txn_file_name)
        if valid and is_segwit == 0:
            # Adding NON-SEGWIT txn at last to give it less priority over SEGWIT tx
            valid_txn_files.append(txn_file_name)
    return valid_txn_files