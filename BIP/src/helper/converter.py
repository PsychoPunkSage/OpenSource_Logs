import hashlib
from Crypto.Hash import RIPEMD160

def to_compact_size(value):
    """
    Convert an integer value to a compact-size encoding.
    
    @param int value: The integer value to be encoded.

    @return         : The compact-size encoded hexadecimal string.
    @rtype          : string
    """
    if value < 0xfd:
        return value.to_bytes(1, byteorder='little').hex()
    elif value <= 0xffff:
        return (0xfd).to_bytes(1, byteorder='little').hex() + value.to_bytes(2, byteorder='little').hex()
    elif value <= 0xffffffff:
        return (0xfe).to_bytes(1, byteorder='little').hex() + value.to_bytes(4, byteorder='little').hex()
    else:
        return (0xff).to_bytes(1, byteorder='little').hex() + value.to_bytes(8, byteorder='little').hex()

def to_little_endian(num, size):
    """
    Convert an integer to its little-endian byte representation.

    @param int num : The integer value to be converted.
    @param int size: The number of bytes in the output.

    @return        : The little-endian byte representation as a hexadecimal string.
    @rtype         : string
    """
    return num.to_bytes(size, byteorder='little').hex()

def to_hash160(hex_input):
    """
    Calculate the RIPEMD160 hash of the SHA256 hash of the input hexadecimal string.

    @param str hex_input: The input hexadecimal string.

    @return             : The RIPEMD160 hash as a hexadecimal string.
    @rtype              : string
    """
    sha = hashlib.sha256(bytes.fromhex(hex_input)).hexdigest()
    hash_160 = RIPEMD160.new()
    hash_160.update(bytes.fromhex(sha))
    return hash_160.hexdigest()

def to_hash256(hex_input):
    """
    Calculate the double SHA256 hash of the input hexadecimal string.

    @param str hex_input: The input hexadecimal string.

    @return             : The double SHA256 hash as a hexadecimal string.
    @rtype              : string
    """
    return hashlib.sha256(hashlib.sha256(bytes.fromhex(hex_input)).digest()).digest().hex()

def to_sha256(hex_input):
    """
    Calculate the SHA256 hash of the input hexadecimal string.

    @param str hex_input: The input hexadecimal string.

    @return             : The SHA256 hash as a hexadecimal string.
    @rtype              : string
    """
    return hashlib.sha256(bytes.fromhex(hex_input)).digest().hex()

def to_reverse_bytes_string(hex_input):
    """
    Reverse the bytes of the input hexadecimal string.

    @param str hex_input: The input hexadecimal string.

    @return             : The reversed bytes as a hexadecimal string.
    @rtype              : string
    """
    return bytes.fromhex(hex_input)[::-1].hex()