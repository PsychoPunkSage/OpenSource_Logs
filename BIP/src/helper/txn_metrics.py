import os
import json
import helper.txn_info as txinfo


################
## Txn Weight ##
################
def txn_weight(txn_filename):
    """
    Validate a transaction's weight based on its filename.

    @param txn_filename : The filename of the transaction to calculate the weight for.
    @type  txn_filename : string
    @return             : A list containing the transaction's weight information:
                              - Index 0: The transaction's size in bytes.
                              - Index 1: The transaction's weight.
                              - Index 2: The transaction's virtual weight.
    @rtype              : list [int, int, float]
    """
    tx_bytes = len(txinfo.create_raw_txn_data_full(txn_filename))//2
    tx_weight = 4*(len(txinfo.create_raw_txn_data_min(txn_filename))//2) + (tx_bytes - len(txinfo.create_raw_txn_data_min(txn_filename))//2)
    tx_virtual_weight = tx_weight/4

    return [tx_bytes, tx_weight, tx_virtual_weight]

##########
## FEES ##
##########
def fees(txn_filename):
    """
    Calculate the fees for a transaction based on its filename.

    @param txn_filename : The filename of the transaction to calculate the fees for.
    @type  txn_filename : string
    @return             : The transaction's fees, calculated as the sum of input amounts minus the sum of output amounts.
    @rtype              : int
    """
    file_path = os.path.join('mempool', f'{txn_filename}.json') # file path
    if os.path.exists(file_path):
        # Read the JSON data from the file
        with open(file_path, 'r') as file:
            txn_data = json.load(file)

    amt_vin = sum([vin["prevout"]["value"] for vin in txn_data["vin"]])
    amt_vout = sum([vout["value"] for vout in txn_data["vout"]])

    return amt_vin - amt_vout