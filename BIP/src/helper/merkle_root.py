import binascii
import hashlib


def hashIt(firstTxHash, secondTxHash):
    unhex_reverse_first = binascii.unhexlify(firstTxHash)[::-1]
    unhex_reverse_second = binascii.unhexlify(secondTxHash)[::-1]

    concat_inputs = unhex_reverse_first+unhex_reverse_second
    first_hash_inputs = hashlib.sha256(concat_inputs).digest()
    final_hash_inputs = hashlib.sha256(first_hash_inputs).digest()
    return binascii.hexlify(final_hash_inputs[::-1])
 
def merkleCalculator(hashList):
    if len(hashList) == 1:
        return hashList[0]
    newHashList = []
    for i in range(0, len(hashList)-1, 2):
        newHashList.append(hashIt(hashList[i], hashList[i+1]))
    if len(hashList) % 2 == 1: # odd, hash last item twice
        newHashList.append(hashIt(hashList[-1], hashList[-1]))
    return merkleCalculator(newHashList)

# print('Expected MerkleRoot :   f3e94742aca4b5ef85488dc37c06c3282295ffec960994b2c0d5ac2a25a95766')

# # Transaction Hashes of block #100000
# txHashes = [
#     '8c14f0db3df150123e6f3dbbf30f8b955a8249b62ac1d1ff16284aefa3d06d87',
#     'fff2525b8931402dd09222c50775608f75787bd2b87e56995a7bdd30f79702c4',
#     '6359f0868171b1d194cbee1af2f16ea598ae8fad666d9b012c8ed2b79a236ec4',
#     'e9a66845e05d5abc0ad04ec80f774a7e585c6e8db975962d069a522137b80c1d'
# ]   

# CalculatedMerkleRoot = str(merkleCalculator(txHashes), 'utf-8')
# print('Calculated MerkleRoot : ' + CalculatedMerkleRoot)

def hash256(hex):
    binary = bytes.fromhex(hex)
    hash1 = hashlib.sha256(binary).digest()
    hash2 = hashlib.sha256(hash1).digest()
    result = hash2.hex()
    return result

def generate_merkle_root(txids):
    if len(txids) == 0:
        return None

    # Reverse the txids
    level = [bytes.fromhex(txid)[::-1].hex() for txid in txids]

    while len(level) > 1:
        next_level = []
        for i in range(0, len(level), 2):
            if i + 1 == len(level):
                # In case of an odd number of elements, duplicate the last one
                pair_hash = hash256(level[i] + level[i])
            else:
                pair_hash = hash256(level[i] + level[i + 1])
            next_level.append(pair_hash)
        level = next_level
    return level[0]