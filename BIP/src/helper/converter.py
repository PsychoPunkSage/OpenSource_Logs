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

# hex_in = "0100000001b7994a0db2f373a29227e1d90da883c6ce1cb0dd2d6812e4558041ebbbcfa54b000000001976a9144299ff317fcd12ef19047df66d72454691797bfc88acffffffff01983a0000000000001976a914b3e2819b6262e0b1f19fc7229d75677f347c91ac88ac0000000001000000"
# print(f"hash256::> {to_hash256(hex_in)}")