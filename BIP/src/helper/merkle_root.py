import helper.converter as convert

def merkle_root_calculator(txids):
    """
    Calculate the Merkle root of a list of transaction IDs.

    @param txids: The list of transaction IDs.
    @type  txids: list of str
    @return     : The Merkle root of the provided transaction IDs.
    @rtype      : str
    """
    if len(txids) == 0:
        return None

    # Reverse the txids
    level = [bytes.fromhex(txid)[::-1].hex() for txid in txids]

    while len(level) > 1:
        next_level = []
        for i in range(0, len(level), 2):
            if i + 1 == len(level):
                # Duplicate the last one, in case of an odd number of elements
                pair_hash = convert.to_hash256(level[i] + level[i])
            else:
                pair_hash = convert.to_hash256(level[i] + level[i + 1])
            next_level.append(pair_hash)
        level = next_level
    return level[0]