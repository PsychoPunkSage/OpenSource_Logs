import helper.txn_info as txinfo
import helper.converter as convert
import helper.merkle_root as merkle

###############
## CONSTANTS ##
###############
WTXID_COINBASE = bytes(32).hex()
WITNESS_RESERVED_VALUE_HEX = '0000000000000000000000000000000000000000000000000000000000000000'

def calculate_witness_commitment(txn_files):
    """
    Calculate the witness commitment of the transactions in the block.

    @param txn_files: A list of transaction files to include in the calculation.
    @type  txn_files: list

    @return         : The witness commitment calculated for the given transactions.
    @rtype          : str
    """
    wtxids = [WTXID_COINBASE] # must begin with wtxid of Coinbase txn

    # Calculate wtxid of list of transactions
    for tx in txn_files:
        w_txid = txinfo.wtxid(tx)
        wtxids.append(w_txid)

    # Get merkle root of wtxids
    witness_root = merkle.merkle_root_calculator(wtxids)
    print(f"witness root::> {witness_root}")

    # Append witness reserved value at the end.
    witness_reserved_value_hex = WITNESS_RESERVED_VALUE_HEX
    combined_data = witness_root + witness_reserved_value_hex

    # Calculate the hash256 to get witness commitment
    witness_commitment = convert.to_hash256(combined_data)
    return witness_commitment

def create_coinbase_transaction(witness_commitment, fees = 0):
    """
    Creates a coinbase transaction with the given witness commitment and fees.

    @param str witness_commitment: The witness commitment to include in the transaction.
    @param int fees              : The transaction fees (default is 0).

    @return                      : A tuple containing the serialized transaction data and the reversed bytes string of the transaction ID.
    @rtype                       : tuple[str, str]
    """

    # f595814a00000000 -> fees
    fees_le = convert.to_little_endian(fees, 8)

    #########################
    # Coinbase txn template #
    #########################
    tx_template = {
        "version": 1,
        "marker": "00",
        "flag": "01",
        "inputcount": "01",
        "vin": [
            {
                "txid": "0000000000000000000000000000000000000000000000000000000000000000",
                "vout": int("ffffffff", 16),
                "scriptsigsize": 37,
                "scriptsig": "03233708184d696e656420627920416e74506f6f6c373946205b8160a4256c0000946e0100",
                "sequence": int("ffffffff", 16),
            }
        ],
        "outputcount": "02",
        "vout": [
            {
                "value": fees,
                "scriptpubkeysize": "19",
                "scriptpubkey": "76a914edf10a7fac6b32e24daa5305c723f3de58db1bc888ac",
            },
            {
                "value": 0,
                "scriptpubkeysize": "26",
                "scriptpubkey": f"6a24aa21a9ed{witness_commitment}",
            },
        ],
        "witness": [
            {
                "stackitems": "01",
                "0": {
                    "size": "20",
                    "item": "0000000000000000000000000000000000000000000000000000000000000000",
                },
            }
        ],
        "locktime": 0,
    }

    ###########
    # Version #
    ###########
    tx_data = f"{convert.to_little_endian(tx_template['version'],4)}"

    ###################
    # Marker and Flag #
    ###################
    tx_data += tx_template["marker"] + tx_template["flag"]

    ###############
    # Input Count #
    ###############
    tx_data += tx_template["inputcount"]

    #########
    # Input #
    #########
    input_data = tx_template["vin"][0]
    tx_data += input_data["txid"]
    tx_data += f"{convert.to_little_endian(input_data['vout'], 4)}"
    tx_data += f"{convert.to_compact_size(input_data['scriptsigsize'])}"
    tx_data += input_data["scriptsig"]
    tx_data += f"{convert.to_little_endian(input_data['sequence'], 4)}"

    ################
    # Output Count #
    ################
    tx_data += tx_template["outputcount"]

    ###########
    # Outputs #
    ###########
    for output in tx_template["vout"]:
        tx_data += f"{convert.to_little_endian(output['value'], 8)}"
        tx_data += output["scriptpubkeysize"].zfill(2)
        tx_data += output["scriptpubkey"]

    ###########
    # Witness #
    ###########
    witness_data = tx_template["witness"][0]
    tx_data += witness_data["stackitems"]
    tx_data += witness_data["0"]["size"].zfill(2)
    tx_data += witness_data["0"]["item"]

    ############
    # Locktime #
    ############
    tx_data += f"{convert.to_little_endian(tx_template['locktime'], 4)}"
    # print(f"coinbase_txn_data(inside)::> {tx_data}")

    return tx_data, txinfo.coinbase_txn_id(tx_template)
