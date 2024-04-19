import hashlib

def to_compact_size(value):
    if value < 0xfd:
        return value.to_bytes(1, byteorder='little').hex()
    elif value <= 0xffff:
        return (0xfd).to_bytes(1, byteorder='little').hex() + value.to_bytes(2, byteorder='little').hex()
    elif value <= 0xffffffff:
        return (0xfe).to_bytes(1, byteorder='little').hex() + value.to_bytes(4, byteorder='little').hex()
    else:
        return (0xff).to_bytes(1, byteorder='little').hex() + value.to_bytes(8, byteorder='little').hex()

def to_little_endian(num, size):
    return num.to_bytes(size, byteorder='little').hex()

def to_hash160(hex_input):
    # print(hex_input)
    sha = hashlib.sha256(bytes.fromhex(hex_input)).hexdigest()
    hash_160 = hashlib.new('ripemd160')
    hash_160.update(bytes.fromhex(sha))

    return hash_160.hexdigest()

def to_hash256(hex_input):
    return hashlib.sha256(hashlib.sha256(bytes.fromhex(hex_input)).digest()).digest().hex()

def to_sha256(hex_input):
    return hashlib.sha256(bytes.fromhex(hex_input)).digest().hex()

def to_reverse_bytes_string(hex_input):
    return bytes.fromhex(hex_input)[::-1].hex()